use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::http::Status;
use crate::auth::AuthUser;
use crate::db::Db;
use crate::db::device::{Device, DeviceOperations, MoveDirection};
use crate::error::{PredefinedApiError, SystemError};

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct GetDeviceRes {
    pub devices: Vec<Device>,
}

#[get("/devices")]
pub async fn get_devices(db: &Db, _auth: AuthUser) -> Result<Json<GetDeviceRes>, SystemError> {
    let devices = Device::get_all(&db.0).await?;
    Ok(Json(GetDeviceRes { devices }))
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CreateDeviceReq {
    pub name: String,
    pub mac: String,
    pub order_num: i64,
}

#[post("/devices", data = "<request>")]
pub async fn create_device(db: &Db, _auth: AuthUser, request: Json<CreateDeviceReq>,
) -> Result<Status, SystemError> {
    let device = Device{
        id: 0,
        name: request.name.clone(),
        mac: request.mac.clone(),
        order_num: request.order_num,
    };
    Device::insert(db,&device).await?;
    Ok(Status::Ok)
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct UpdateDeviceReq {
    pub name: String,
    pub mac: String,
    pub order_num: i64,
}

#[put("/devices/<id>", data = "<request>")]
pub async fn update_device(db: &Db, _auth: AuthUser, id: i64, request: Json<UpdateDeviceReq>) 
    -> Result<Status, SystemError> {
    let mut device = Device::get(db, id).await?
        .ok_or(PredefinedApiError::NotFound.get())?;

    // 필드 업데이트
    device.name = request.name.clone();
    device.mac = request.mac.clone();
    device.order_num = request.order_num;

    // DB 업데이트
    device.update(db).await?;

    Ok(Status::Ok)
}

#[delete("/devices/<id>")]
pub async fn delete_device(db: &Db, _auth: AuthUser, id: i64) -> Result<Status, SystemError> {
    Device::delete(&db.0, id).await?;
    Ok(Status::Ok)
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct MoveDeviceReq {
    pub direction: String, // "up" or "down"
}

#[post("/devices/<id>/move", data = "<request>")]
pub async fn move_device(db: &Db, _auth: AuthUser, id: i64, request: Json<MoveDeviceReq>,
) -> Result<Status, SystemError> {
    // 디바이스 존재 확인
    Device::get(db, id)
        .await?
        .ok_or(PredefinedApiError::NotFound.get())?;

    // 방향 파싱
    let direction = match request.direction.to_lowercase().as_str() {
        "up" => MoveDirection::Up,
        "down" => MoveDirection::Down,
        _ => return Err(SystemError::APIError(400, 0, "Invalid direction. Use 'up' or 'down'".to_string())),
    };

    // 순서 변경
    Device::move_order(db, id, direction).await?;
    Ok(Status::Ok)
}