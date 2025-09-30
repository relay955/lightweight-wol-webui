use rocket::serde::{Deserialize, Serialize};
use rocket_db_pools::Database;
use crate::error::ApiError;

#[derive(Database)]
#[database("sqlite")]
struct Db(sqlx::SqlitePool);

// #[derive(Debug, Deserialize)]
// struct SignupReq {
//     email: String,
//     password: String,
// }
// 
// #[derive(Debug, Deserialize)]
// struct LoginReq {
//     email: String,
//     password: String,
// }
// 
// #[derive(Debug, Serialize)]
// struct MeResp {
//     id: i64,
//     email: String,
//     created_at: String,
// }
// 


pub async fn create_tables(db: &Db) -> Result<(), ApiError> {
    println!("create tables");
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS user (
            id INTEGER PRIMARY KEY,
            user_name VARCHAR(20) NOT NULL,
            password VARCHAR(255) NOT NULL
        )"#,
    )
    .execute(&db.0)
    .await?;
    Ok(())
}