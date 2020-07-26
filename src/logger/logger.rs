use rusqlite::{params, Connection, Result, NO_PARAMS};
use thiserror::Error;
use std::sync::{Arc, Mutex};

const DATABASE_NAME: &str = "PodcastLibrary.db";

pub struct Log {
    conn: Arc<Mutex<Connection>>,
}

/// LogError enumerates all possible errors returned by the Logger.
#[derive(Error, Debug)]
pub enum LogError {
    /// Represents a logger database connection error
    #[error("Failed to connect to database: {0}")]
    LoggerConnectionError(String),

    /// Represents a logger database error
    #[error("Failed to open database file: {0}")]
    LoggerOpenDBError(String),

    /// Represents a logger database error
    #[error("Failed to insert podcast: {0}")]
    LoggerInsertError(String),

    /// Represents a logger select error
    #[error("Failed select: {0}")]
    LoggerSelectError(String),

    /// Represents a logger database error
    #[error("Failed to create table: {0}")]
    LoggerCreateTableError(String),
}

impl Log {
    pub fn new() -> Result<Log, LogError> {
        let conn = match Connection::open(DATABASE_NAME) {
            Ok(c) => c,
            Err(c) => return Err(LogError::LoggerOpenDBError(String::from(DATABASE_NAME))),
        };

        Ok(Log { conn: Arc::new(Mutex::new(conn)) })
    }

    pub fn create_podcast_table(&self, name: &str) -> Result<(), LogError> {
        let q = format!(
            "CREATE TABLE IF NOT EXISTS `{}` (
                episode STRING PRIMARY KEY UNIQUE
            )",
            name
        );
        {
            let conn = self.conn.lock().unwrap();
            match conn.execute(q.as_str(), NO_PARAMS) {
                Ok(c) => c,
                Err(e) => {
                    return Err(LogError::LoggerCreateTableError(format!(
                        "{}\n{:?}",
                        name.to_string(),
                        e
                    )))
                }
            };
        }

        Ok(())
    }

    fn insert_episode(&self, podcast: &str, episode_name: &str) -> Result<(), LogError> {
        {
            let conn = self.conn.lock().unwrap();
            match conn.execute(
                format!("INSERT INTO `{}` (episode) VALUES (?1)", podcast).as_str(),
                params![episode_name],
            ) {
                Ok(c) => c,
                Err(e) => {
                    return Err(LogError::LoggerInsertError(format!(
                        "{}\n{:?}",
                        podcast.to_string(),
                        e
                    )))
                }
            };
        }

        Ok(())
    }

    pub fn entry_exists(&self, podcast: &str, episode_name: &str) -> Result<bool, LogError> {
        let q = format!("SELECT EXISTS(SELECT 1 FROM `{}` WHERE episode= ? )", podcast);
        let exists;
        {
            let conn = self.conn.lock().unwrap();

            let mut stmt = match conn.prepare(q.as_str()) {
                Ok(c) => c,
                Err(e) => {
                    return Err(LogError::LoggerSelectError(format!(
                        "{}\n{:?}",
                        episode_name.to_string(),
                        e
                    )))
                }
            };

            let mut rows = match stmt.query(params![episode_name]) {
                Ok(c) => c,
                Err(e) => {
                    return Err(LogError::LoggerSelectError(format!(
                        "{}\n{:?}",
                        episode_name.to_string(),
                        e
                    )))
                }
            };

            exists = match rows.next() {
                Err(e) => false,
                Ok(r) => match r {
                    None => false,
                    Some(v) => match v.get(0) {
                        Err(e) => false,
                        Ok(b) => b,
                    },
                },
            };
        }

        Ok(exists)
    }

    pub fn update_log(&self, podcast: &str, episode_name: &str) -> Result<(), LogError> {
        self.insert_episode(podcast, episode_name)?;

        Ok(())
    }
}
