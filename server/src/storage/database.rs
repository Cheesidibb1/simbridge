// Database connection and initialization

use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::path::Path;
use thiserror::Error;

/// Database connection pool
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Create a new database connection
    pub async fn new(database_path: &Path) -> Result<Self, DatabaseError> {
        // Ensure parent directory exists
        if let Some(parent) = database_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| DatabaseError::CreateDirError(e.to_string()))?;
        }

        let options = SqliteConnectOptions::new()
            .filename(database_path)
            .create_if_missing(true);
        let pool = SqlitePool::connect_with(options)
            .await
            .map_err(|e| DatabaseError::ConnectionError(e.to_string()))?;

        Ok(Self { pool })
    }

    /// Run database migrations
    pub async fn migrate(&self) -> Result<(), DatabaseError> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| DatabaseError::MigrationError(e.to_string()))?;

        Ok(())
    }

    /// Get the connection pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Close the database connection
    pub async fn close(self) {
        self.pool.close().await;
    }
}

/// Database errors
#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Migration error: {0}")]
    MigrationError(String),

    #[error("Query error: {0}")]
    QueryError(String),

    #[error("Create directory error: {0}")]
    CreateDirError(String),

    #[error("Not found")]
    NotFound,
}
