use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::http::Status;
use serde_json::json;
use serde_json::ser::CharEscape::CarriageReturn;
use lazy_static::lazy_static;
use validator::Validate;
use std::net::UdpSocket;
use crate::api::validate_request;
use crate::module::auth::AuthUser;
use crate::db::Db;
use crate::db::device::{Device, DeviceOperations, MoveDirection};
use crate::error::{PredefinedApiError, SystemError};
use crate::module::magic_packet::send_magic_packet;

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
    pub order_num: i64,
}

#[get("/devices")]
pub async fn get_devices(db: &Db, _auth: AuthUser) -> Result<Json<Vec<GetDeviceRes>>, SystemError> {
    let devices = Device::get_all(db).await?;
    let mut device_list = Vec::new();
    for device in devices {
        device_list.push(GetDeviceRes {
            id: device.id,
            name: device.name,
            mac: device.mac,
            order_num: device.order_num,
        });
    }
    Ok(Json(device_list))
}

#[get("/device/<id>")]
pub async fn get_device(db: &Db, _auth: AuthUser, id: i64) -> Result<Json<GetDeviceRes>, SystemError> {
    let device = Device::get(db, id).await?
        .ok_or(PredefinedApiError::NotFound.get())?;

    let device_res = GetDeviceRes {
        id: device.id,
        name: device.name,
        mac: device.mac,
        order_num: device.order_num,
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
        order_num: max_order + 1,
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
pub async fn wake_device(db: &Db, _auth: AuthUser, id: i64)
                         -> Result<Status, SystemError> {
    let device = Device::get(db, id).await?
        .ok_or(PredefinedApiError::NotFound.get())?;
    
    send_magic_packet(&device.mac)?;
    Ok(Status::Ok)
}
