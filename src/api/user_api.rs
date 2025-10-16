
use rocket::serde::{json::Json,Deserialize, Serialize};
use rocket::State;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rocket::http::{Cookie, CookieJar, Status};
use crate::config::JwtConfig;
use crate::module::auth::{bulid_cookie, AuthUser};
use crate::db::{Db, user::{User, UserOperations}};
use crate::db::token::{Token, TokenOperations};
use crate::error::{PredefinedApiError, SystemError};
use crate::module::jwt::{create_jwt, verify_jwt};

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CheckFirstUserRes {
    pub has_user: bool,
}

#[get("/check-first-user")]
pub async fn check_first_user(db: &Db) -> Result<Json<CheckFirstUserRes>, SystemError> {
    let has_user = User::has_any_user(db).await?;
    Ok(Json(CheckFirstUserRes { has_user }))
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct JoinReq {
    pub user_name: String,
    pub password: String,
}

#[post("/join", data = "<request>")]
pub async fn join(db: &Db, request: Json<JoinReq>) -> Result<Status, SystemError> {
    // 이미 계정이 존재하는지 확인
    if User::has_any_user(db).await? {
        return Err(SystemError::APIError(403, 0, "Account already exists. Cannot create more accounts.".to_string()));
    }

    // username 중복 확인
    if User::get_by_user_name(db, &request.user_name).await?.is_some() {
        return Err(PredefinedApiError::Duplicated.get());
    }

    // 비밀번호 해싱
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2.hash_password(request.password.as_bytes(), &salt)?;

    // DB에 사용자 저장
    User::insert(db, &request.user_name, &password_hash.to_string()).await?;
    Ok(Status::Ok)
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct LoginReq {
    pub user_name: String,
    pub password: String,
}

#[post("/login", data = "<request>")]
pub async fn login(db: &Db, cookies: &CookieJar<'_>, jwt_config: &State<JwtConfig>, request: Json<LoginReq>)
    -> Result<Status, SystemError> {
    // 사용자 조회
    let user = User::get_by_user_name(db, &request.user_name)
        .await?.ok_or(PredefinedApiError::NotFound.get())?;

    // 비밀번호 검증
    let argon2 = Argon2::default();
    let parsed_hash = argon2::PasswordHash::new(&user.password)?;

    argon2.verify_password(request.password.as_bytes(), &parsed_hash)
        .map_err(|_| SystemError::APIError(422, 0, "Username or password does not match".to_string()))?;

    // JWT 토큰 생성
    let access_token = create_jwt(&user, &jwt_config)?;

    //기존 해당 유저의 refresh토큰이 있었다면, 해당 토큰을 모두 제거
    Token::delete_by_user_id(db, user.id).await?;

    // refresh토큰 생성
    let refresh_token = crate::module::auth::generate_refresh_token(&user);
    Token::insert(db, &refresh_token).await?;

    // 쿠키에 토큰 저장
    cookies.add(bulid_cookie(&jwt_config,"accessToken", &access_token));
    cookies.add(bulid_cookie(&jwt_config,"refreshToken", &refresh_token.refresh_token));

    Ok(Status::Ok)
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ProfileRes {
    pub user_name: String,
}

#[get("/profile")]
pub async fn profile(db: &Db, auth:AuthUser) -> Result<Json<ProfileRes>, SystemError> {
    // 사용자 조회
    let user = User::get_by_user_name(&db.0, auth.user_name.as_str())
        .await?.ok_or(PredefinedApiError::NotFound.get())?;

    Ok(Json(ProfileRes {
        user_name: user.user_name,
    }))
}
