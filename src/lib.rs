mod downloader;

extern crate serde;
extern crate serde_derive;
extern crate toml;

use std::path::PathBuf;
use std::{fs, str};

use crate::downloader::downloader::{Podcast, PodcastEntry};

use anyhow::{Error, Result};
use rayon::prelude::*;
use serde_derive::Deserialize;
use std::collections::HashMap;
use thiserror::Error;

const PROGRAM: &str = "rustypod";

/// PodError enumerates all possible errors returned by this library.
#[derive(Error, Debug)]
pub enum PodError {
    /// Represents a configuration file error
    #[error("Failed to read config file: {0}")]
    ConfigReadError(String),

    /// Represents a configuration file error
    #[error("Failed to parse config file: {0}")]
    ConfigParseError(String),

    /// Represents a configuration file error
    #[error("{0}")]
    DirectoryError(String),

    /// Represents all other cases of `std::io::Error`.
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

#[derive(Deserialize)]
struct Config {
    podcasts: Vec<HashMap<String, String>>, // TODO: Parse and convert environment variables
    directory: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            podcasts: Vec::new(),
            directory: String::from(""),
        }
    }
}

pub fn expected_config_location() -> (String, String) {
    if cfg!(target_os = "windows") {
        (
            vec!["%APPDATA%", PROGRAM, "config.toml"].join("\\"),
            vec!["%LOCALAPPDATA%", PROGRAM, "config.toml"].join("\\"),
        )
    } else if cfg!(target_os = "macos") {
        (
            vec!["/Library", "Preferences", PROGRAM, "config.toml"].join("/"),
            vec!["~/Library", "Preferences", PROGRAM, "config.toml"].join("/"),
        )
    } else {
        (
            vec!["/etc", "xdg", PROGRAM, "config.toml"].join("/"),
            vec!["$XDG_CONFIG_HOME", PROGRAM, "config.toml"].join("/"),
        )
    }
}

fn find_config_from_env(env: &str, suffix: &Vec<&str>) -> Option<PathBuf> {
    match std::env::var(env) {
        Ok(val) => {
            let mut cfg_entry = vec![val.as_str()];
            cfg_entry.extend(suffix);
            let path: PathBuf = cfg_entry.iter().collect();
            if path.exists() {
                return Some(path);
            }

            None
        }
        Err(_v) => None,
    }
}

// XDG_CONFIG_HOME -> %LOCALAPPDATA%
// System-wide -> %APPDATA%
fn find_config_windows() -> Option<PathBuf> {
    let suffix = vec![PROGRAM, "config.toml"];

    let config_location = match find_config_from_env("LOCALAPPDATA", &suffix) {
        Some(val) => Some(val),
        None => find_config_from_env("APPDATA", &suffix),
    };

    config_location
}

fn find_user_config_unix(suffix_directory: &Vec<&str>) -> Option<PathBuf> {
    let suffix = vec![PROGRAM, "config.toml"];
    let config_path = match find_config_from_env("XDG_CONFIG_HOME", &suffix) {
        Some(val) => Some(val),
        None => {
            let mut cfg_suffix: Vec<&str> = Vec::new();
            cfg_suffix.extend(suffix_directory);
            cfg_suffix.extend(&suffix);

            find_config_from_env("HOME", &cfg_suffix)
        }
    };

    config_path
}

fn check_path_exists(path: Vec<&str>) -> Option<PathBuf> {
    let path_buffer: PathBuf = path.iter().collect();
    if path_buffer.exists() {
        return Some(path_buffer);
    }
    None
}

fn find_system_config_unix(prefix_directory: &Vec<&str>) -> Option<PathBuf> {
    let mut default_path: Vec<&str> = Vec::new();
    default_path.extend(prefix_directory);
    default_path.extend(&vec![PROGRAM, "config.toml"]);

    let config_path: Option<PathBuf> = match std::env::var("XDG_DATA_DIRS") {
        Ok(val) => {
            let directory_it = val.split(":");
            for e in directory_it {
                let p = check_path_exists(vec![e, PROGRAM, "config.toml"]);
                if p.is_some() {
                    return p;
                }
            }
            check_path_exists(default_path)
        }
        Err(_v) => check_path_exists(default_path),
    };

    config_path
}

fn find_config_unix() -> Option<PathBuf> {
    let config_path: Option<PathBuf> = match find_user_config_unix(&vec![".config"]) {
        Some(val) => Some(val),
        None => find_system_config_unix(&vec!["/", "etc", "xdg"]),
    };

    config_path
}

// XDG_CONFIG_HOME -> ~/Library/Preferences/
// System-wide -> /Library/Preferences/
fn find_config_macos() -> Option<PathBuf> {
    let user_config = find_user_config_unix(&vec!["Library", "Preferences"]);
    let config_path: Option<PathBuf> = match user_config {
        Some(val) => Some(val),
        None => find_system_config_unix(&vec!["/", "Library", "Preferences"]),
    };

    config_path
}

pub fn find_config() -> Result<Option<PathBuf>, PodError> {
    let config_location = if cfg!(target_os = "windows") {
        find_config_windows()
    } else if cfg!(target_os = "macos") {
        find_config_macos()
    } else {
        find_config_unix()
    };

    Ok(config_location)
}

fn read_config(config_file: &str) -> Result<Config, PodError> {
    let file_data = match fs::read(&config_file) {
        Ok(f) => f,
        Err(_f) => return Err(PodError::ConfigReadError(config_file.to_string())),
    };

    match toml::from_slice(&file_data) {
        Ok(f) => Ok(f),
        Err(_f) => Err(PodError::ConfigParseError(config_file.to_string())),
    }
}

pub fn run(config_file: &str) -> Result<(), PodError> {
    let config = read_config(config_file)?;
    let download_dir: PathBuf = PathBuf::from(config.directory);

    if download_dir.exists() {
        if !download_dir.is_dir() {
            return Err(PodError::DirectoryError(format!(
                "Specified directory {} is not a directory",
                download_dir.to_str().unwrap()
            )));
        }
    } else {
        match fs::create_dir(download_dir.clone()) {
            Ok(_o) => _o,
            Err(e) => {
                return Err(PodError::DirectoryError(format!(
                    "Failed to create directory {}:\n{:?}",
                    download_dir.to_str().unwrap(),
                    e
                )));
            }
        }
    }

    let mut pods: Vec<PodcastEntry> = Vec::new();

    for pc in config.podcasts {
        let pod = Podcast::new(
            String::from(pc.get("name").unwrap()),
            String::from(pc.get("uri").unwrap()),
        );
        pods.extend(pod.entries(str::parse::<usize>(pc.get("episodes").unwrap()).unwrap()));
        pod.setup_tree(&download_dir); // TODO: Error handling
    }

    let _results: Vec<Result<(), Error>> =
        pods.par_iter().map(|p| p.download(&download_dir)).collect();

    Ok(())
}
