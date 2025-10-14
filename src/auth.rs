use crate::db::user::UserOperations;
use crate::error::SystemError;
use crate::jwt::verify_jwt;
use rocket::http::Status;
use rocket::outcome::Outcome::{Error, Success};
use rocket::request::{FromRequest, Outcome, Request};

pub struct AuthUser {
    pub id: i64,
    pub user_name: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = SystemError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, SystemError> {
        let token = match request.cookies().get("accessToken").map(|c| c.value().to_string()){
            Some(t) => t,
            None => return Error((Status::Unauthorized,SystemError::APIError(401, 0, "로그인이 필요합니다".to_string())))
        };
        
        let claims = match verify_jwt(&token) {
            Ok(c) => c,
            Err(e) => return Error((Status::Unauthorized,SystemError::APIError(401, 0, "토큰이 만료되었습니다.".to_string()))),
        };
        
        Success(AuthUser { id:claims.sub, user_name: claims.user_name })
    }
}