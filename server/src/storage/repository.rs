// Data repository implementations

use simbridge_shared::models::{Device, Session, SessionStreamConfig, StreamQuality};
use sqlx::{Row, SqlitePool};
use thiserror::Error;
use uuid::Uuid;

/// Device repository
pub struct DeviceRepository {
    pool: SqlitePool,
}

impl DeviceRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a device
    pub async fn create(&self, device: &Device) -> Result<(), RepositoryError> {
        sqlx::query(
            "INSERT INTO devices (id, name, device_type, platform, os_version, paired_at, last_seen, is_trusted, public_key) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&device.id)
        .bind(&device.name)
        .bind(format!("{:?}", device.device_type))
        .bind(&device.platform)
        .bind(&device.os_version)
        .bind(device.paired_at)
        .bind(device.last_seen)
        .bind(device.is_trusted)
        .bind(&device.public_key)
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::QueryError(e.to_string()))?;

        Ok(())
    }

    /// Get a device by ID
    pub async fn get_by_id(&self, id: &str) -> Result<Option<Device>, RepositoryError> {
        let row = sqlx::query(
            "SELECT id, name, device_type, platform, os_version, paired_at, last_seen, is_trusted, public_key 
             FROM devices WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::QueryError(e.to_string()))?;

        Ok(row.map(|r| Device {
            id: r.get("id"),
            name: r.get("name"),
            device_type: match r.get::<String, _>("device_type").as_str() {
                "Android" => simbridge_shared::models::DeviceType::Android,
                "Ios" => simbridge_shared::models::DeviceType::Ios,
                "Desktop" => simbridge_shared::models::DeviceType::Desktop,
                _ => simbridge_shared::models::DeviceType::Desktop,
            },
            platform: r.get("platform"),
            os_version: r.get("os_version"),
            paired_at: r.get("paired_at"),
            last_seen: r.get("last_seen"),
            is_trusted: r.get("is_trusted"),
            public_key: r.get("public_key"),
        }))
    }

    /// Get all devices
    pub async fn get_all(&self) -> Result<Vec<Device>, RepositoryError> {
        let rows = sqlx::query(
            "SELECT id, name, device_type, platform, os_version, paired_at, last_seen, is_trusted, public_key 
             FROM devices"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::QueryError(e.to_string()))?;

        Ok(rows
            .iter()
            .map(|r| Device {
                id: r.get("id"),
                name: r.get("name"),
                device_type: match r.get::<String, _>("device_type").as_str() {
                    "Android" => simbridge_shared::models::DeviceType::Android,
                    "Ios" => simbridge_shared::models::DeviceType::Ios,
                    "Desktop" => simbridge_shared::models::DeviceType::Desktop,
                    _ => simbridge_shared::models::DeviceType::Desktop,
                },
                platform: r.get("platform"),
                os_version: r.get("os_version"),
                paired_at: r.get("paired_at"),
                last_seen: r.get("last_seen"),
                is_trusted: r.get("is_trusted"),
                public_key: r.get("public_key"),
            })
            .collect())
    }

    /// Update device
    pub async fn update(&self, device: &Device) -> Result<(), RepositoryError> {
        sqlx::query(
            "UPDATE devices SET name = ?, platform = ?, os_version = ?, last_seen = ?, is_trusted = ? 
             WHERE id = ?"
        )
        .bind(&device.name)
        .bind(&device.platform)
        .bind(&device.os_version)
        .bind(device.last_seen)
        .bind(device.is_trusted)
        .bind(&device.id)
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::QueryError(e.to_string()))?;

        Ok(())
    }

    /// Delete a device
    pub async fn delete(&self, id: &str) -> Result<(), RepositoryError> {
        sqlx::query("DELETE FROM devices WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::QueryError(e.to_string()))?;

        Ok(())
    }
}

/// Session repository
pub struct SessionRepository {
    pool: SqlitePool,
}

impl SessionRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a session
    pub async fn create(&self, session: &Session) -> Result<(), RepositoryError> {
        sqlx::query(
            "INSERT INTO sessions (id, device_id, simulator_id, status, created_at, connected_at, disconnected_at, last_activity) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(session.id)
        .bind(&session.device_id)
        .bind(&session.simulator_id)
        .bind(format!("{:?}", session.status))
        .bind(session.created_at)
        .bind(session.connected_at)
        .bind(session.disconnected_at)
        .bind(session.last_activity)
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::QueryError(e.to_string()))?;

        Ok(())
    }

    /// Get a session by ID
    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<Session>, RepositoryError> {
        let row = sqlx::query(
            "SELECT id, device_id, simulator_id, status, created_at, connected_at, disconnected_at, last_activity 
             FROM sessions WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::QueryError(e.to_string()))?;

        Ok(row.map(|r| Session {
            id: r.get("id"),
            device_id: r.get("device_id"),
            simulator_id: r.get("simulator_id"),
            status: match r.get::<String, _>("status").as_str() {
                "Active" => simbridge_shared::protocol::SessionStatus::Active,
                "Paused" => simbridge_shared::protocol::SessionStatus::Paused,
                "Terminated" => simbridge_shared::protocol::SessionStatus::Terminated,
                _ => simbridge_shared::protocol::SessionStatus::Terminated,
            },
            created_at: r.get("created_at"),
            connected_at: r.get("connected_at"),
            disconnected_at: r.get("disconnected_at"),
            last_activity: r.get("last_activity"),
            stream_config: SessionStreamConfig {
                quality: StreamQuality::Medium,
                fps: 30,
                audio_enabled: false,
                video_codec: "h264".to_string(),
            },
        }))
    }

    /// Get all sessions
    pub async fn get_all(&self) -> Result<Vec<Session>, RepositoryError> {
        let rows = sqlx::query(
            "SELECT id, device_id, simulator_id, status, created_at, connected_at, disconnected_at, last_activity 
             FROM sessions"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::QueryError(e.to_string()))?;

        Ok(rows
            .iter()
            .map(|r| Session {
                id: r.get("id"),
                device_id: r.get("device_id"),
                simulator_id: r.get("simulator_id"),
                status: match r.get::<String, _>("status").as_str() {
                    "Active" => simbridge_shared::protocol::SessionStatus::Active,
                    "Paused" => simbridge_shared::protocol::SessionStatus::Paused,
                    "Terminated" => simbridge_shared::protocol::SessionStatus::Terminated,
                    _ => simbridge_shared::protocol::SessionStatus::Terminated,
                },
                created_at: r.get("created_at"),
                connected_at: r.get("connected_at"),
                disconnected_at: r.get("disconnected_at"),
                last_activity: r.get("last_activity"),
                stream_config: SessionStreamConfig {
                    quality: StreamQuality::Medium,
                    fps: 30,
                    audio_enabled: false,
                    video_codec: "h264".to_string(),
                },
            })
            .collect())
    }

    /// Update session
    pub async fn update(&self, session: &Session) -> Result<(), RepositoryError> {
        sqlx::query(
            "UPDATE sessions SET status = ?, disconnected_at = ?, last_activity = ? 
             WHERE id = ?",
        )
        .bind(format!("{:?}", session.status))
        .bind(session.disconnected_at)
        .bind(session.last_activity)
        .bind(session.id)
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::QueryError(e.to_string()))?;

        Ok(())
    }

    /// Delete a session
    pub async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        sqlx::query("DELETE FROM sessions WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::QueryError(e.to_string()))?;

        Ok(())
    }
}

/// Repository errors
#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Query error: {0}")]
    QueryError(String),

    #[error("Not found")]
    NotFound,

    #[error("Serialization error: {0}")]
    SerializationError(String),
}
