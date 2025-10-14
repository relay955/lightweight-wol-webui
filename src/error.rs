use std::num::ParseIntError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("invalid header")]
    InvalidHeaderError,
    #[error("skip")]
    Skip,
    #[error("sql")]
    Sql(#[from] sqlx::Error),
    #[error("rocket")]
    Rocket(#[from] rocket::Error),
}

impl serde::Serialize for ApiError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}