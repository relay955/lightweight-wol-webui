use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct JwtConfig {
    pub secret: String,
    pub expiration_minutes: i64,
    pub refresh_token_expiration_days: i64,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: "b3f5c7a1d9e24f6b8c1a2d3e4f5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3".to_string(),
            expiration_minutes: 30,
            refresh_token_expiration_days: 14,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct CorsConfig {
    pub allow_origin: String,
    pub allow_credentials: bool,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allow_origin: "*".to_string(),
            allow_credentials: true,
        }
    }
}



