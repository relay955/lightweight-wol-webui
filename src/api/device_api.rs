use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::http::Status;
use rocket::State;
use serde_json::json;
use serde_json::ser::CharEscape::CarriageReturn;
use lazy_static::lazy_static;
use validator::Validate;
use std::net::UdpSocket;
use crate::api::validate_request;
use crate::module::auth::AuthUser;
use crate::db::Db;
use crate::db::device::{Device, DeviceOperations, MoveDirection, generate_authkey};
use crate::error::{PredefinedApiError, SystemError};
use crate::config::WolConfig;
use crate::module::magic_packet::send_magic_packet;
use crate::module::agent::AgentConnections;

// MAC 주소 정규식 (일반적인 형식: AA:BB:CC:DD:EE:FF 또는 AA-BB-CC-DD-EE-FF)
lazy_static::lazy_static! {
    static ref MAC_ADDRESS_REGEX: regex::Regex = regex::Regex::new(
        r"^([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})$"
    ).unwrap();
}

// MAC 주소 검증 함수
fn validate_mac_address(mac: &str) -> Result<(), validator::ValidationError> {
    if MAC_ADDRESS_REGEX.is_match(mac) {
        Ok(())
    } else {
        Err(validator::ValidationError::new("invalid_mac_address"))
    }
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct GetDeviceRes {
    pub id: i64,
    pub name: String,
    pub mac: String,
    pub ip: String,   
    pub order_num: i64,
    pub authkey: String,
    pub connected: bool,
}

#[get("/devices")]
pub async fn get_devices(db: &Db, agents: &State<AgentConnections>, _auth: AuthUser)
    -> Result<Json<Vec<GetDeviceRes>>, SystemError> {
    let devices = Device::get_all(db).await?;
    let connected_ids = agents.connected_device_ids();
    let mut device_list = Vec::new();
    for device in devices {
        device_list.push(GetDeviceRes {
            id: device.id,
            name: device.name,
            mac: device.mac,
            ip: device.ip,       
            order_num: device.order_num,
            authkey: device.authkey,
            connected: connected_ids.contains(&device.id),
        });
    }
    Ok(Json(device_list))
}

#[get("/device/<id>")]
pub async fn get_device(db: &Db, agents: &State<AgentConnections>, _auth: AuthUser, id: i64)
    -> Result<Json<GetDeviceRes>, SystemError> {
    let device = Device::get(db, id).await?
        .ok_or(PredefinedApiError::NotFound.get())?;

    let device_res = GetDeviceRes {
        id: device.id,
        name: device.name,
        mac: device.mac,
        ip: device.ip,       
        order_num: device.order_num,
        authkey: device.authkey,
        connected: agents.connected_device_ids().contains(&id),
    };
    
    Ok(Json(device_res))
}

#[derive(Deserialize, Validate)]
#[serde(crate = "rocket::serde")]
pub struct PostDeviceReq {
    pub id: Option<i64>,
    #[validate(length(min = 1, message = "이름은 공백일 수 없습니다"))]
    pub name: String,
    #[validate(custom(function = "validate_mac_address", message = "올바른 MAC 주소 형식이 아닙니다 (예: AA:BB:CC:DD:EE:FF)"))]
    pub mac: String,
    pub ip: String, 
}

#[post("/device", data = "<req>")]
pub async fn create_device(db: &Db, _auth: AuthUser, req: Json<PostDeviceReq>,
) -> Result<Status, SystemError> {
    validate_request(&*req)?;

    let max_order = Device::get_max_order_num(&db.0).await?;

    let device = Device{
        id: 0,
        name: req.name.clone(),
        mac: req.mac.clone(),
        ip: req.ip.to_string(),       
        order_num: max_order + 1,
        authkey: generate_authkey(),
    };
    Device::insert(db,&device).await?;
    Ok(Status::Ok)
}


#[put("/device", data = "<req>")]
pub async fn update_device(db: &Db, _auth: AuthUser, req: Json<PostDeviceReq>)
                           -> Result<Status, SystemError> {
    validate_request(&*req)?;
    
    if req.id.is_none() {
        return Err(PredefinedApiError::InvalidRequest.get());
    }
    
    let mut device = Device::get(db, req.id.unwrap()).await?
        .ok_or(PredefinedApiError::NotFound.get())?;

    // 필드 업데이트
    device.name = req.name.clone();
    device.mac = req.mac.clone();
    device.ip = req.ip.clone();  

    // DB 업데이트
    device.update(db).await?;

    Ok(Status::Ok)
}

#[delete("/device/<id>")]
pub async fn delete_device(db: &Db, _auth: AuthUser, id: i64) -> Result<Status, SystemError> {
    Device::delete(&db.0, id).await?;
    Ok(Status::Ok)
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct MoveDeviceReq {
    pub id: i64,
    pub direction: String, // "up" or "down"
}

#[put("/device/move", data = "<req>")]
pub async fn move_device(db: &Db, _auth: AuthUser,req: Json<MoveDeviceReq>,
) -> Result<Status, SystemError> {
    let direction = match req.direction.to_lowercase().as_str() {
        "up" => MoveDirection::Up,
        "down" => MoveDirection::Down,
        _ => return Err(SystemError::APIError(400, 0, "Invalid direction. Use 'up' or 'down'".to_string())),
    };

    Device::move_order(db, req.id, direction).await?;
    Ok(Status::Ok)
}

#[post("/device/wake/<id>")]
pub async fn wake_device(db: &Db, wol_config: &State<WolConfig>, _auth: AuthUser, id: i64)
                         -> Result<Status, SystemError> {
    let device = Device::get(db, id).await?
        .ok_or(PredefinedApiError::NotFound.get())?;

    send_magic_packet(&device.mac, &wol_config.broadcast_ip)?;
    Ok(Status::Ok)
}

/// 로그인한 사용자의 장치 종료 요청을 연결된 에이전트에 전달합니다.
#[post("/device/shutdown/<id>")]
pub async fn shutdown_device(db: &Db, agents: &State<AgentConnections>, _auth: AuthUser, id: i64)
    -> Result<Status, SystemError> {
    Device::get(db, id).await?.ok_or(PredefinedApiError::NotFound.get())?;
    agents.shutdown(id).await?;
    Ok(Status::Ok)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rocket::async_test]
    async fn device_queries_reflect_agent_connection_lifecycle() {
        let db = Db(sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap());
        crate::db::migration::migrate(&db.0).await.unwrap();
        sqlx::query("INSERT INTO device (name, mac, ip, order_num, authkey) VALUES ('test', '', '', 1, ?)")
            .bind(generate_authkey()).execute(&db.0).await.unwrap();
        let agents = AgentConnections::default();
        let state = State::from(&agents);
        let user = || AuthUser { id: 1, user_name: "test".into() };
        assert!(!get_devices(&db, state, user()).await.unwrap().0[0].connected);
        let (registration, receiver) = agents.register(1);
        assert!(get_devices(&db, state, user()).await.unwrap().0[0].connected);
        assert!(get_device(&db, state, user(), 1).await.unwrap().connected);
        drop(receiver);
        assert!(!get_devices(&db, state, user()).await.unwrap().0[0].connected);
        drop(registration);
        assert!(!get_device(&db, state, user(), 1).await.unwrap().connected);
    }

    #[rocket::async_test]
    async fn shutdown_reports_offline_agent() {
        let db = Db(sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap());
        crate::db::migration::migrate(&db.0).await.unwrap();
        sqlx::query("INSERT INTO device (name, mac, ip, order_num, authkey) VALUES ('test', '', '', 1, ?)")
            .bind(generate_authkey()).execute(&db.0).await.unwrap();
        let agents = AgentConnections::default();
        let user = AuthUser { id: 1, user_name: "test".into() };
        let error = shutdown_device(&db, State::from(&agents), user, 1).await.unwrap_err();
        assert!(matches!(error, SystemError::APIError(503, _, message)
            if message.contains("연결되어 있지 않습니다")));
    }
}
