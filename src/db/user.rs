use rocket::serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use crate::db::is_exist_tables;

#[async_trait::async_trait]
pub trait UserOperations {
    async fn get(pool: &SqlitePool, id: i64) -> Result<User, sqlx::Error>;
    async fn insert(pool: &SqlitePool, user_name: &str, password: &str) -> Result<User, sqlx::Error>;
    async fn update(&self, pool: &SqlitePool) -> Result<(), sqlx::Error>;
}

#[derive(Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct User{
    pub id:i64,
    pub user_name:String,
    pub password:String,
}


#[async_trait::async_trait]
impl UserOperations for User {
    async fn get(pool: &SqlitePool, id: i64) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT id, user_name, password FROM user WHERE id = ?")
            .bind(id)
            .fetch_one(pool)
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
}