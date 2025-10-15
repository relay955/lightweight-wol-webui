use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rocket::serde::{Deserialize, Serialize};
use crate::db::user::User;
use crate::error::SystemError;

const JWT_SECRET: &[u8] = b"b3f5c7a1d9e24f6b8c1a2d3e4f5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3 ";
const EXPIRATION_MINUTES: i64 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64, // user_name
    pub exp: usize,  // expiration time
    pub user_name: String,
}

pub fn create_jwt(user: &User) -> Result<String, SystemError> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::minutes(EXPIRATION_MINUTES))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user.id,
        user_name: user.user_name.clone(),
        exp: expiration,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(JWT_SECRET))
        .map_err(|e| SystemError::APIError(500, 0, format!("JWT 생성 실패: {}", e)))
}

pub fn verify_jwt(token: &str) -> Result<Claims, SystemError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| SystemError::APIError(401, 0, format!("유효하지 않은 토큰: {}", e)))
}
