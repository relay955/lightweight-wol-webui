pub mod user;
pub(crate) mod token;
pub mod device;
pub mod migration;

use rocket_db_pools::Database;

#[derive(Database)]
#[database("sqlite")]
pub struct Db(pub sqlx::SqlitePool);
