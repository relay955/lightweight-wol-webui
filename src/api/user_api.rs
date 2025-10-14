
use rocket::serde::{json::Json,Deserialize, Serialize};
use rocket::State;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use rocket::http::Status;
use crate::db::{Db, user::{User, UserOperations}};
use crate::error::{PredefinedApiError, SystemError};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct JoinReq {
    pub user_name: String,
    pub password: String,
}

#[post("/join", data = "<request>")]
pub async fn join(db: &Db, request: Json<JoinReq>) -> Result<Status, SystemError> {
    // 비밀번호 해싱
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2.hash_password(request.password.as_bytes(), &salt)?;

    // DB에 사용자 저장
    User::insert(&db.0, &request.user_name, &password_hash.to_string()).await?;
    Ok(Status::Ok)
}
