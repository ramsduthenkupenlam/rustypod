use rusqlite::{params, Connection, Result, NO_PARAMS};
use thiserror::Error;

const DATABASE_NAME: &str = "PodcastLibrary.db";

pub struct Log {
    conn: Option<Connection>,
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

    /// Represents a logger database error
    #[error("Failed to create table: {0}")]
    LoggerCreateTableError(String),
}

impl Log {
    pub fn new() -> Log {
        Log { conn: None }
    }

    pub fn open_connection(&mut self) -> Result<(), LogError> {
        self.conn = match Connection::open(DATABASE_NAME) {
            Ok(c) => Some(c),
            Err(c) => return Err(LogError::LoggerOpenDBError(String::from(DATABASE_NAME))),
        };

        Ok(())
    }

    fn create_podcast_table(name: &str, conn: &Connection) -> Result<(), LogError> {
        let q = format!(
            "CREATE TABLE IF NOT EXISTS {} (
                episode STRING PRIMARY KEY UNIQUE
            )",
            name
        );
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

        Ok(())
    }

    fn insert_episode(
        podcast: &str,
        episode_name: &str,
        conn: &Connection,
    ) -> Result<(), LogError> {
        match conn.execute(
            format!("INSERT INTO {} (episode) VALUES (?1)", podcast).as_str(),
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

        Ok(())
    }

    pub fn update_log(&self, podcast: &str, episode_name: &str) -> Result<(), LogError> {
        let conn = match &self.conn {
            None => return Err(LogError::LoggerConnectionError(DATABASE_NAME.to_string())),
            Some(c) => c,
        };

        Log::create_podcast_table(podcast, &conn)?;
        Log::insert_episode(podcast, episode_name, &conn)?;

        Ok(())
    }
}
