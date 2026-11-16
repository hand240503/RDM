use anyhow::Result;
use chrono::{Duration, SecondsFormat, Utc};
use sqlx::{Row, SqlitePool, sqlite::SqlitePoolOptions};

use crate::model::{BatteryHistoryRecord, BatteryInfo, ThermalHistoryRecord, ThermalSensor};

#[derive(Clone)]
pub struct Storage {
    pool: SqlitePool,
    retention_days: i64,
}

impl Storage {
    pub async fn open(path: &str, retention_days: i64) -> Result<Self> {
        let url = if path.starts_with("sqlite:") {
            path.to_owned()
        } else {
            format!("sqlite://{path}")
        };
        let options = sqlx::sqlite::SqliteConnectOptions::from_str(&url)?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .foreign_keys(true);
        let max_connections = if path.contains(":memory:") { 1 } else { 4 };
        let pool = SqlitePoolOptions::new()
            .max_connections(max_connections)
            .connect_with(options)
            .await?;
        sqlx::query("CREATE TABLE IF NOT EXISTS battery_history (timestamp TEXT PRIMARY KEY, level INTEGER, temperature_celsius REAL)")
            .execute(&pool).await?;
        sqlx::query("CREATE TABLE IF NOT EXISTS thermal_history (sensor_name TEXT NOT NULL, timestamp TEXT NOT NULL, value_celsius REAL NOT NULL, PRIMARY KEY(sensor_name, timestamp))")
            .execute(&pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_thermal_history_timestamp ON thermal_history(timestamp)")
            .execute(&pool).await?;
        Ok(Self {
            pool,
            retention_days: retention_days.max(1),
        })
    }

    pub async fn record(&self, battery: &BatteryInfo, thermal: &[ThermalSensor]) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("INSERT OR IGNORE INTO battery_history(timestamp, level, temperature_celsius) VALUES(?, ?, ?)")
            .bind(&battery.timestamp)
            .bind(battery.level)
            .bind(battery.temperature_celsius)
            .execute(&mut *tx).await?;
        for sensor in thermal.iter().filter(|sensor| sensor.available) {
            if let Some(value) = sensor.value_celsius {
                sqlx::query("INSERT OR IGNORE INTO thermal_history(sensor_name, timestamp, value_celsius) VALUES(?, ?, ?)")
                    .bind(&sensor.name).bind(&battery.timestamp).bind(value).execute(&mut *tx).await?;
            }
        }
        let cutoff = (Utc::now() - Duration::days(self.retention_days))
            .to_rfc3339_opts(SecondsFormat::Millis, true);
        sqlx::query("DELETE FROM battery_history WHERE timestamp < ?")
            .bind(&cutoff)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM thermal_history WHERE timestamp < ?")
            .bind(cutoff)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn battery_history(&self, limit: i64) -> Result<Vec<BatteryHistoryRecord>> {
        let rows = sqlx::query("SELECT level, temperature_celsius, timestamp FROM (SELECT level, temperature_celsius, timestamp FROM battery_history ORDER BY timestamp DESC LIMIT ?) ORDER BY timestamp ASC")
            .bind(limit.clamp(1, 500)).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|row| {
                Ok(BatteryHistoryRecord {
                    level: row.try_get("level")?,
                    temperature_celsius: row.try_get("temperature_celsius")?,
                    timestamp: row.try_get("timestamp")?,
                })
            })
            .collect()
    }

    pub async fn thermal_history(&self, limit: i64) -> Result<Vec<ThermalHistoryRecord>> {
        let rows = sqlx::query("SELECT sensor_name, value_celsius, timestamp FROM (SELECT sensor_name, value_celsius, timestamp FROM thermal_history ORDER BY timestamp DESC LIMIT ?) ORDER BY timestamp ASC")
            .bind(limit.clamp(1, 500)).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|row| {
                Ok(ThermalHistoryRecord {
                    sensor_name: row.try_get("sensor_name")?,
                    value_celsius: row.try_get("value_celsius")?,
                    timestamp: row.try_get("timestamp")?,
                })
            })
            .collect()
    }
}

use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn persists_and_limits_history() {
        let storage = Storage::open("sqlite::memory:", 30).await.unwrap();
        let battery = BatteryInfo {
            level: Some(76),
            charging_status: "discharging".into(),
            temperature_celsius: Some(33.4),
            voltage_mv: None,
            current_ma: None,
            health: None,
            technology: None,
            timestamp: "2026-11-16T00:00:00.000Z".into(),
        };
        let thermal = vec![ThermalSensor {
            name: "battery".into(),
            sensor_type: "unknown".into(),
            value_celsius: Some(33.4),
            available: true,
            error: None,
        }];
        storage.record(&battery, &thermal).await.unwrap();
        storage.record(&battery, &thermal).await.unwrap();
        assert_eq!(storage.battery_history(60).await.unwrap().len(), 1);
        let records = storage.thermal_history(60).await.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].sensor_name, "battery");
    }
}
