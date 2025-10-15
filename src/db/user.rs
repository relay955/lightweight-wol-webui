use rocket::serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use crate::db::is_exist_tables;

#[derive(Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct User{
    pub id:i64,
    pub user_name:String,
    pub password:String,
}

#[async_trait::async_trait]
pub trait UserOperations {
    async fn get(pool: &SqlitePool, id: i64) -> Result<Option<User>, sqlx::Error>;
    async fn get_by_user_name(pool: &SqlitePool, user_name: &str) -> Result<Option<User>, sqlx::Error>;
    async fn insert(pool: &SqlitePool, user_name: &str, password: &str) -> Result<User, sqlx::Error>;
    async fn update(&self, pool: &SqlitePool) -> Result<(), sqlx::Error>;
    async fn has_any_user(pool: &SqlitePool) -> Result<bool, sqlx::Error>;
}

#[async_trait::async_trait]
impl UserOperations for User {
    async fn get(pool: &SqlitePool, id: i64) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT id, user_name, password FROM user WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    async fn get_by_user_name(pool: &SqlitePool, user_name: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT id, user_name, password FROM user WHERE user_name = ?")
            .bind(user_name)
            .fetch_optional(pool)
            .await
    }

    async fn insert(pool: &SqlitePool, user_name: &str, password: &str) -> Result<User, sqlx::Error> {
        let result = sqlx::query("INSERT INTO user (user_name, password) VALUES (?, ?)")
            .bind(user_name)
            .bind(password)
            .execute(pool)
            .await?;
        
        let id = result.last_insert_rowid();

        Ok(User {
            id,
            user_name: user_name.to_string(),
            password: password.to_string(),
        })
    }

    async fn update(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE user SET user_name = ?, password = ? WHERE id = ?")
            .bind(&self.user_name)
            .bind(&self.password)
            .bind(self.id)
            .execute(pool)
            .await?;

        Ok(())
    }

    async fn has_any_user(pool: &SqlitePool) -> Result<bool, sqlx::Error> {
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM user")
            .fetch_one(pool)
            .await?;

        Ok(result.0 > 0)
    }
}