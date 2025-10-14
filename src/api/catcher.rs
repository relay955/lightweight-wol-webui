use rocket::Request;
use rocket::serde::json::Json;
use crate::error::{ApiErrorFormat, SystemError};

#[catch(401)]
pub fn unauthorized(req: &Request) -> Json<ApiErrorFormat> {
    if let Some(error) = req.local_cache(|| None::<SystemError>) {
        if let SystemError::APIError(_, code, message) = error {
            return Json(ApiErrorFormat::new(*code, message.clone()));
        }
    }
    Json(ApiErrorFormat::new(0, "로그인이 필요합니다"))
}