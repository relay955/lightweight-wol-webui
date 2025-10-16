use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rocket::serde::{Deserialize, Serialize};
use crate::config::JwtConfig;
use crate::db::user::User;
use crate::error::SystemError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64, // user_name
    pub exp: usize,  // expiration time
    pub user_name: String,
}

pub fn create_jwt(user: &User, jwt_config: &JwtConfig) -> Result<String, SystemError> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::minutes(jwt_config.expiration_minutes))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user.id,
        user_name: user.user_name.clone(),
        exp: expiration,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(jwt_config.secret.as_bytes()))
        .map_err(|e| SystemError::APIError(500, 0, format!("Failed to create JWT: {}", e)))
}

pub fn verify_jwt(token: &str,  jwt_config: &JwtConfig) -> Result<Claims, SystemError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_config.secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| SystemError::APIError(401, 0, format!("Invalid token: {}", e)))
}

pub fn generate_random_secret() -> String {
    use rand_core::OsRng;
    let mut rng = OsRng;
    let mut bytes = [0u8; 32];
    use rand_core::RngCore;
    rng.fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>()
}