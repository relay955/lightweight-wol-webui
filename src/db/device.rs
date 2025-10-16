use rocket::serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Device {
    pub id: i64,
    pub name: String,
    pub mac: String,
    pub order_num: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveDirection { Up, Down }

#[async_trait::async_trait]
pub trait DeviceOperations {
    async fn get_all(pool: &SqlitePool) -> Result<Vec<Device>, sqlx::Error>;
    async fn get(pool: &SqlitePool, id: i64) -> Result<Option<Device>, sqlx::Error>;
    async fn get_max_order_num(pool: &SqlitePool) -> Result<i64, sqlx::Error>;
    async fn insert(pool: &SqlitePool, device: &Device) -> Result<Device, sqlx::Error>;
    async fn update(&self, pool: &SqlitePool) -> Result<(), sqlx::Error>;
    async fn delete(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error>;
    async fn move_order(pool: &SqlitePool, id: i64, direction: MoveDirection) -> Result<bool, sqlx::Error>;
}

#[async_trait::async_trait]
impl DeviceOperations for Device {
    async fn get_all(pool: &SqlitePool) -> Result<Vec<Device>, sqlx::Error> {
        
        sqlx::query_as::<_, Device>("SELECT id, name, mac, order_num FROM device ORDER BY order_num")
            .fetch_all(pool)
            .await
    }

    async fn get(pool: &SqlitePool, id: i64) -> Result<Option<Device>, sqlx::Error> {
        sqlx::query_as::<_, Device>("SELECT id, name, mac, order_num FROM device WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    async fn get_max_order_num(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
        let result: Option<(Option<i64>,)> = sqlx::query_as("SELECT MAX(order_num) FROM device")
            .fetch_optional(pool)
            .await?;

        Ok(result.and_then(|r| r.0).unwrap_or(0))
    }

    async fn insert(pool: &SqlitePool, device: &Device) -> Result<Device, sqlx::Error> {
        let result = sqlx::query("INSERT INTO device (name, mac, order_num) VALUES (?, ?, ?)")
            .bind(&device.name)
            .bind(&device.mac)
            .bind(device.order_num)
            .execute(pool)
            .await?;

        let id = result.last_insert_rowid();

        Ok(Device {
            id,
            name: device.name.clone(),
            mac: device.mac.clone(),
            order_num: device.order_num,
        })
    }

    async fn update(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE device SET name = ?, mac = ?, order_num = ? WHERE id = ?")
            .bind(&self.name)
            .bind(&self.mac)
            .bind(self.order_num)
            .bind(self.id)
            .execute(pool)
            .await?;

        Ok(())
    }

    async fn delete(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM device WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    async fn move_order(pool: &SqlitePool, id: i64, direction: MoveDirection) -> Result<bool, sqlx::Error> {
        // 현재 항목 조회
        let current = match Self::get(pool, id).await? {
            Some(device) => device,
            None => return Ok(false),
        };

        // 방향에 따라 SQL 쿼리 선택
        let query = match direction {
            MoveDirection::Up => "SELECT id, name, mac, order_num FROM device WHERE order_num < ? ORDER BY order_num DESC LIMIT 1",
            MoveDirection::Down => "SELECT id, name, mac, order_num FROM device WHERE order_num > ? ORDER BY order_num ASC LIMIT 1",
        };

        // 인접한 항목 조회
        let adjacent: Option<Device> = sqlx::query_as::<_, Device>(query)
            .bind(current.order_num)
            .fetch_optional(pool)
            .await?;

        if let Some(adjacent) = adjacent {
            // order_num 교환
            let temp_order = current.order_num;

            sqlx::query("UPDATE device SET order_num = ? WHERE id = ?")
                .bind(adjacent.order_num)
                .bind(current.id)
                .execute(pool)
                .await?;

            sqlx::query("UPDATE device SET order_num = ? WHERE id = ?")
                .bind(temp_order)
                .bind(adjacent.id)
                .execute(pool)
                .await?;

            Ok(true)
        } else {
            Ok(false)
        }
    }
}
