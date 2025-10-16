use rocket::Request;
use rocket::serde::json::Json;
use rocket::response::Redirect;
use crate::error::{ApiErrorFormat, SystemError};

#[catch(401)]
pub fn unauthorized(req: &Request) -> Json<ApiErrorFormat> {
    if let Some(error) = req.local_cache(|| None::<SystemError>) {
        if let SystemError::APIError(_, code, message) = error {
            return Json(ApiErrorFormat::new(*code, message.clone()));
        }
    }
    Json(ApiErrorFormat::new(0, "Login required"))
}

#[catch(404)]
pub fn not_found(req: &Request) -> Result<Redirect, Json<ApiErrorFormat>> {
    if req.uri().path().as_str().starts_with("/api") {
        Err(Json(ApiErrorFormat::new(404, "Not found".to_string())))
    } else {
        Ok(Redirect::to("/index.html"))
    }
}