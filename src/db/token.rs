use rocket::serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use crate::db::is_exist_tables;
use crate::db::user::User;

#[derive(Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Token{
    pub id:i64,
    pub refresh_token:String,
    pub expire_at:String,
}


#[async_trait::async_trait]
pub trait TokenOperations {
    async fn insert(pool: &SqlitePool, token: &Token) -> Result<(), sqlx::Error>;
    async fn get_by_refresh_token(pool: &SqlitePool, refresh_token: String) -> Result<Option<Token>, sqlx::Error>;
}
#[async_trait::async_trait]
impl TokenOperations for Token {
    async fn insert(pool: &SqlitePool, token: &Token) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO token (id, refresh_token, expire_at) VALUES (?, ?, ?)")
            .bind(token.id)
            .bind(token.refresh_token.clone())
            .bind(token.expire_at.clone())
            .execute(pool)
            .await?;
        Ok(())
    }
    
    async fn get_by_refresh_token(pool: &SqlitePool, refresh_token: String) -> Result<Option<Token>, sqlx::Error> {
        sqlx::query_as::<_, Token>("SELECT * FROM token WHERE refresh_token = ?")
            .bind(refresh_token)
            .fetch_optional(pool)
            .await
    }
}