// Data repository implementations

use sqlx::SqlitePool;
use uuid::Uuid;
use simbridge_shared::models::{Device, Session};
use thiserror::Error;

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
        sqlx::query!(
            "INSERT INTO devices (id, name, device_type, platform, os_version, paired_at, last_seen, is_trusted, public_key) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            device.id,
            device.name,
            format!("{:?}", device.device_type),
            device.platform,
            device.os_version,
            device.paired_at,
            device.last_seen,
            device.is_trusted,
            device.public_key
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::QueryError(e.to_string()))?;
        
        Ok(())
    }

    /// Get a device by ID
    pub async fn get_by_id(&self, id: &str) -> Result<Option<Device>, RepositoryError> {
        let row = sqlx::query!(
            "SELECT id, name, device_type, platform, os_version, paired_at, last_seen, is_trusted, public_key 
             FROM devices WHERE id = ?",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::QueryError(e.to_string()))?;

        Ok(row.map(|r| Device {
            id: r.id,
            name: r.name,
            device_type: match r.device_type.as_str() {
                "Android" => simbridge_shared::models::DeviceType::Android,
                "Ios" => simbridge_shared::models::DeviceType::Ios,
                "Desktop" => simbridge_shared::models::DeviceType::Desktop,
                _ => simbridge_shared::models::DeviceType::Desktop,
            },
            platform: r.platform,
            os_version: r.os_version,
            paired_at: r.paired_at,
            last_seen: r.last_seen,
            is_trusted: r.is_trusted,
            public_key: r.public_key,
        }))
    }

    /// Get all devices
    pub async fn get_all(&self) -> Result<Vec<Device>, RepositoryError> {
        let rows = sqlx::query!(
            "SELECT id, name, device_type, platform, os_version, paired_at, last_seen, is_trusted, public_key 
             FROM devices"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::QueryError(e.to_string()))?;

        Ok(rows.iter().map(|r| Device {
            id: r.id.clone(),
            name: r.name.clone(),
            device_type: match r.device_type.as_str() {
                "Android" => simbridge_shared::models::DeviceType::Android,
                "Ios" => simbridge_shared::models::DeviceType::Ios,
                "Desktop" => simbridge_shared::models::DeviceType::Desktop,
                _ => simbridge_shared::models::DeviceType::Desktop,
            },
            platform: r.platform.clone(),
            os_version: r.os_version.clone(),
            paired_at: r.paired_at,
            last_seen: r.last_seen,
            is_trusted: r.is_trusted,
            public_key: r.public_key.clone(),
        }).collect())
    }

    /// Update device
    pub async fn update(&self, device: &Device) -> Result<(), RepositoryError> {
        sqlx::query!(
            "UPDATE devices SET name = ?, platform = ?, os_version = ?, last_seen = ?, is_trusted = ? 
             WHERE id = ?",
            device.name,
            device.platform,
            device.os_version,
            device.last_seen,
            device.is_trusted,
            device.id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::QueryError(e.to_string()))?;
        
        Ok(())
    }

    /// Delete a device
    pub async fn delete(&self, id: &str) -> Result<(), RepositoryError> {
        sqlx::query!("DELETE FROM devices WHERE id = ?", id)
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
        sqlx::query!(
            "INSERT INTO sessions (id, device_id, simulator_id, status, created_at, connected_at, disconnected_at, last_activity) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            session.id,
            session.device_id,
            session.simulator_id,
            format!("{:?}", session.status),
            session.created_at,
            session.connected_at,
            session.disconnected_at,
            session.last_activity
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::QueryError(e.to_string()))?;
        
        Ok(())
    }

    /// Get a session by ID
    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<Session>, RepositoryError> {
        let row = sqlx::query!(
            "SELECT id, device_id, simulator_id, status, created_at, connected_at, disconnected_at, last_activity 
             FROM sessions WHERE id = ?",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::QueryError(e.to_string()))?;

        Ok(row.map(|r| Session {
            id: r.id,
            device_id: r.device_id,
            simulator_id: r.simulator_id,
            status: match r.status.as_str() {
                "Active" => simbridge_shared::protocol::SessionStatus::Active,
                "Paused" => simbridge_shared::protocol::SessionStatus::Paused,
                "Terminated" => simbridge_shared::protocol::SessionStatus::Terminated,
                _ => simbridge_shared::protocol::SessionStatus::Terminated,
            },
            created_at: r.created_at,
            connected_at: r.connected_at,
            disconnected_at: r.disconnected_at,
            last_activity: r.last_activity,
            stream_config: simbridge_shared::models::StreamConfig {
                quality: simbridge_shared::models::StreamQuality::Medium,
                fps: 30,
                audio_enabled: false,
                video_codec: "h264".to_string(),
            },
        }))
    }

    /// Get all sessions
    pub async fn get_all(&self) -> Result<Vec<Session>, RepositoryError> {
        let rows = sqlx::query!(
            "SELECT id, device_id, simulator_id, status, created_at, connected_at, disconnected_at, last_activity 
             FROM sessions"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::QueryError(e.to_string()))?;

        Ok(rows.iter().map(|r| Session {
            id: r.id,
            device_id: r.device_id.clone(),
            simulator_id: r.simulator_id.clone(),
            status: match r.status.as_str() {
                "Active" => simbridge_shared::protocol::SessionStatus::Active,
                "Paused" => simbridge_shared::protocol::SessionStatus::Paused,
                "Terminated" => simbridge_shared::protocol::SessionStatus::Terminated,
                _ => simbridge_shared::protocol::SessionStatus::Terminated,
            },
            created_at: r.created_at,
            connected_at: r.connected_at,
            disconnected_at: r.disconnected_at,
            last_activity: r.last_activity,
            stream_config: simbridge_shared::models::StreamConfig {
                quality: simbridge_shared::models::StreamQuality::Medium,
                fps: 30,
                audio_enabled: false,
                video_codec: "h264".to_string(),
            },
        }).collect())
    }

    /// Update session
    pub async fn update(&self, session: &Session) -> Result<(), RepositoryError> {
        sqlx::query!(
            "UPDATE sessions SET status = ?, disconnected_at = ?, last_activity = ? 
             WHERE id = ?",
            format!("{:?}", session.status),
            session.disconnected_at,
            session.last_activity,
            session.id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::QueryError(e.to_string()))?;
        
        Ok(())
    }

    /// Delete a session
    pub async fn delete(&self, id: Uuid) -> Result<(), RepositoryError> {
        sqlx::query!("DELETE FROM sessions WHERE id = ?", id)
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
