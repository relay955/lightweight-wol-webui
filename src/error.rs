use std::num::ParseIntError;
use rocket::response::{status, Responder};
use rocket::serde::json::Json;
use rocket::yansi::Paint;
use serde_json::json;
use thiserror::Error;

#[derive(serde::Serialize)]
pub struct ApiErrorFormat {
    http_status: u16,
    code: u16,
    message: String
}

impl ApiErrorFormat {
    pub fn new(http_status:u16, code:u16, message: impl Into<String>) -> Self {
        Self { http_status: http_status, code, message: message.into() }
    }
}

#[derive(Debug, Error)]
pub enum SystemError {
    #[error("invalid header")]
    InvalidHeaderError,
    #[error("skip")]
    Skip,
    #[error("sql")]
    Sql(#[from] sqlx::Error),
    #[error("rocket")]
    Rocket(#[from] rocket::Error),
    #[error("api")]
    APIError(u16,u16,String),
    #[error("argon2")]
    Argon2Error(password_hash::Error),
}

impl From<password_hash::Error> for SystemError {
    fn from(err: password_hash::Error) -> Self {
        SystemError::Argon2Error(err)
    }
}

impl<'r> Responder<'r, 'static> for SystemError {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        let json = match self {
            SystemError::APIError(http_status, code, message) => 
                Json(ApiErrorFormat::new(http_status, code, message)),
            _ => Json(ApiErrorFormat::new(500, 0, SystemError::to_string(&self)))
        };
        status::Custom(rocket::http::Status::from_code(json.code).unwrap(), json).respond_to(req)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PredefinedApiError {
    Unauthorized,
    NotFound,
    BadRequest,
    InternalServerError,
}

impl PredefinedApiError {
    pub fn get(self) -> ApiErrorFormat {
        match self {
            PredefinedApiError::Unauthorized => ApiErrorFormat::new(401, 0, "인증되지 않은 요청입니다"),
            PredefinedApiError::NotFound => ApiErrorFormat::new(404, 0, "요청한 자원을 찾을 수 없습니다"),
            PredefinedApiError::BadRequest => ApiErrorFormat::new(400, 0, "잘못된 요청입니다"),
            PredefinedApiError::InternalServerError => ApiErrorFormat::new(500, 0, "서버 내부 오류"),
        }
    }
}