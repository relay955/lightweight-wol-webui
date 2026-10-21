use sqlx::SqlitePool;
use crate::{db::device::generate_authkey, error::SystemError};

const CURRENT_VERSION: i64 = 2;

/// 신규·기존 DB에 버전 순서대로 변경을 적용합니다.
pub async fn migrate(pool: &SqlitePool) -> Result<(), SystemError> {
    let mut transaction = pool.begin().await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS version (
        id INTEGER PRIMARY KEY CHECK (id = 1), number INTEGER NOT NULL
    )").execute(&mut *transaction).await?;
    sqlx::query("INSERT OR IGNORE INTO version (id, number) VALUES (1, 0)")
        .execute(&mut *transaction).await?;
    let version: i64 = sqlx::query_scalar("SELECT number FROM version WHERE id = 1")
        .fetch_one(&mut *transaction).await?;
    if version > CURRENT_VERSION {
        return Err(SystemError::APIError(500, 0, "Unsupported database version".into()));
    }

    if version < 1 { migrate_v1(&mut transaction).await?; }
    if version < 2 { migrate_v2(&mut transaction).await?; }
    sqlx::query("UPDATE version SET number = ? WHERE id = 1")
        .bind(CURRENT_VERSION).execute(&mut *transaction).await?;
    transaction.commit().await?;
    Ok(())
}

/// 버전 1의 초기 테이블을 생성합니다.
async fn migrate_v1(connection: &mut sqlx::SqliteConnection) -> Result<(), SystemError> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS user (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_name VARCHAR(20) NOT NULL,
            password VARCHAR(255) NOT NULL
        )"#,
    )
    .execute(&mut *connection)
    .await?;

    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS token (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            refresh_token VARCHAR(20) NOT NULL,
            expire_at datetime NOT NULL
        )"#,
    )
        .execute(&mut *connection)
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
        .execute(&mut *connection)
        .await?;

    Ok(())
}

/// 버전 2의 장치 인증 키를 추가하고 기존 장치에도 키를 발급합니다.
async fn migrate_v2(connection: &mut sqlx::SqliteConnection) -> Result<(), SystemError> {
    sqlx::query("ALTER TABLE device ADD COLUMN authkey TEXT NOT NULL DEFAULT ''")
        .execute(&mut *connection).await?;
    let ids: Vec<i64> = sqlx::query_scalar("SELECT id FROM device")
        .fetch_all(&mut *connection).await?;
    for id in ids {
        sqlx::query("UPDATE device SET authkey = ? WHERE id = ?")
            .bind(generate_authkey()).bind(id).execute(&mut *connection).await?;
    }
    sqlx::query("CREATE UNIQUE INDEX device_authkey ON device(authkey)")
        .execute(&mut *connection).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::device::{Device, DeviceOperations};

    #[rocket::async_test]
    async fn migrates_legacy_devices_and_preserves_keys_on_rerun() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let mut connection = pool.acquire().await.unwrap();
        migrate_v1(&mut connection).await.unwrap();
        drop(connection);
        sqlx::query("INSERT INTO device (name, mac, ip, order_num)
            VALUES ('first', 'AA:BB:CC:DD:EE:FF', '', 1), ('second', '00:11:22:33:44:55', '', 2)")
            .execute(&pool).await.unwrap();
        migrate(&pool).await.unwrap();
        let before = Device::get_all(&pool).await.unwrap();
        assert_eq!(before.len(), 2);
        assert_eq!(before[0].authkey.len(), 64);
        assert_ne!(before[0].authkey, before[1].authkey);
        migrate(&pool).await.unwrap();
        let after = Device::get_all(&pool).await.unwrap();
        assert_eq!(before[0].authkey, after[0].authkey);
        let version: i64 = sqlx::query_scalar("SELECT number FROM version")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(version, CURRENT_VERSION);
    }

    #[rocket::async_test]
    async fn initializes_fresh_database() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        migrate(&pool).await.unwrap();
        let device = Device {
            id: 0, name: "test".into(), mac: "AA:BB:CC:DD:EE:FF".into(),
            ip: String::new(), order_num: 1, authkey: generate_authkey(),
        };
        let saved = Device::insert(&pool, &device).await.unwrap();
        assert_eq!(saved.authkey, device.authkey);
        sqlx::query("UPDATE device SET name = 'updated' WHERE id = ?")
            .bind(saved.id).execute(&pool).await.unwrap();
        let updated = Device::get(&pool, saved.id).await.unwrap().unwrap();
        updated.update(&pool).await.unwrap();
        assert_eq!(updated.authkey, saved.authkey);
    }
}
