use crate::db::token::{Token, TokenOperations};
use crate::db::user::{User, UserOperations};
use crate::db::Db;
use crate::error::SystemError;
use crate::module::jwt::{create_jwt, verify_jwt};
use chrono::{Duration, Utc};
use rocket::http::{Cookie, Status};
use rocket::outcome::Outcome::{Error, Success};
use rocket::request::{FromRequest, Outcome, Request};
use rocket::State;
use uuid::Uuid;
use crate::config::JwtConfig;

pub struct AuthUser {
    pub id: i64,
    pub user_name: String,
}

pub struct TokenCookies {
    pub access_token: String,
    pub refresh_token: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = SystemError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, SystemError> {
        let db = match request.guard::<&Db>().await {
            Success(db) => db,
            _ => return create_outcome_error(Status::InternalServerError,"Database connection failed"),       
        };

        let jwt_config = match request.rocket().state::<JwtConfig>() {
            Some(config) => config,
            None => return create_outcome_error(Status::InternalServerError,"JWT config not found"),       
        };

        let access_token = request.cookies().get("accessToken")
            .map(|c| c.value().to_string());
        let refresh_token = request.cookies().get("refreshToken")
            .map(|c| c.value().to_string());

        if access_token.is_none() {
            return create_outcome_error(Status::Unauthorized,"Login required");
        }

        let token = access_token.unwrap();

        match verify_jwt(&token, &jwt_config) {
            Ok(claims) => Success(AuthUser {
                id: claims.sub,
                user_name: claims.user_name,
            }),
            Err(_) => {
                // accessToken 만료시 refreshToken으로 재발급 시도
                if let Some(refresh_token_value) = refresh_token {
                    match do_refresh(db, request,jwt_config, &refresh_token_value).await {
                        Ok(auth_user) => Success(auth_user),
                        Err(e) => Error((Status::Unauthorized, e)),
                    }
                } else {
                    create_outcome_error(Status::Unauthorized,"Token expired")
                }
            }
        }
    }
}

async fn do_refresh(db: &Db, request: &Request<'_>,jwt_config: &JwtConfig, refresh_token: &str) 
    -> Result<AuthUser, SystemError> {

    println!("Executing refresh");
    // refreshToken으로 DB에서 토큰 조회
    let mut token = match Token::get_by_refresh_token(db, refresh_token.to_string()).await {
        Ok(Some(token)) => token,
        Ok(None) => {
            return Err(SystemError::APIError(401, 0, "Invalid refresh token".to_string()));
        }
        Err(e) => {
            return Err(SystemError::APIError(500, 0, "Failed to query token".to_string()));
        }
    };

    // 유저 조회
    let user = match User::get(db, token.user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err(SystemError::APIError(401, 0, "User not found".to_string()));
        }
        Err(_) => {
            return Err(SystemError::APIError(500, 0, "Failed to query user".to_string()));
        }
    };

    // 토큰 만료 검사
    let expire_at = chrono::NaiveDateTime::parse_from_str(&token.expire_at, "%Y-%m-%d %H:%M:%S")
        .map_err(|_| SystemError::APIError(500, 0, "Failed to parse date".to_string()))?;
    let expire_at = chrono::DateTime::<Utc>::from_naive_utc_and_offset(expire_at, Utc);
    let now = Utc::now();

    if now > expire_at {
        // refreshToken도 만료됨
        Token::delete(db, token.id).await.ok();
        return Err(SystemError::APIError(401, 0, "Refresh token expired".to_string()));
    }

    // refreshToken이 만료 7일 이전일 경우 refreshToken 갱신
    if now > expire_at - Duration::days(7) {
        let new_refresh_token = generate_refresh_token(&user);
        token.refresh_token = new_refresh_token.refresh_token;
        token.expire_at = new_refresh_token.expire_at;

        Token::update(db, &token).await
            .map_err(|_| SystemError::APIError(500, 0, "Failed to renew token".to_string()))?;
    }

    // 새로운 accessToken 생성 (설정에서 가져온 만료 시간 사용)
    let new_access_token = create_jwt(&user, &jwt_config)
        .map_err(|_| SystemError::APIError(500, 0, "Failed to create JWT".to_string()))?;

    // 쿠키 설정 (응답에 쿠키 추가)
    request.cookies().add(Cookie::build(("accessToken", new_access_token.clone())));
    request.cookies().add(Cookie::build(("refreshToken", token.refresh_token.clone())));

    Ok(AuthUser {
        id: user.id,
        user_name: user.user_name,
    })
}

pub fn generate_refresh_token(user: &User) -> Token {
    Token {
        id: 0,
        user_id: user.id,
        refresh_token: format!("{}{}", Uuid::new_v4(), Uuid::new_v4()),
        expire_at: (Utc::now() + Duration::days(14)).format("%Y-%m-%d %H:%M:%S").to_string(),
    }
}
fn create_outcome_error(status:Status,str:&str) -> Outcome<AuthUser, SystemError> {
    Error((status, SystemError::APIError(status.code, 0, str.to_string())))
}

