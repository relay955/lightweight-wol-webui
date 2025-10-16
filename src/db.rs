pub mod user;
pub(crate) mod token;
pub mod device;

use rocket::serde::{Deserialize, Serialize};
use rocket_db_pools::Database;
use crate::error::SystemError;

#[derive(Database)]
#[database("sqlite")]
pub struct Db(pub sqlx::SqlitePool);

pub async fn is_exist_tables(db: &Db) -> Result<bool, SystemError> {
    let row = sqlx::query(
        r#"SELECT name FROM sqlite_master WHERE type='table' AND name='user'"#,
    )
        .fetch_optional(&db.0)
        .await?;
    Ok(row.is_some())
}
pub async fn create_tables(db: &Db) -> Result<(), SystemError> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS user (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_name VARCHAR(20) NOT NULL,
            password VARCHAR(255) NOT NULL
        )"#,
    )
    .execute(&db.0)
    .await?;

    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS token (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            refresh_token VARCHAR(20) NOT NULL,
            expire_at datetime NOT NULL
        )"#,
    )
        .execute(&db.0)
        .await?;

    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS device (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name VARCHAR(255) NOT NULL,
            mac VARCHAR(17) NOT NULL,
            ip VARCHAR(20) NOT NULL,
            order_num INTEGER NOT NULL
        )"#,
    )
        .execute(&db.0)
        .await?;

    Ok(())
}