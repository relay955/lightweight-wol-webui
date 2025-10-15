use rocket::response::{status, Responder};
use rocket::serde::json::Json;
use rocket::yansi::Paint;
use thiserror::Error;

#[derive(serde::Serialize)]
pub struct ApiErrorFormat {
    pub code: u16,
    pub msg: String
}

impl ApiErrorFormat {
    pub fn new(code:u16, message: impl Into<String>) -> Self {
        Self { code, msg: message.into() }
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
        let mut http_status_code:u16 = 500;
        let json = match self {
            SystemError::APIError(http_status, code, message) => {
                http_status_code = http_status;
                Json(ApiErrorFormat::new(code, message))
            }
            _ => Json(ApiErrorFormat::new(0, SystemError::to_string(&self)))
        };
        status::Custom(rocket::http::Status::from_code(http_status_code).unwrap(), json).respond_to(req)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PredefinedApiError {
    Forbidden,
    NotFound,
    BadRequest,
    InternalServerError,
    Duplicated,
    TokenExpired,
}

impl PredefinedApiError {
    pub fn get(self) -> SystemError {
        match self {
            PredefinedApiError::Forbidden => SystemError::APIError(403, 1, "권한이 없습니다.".to_string()),
            PredefinedApiError::NotFound => SystemError::APIError(422, 2, "해당 항목이 없습니다.".to_string()),
            PredefinedApiError::BadRequest => SystemError::APIError(400, 3, "잘못된 요청입니다".to_string()),
            PredefinedApiError::InternalServerError => SystemError::APIError(500, 4, "서버 내부 오류".to_string()),
            PredefinedApiError::Duplicated => SystemError::APIError(422, 5, "해당 항목이 이미 존재합니다.".to_string()),
            PredefinedApiError::TokenExpired => SystemError::APIError(422, 6, "토큰이 만료되었습니다.".to_string()),
        }
    }
}