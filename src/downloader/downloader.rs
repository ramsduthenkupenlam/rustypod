use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::PodError;
use anyhow::Error;
use feed_rs::parser;
use reqwest::blocking;
use std::fs;
use std::fs::File;
use std::io::copy;
use std::thread;
use thiserror::Error;

pub struct PodcastEntry {
    pub(crate) uri: String,
    pub(crate) date: String,
    pub(crate) title: String,
    pub(crate) name: String,
}

impl PodcastEntry {
    pub fn new(title: String, uri: String, date: String, name: String) -> PodcastEntry {
        PodcastEntry {
            name: name,
            title: title,
            uri: uri,
            date: date,
        }
    }

    pub fn download(&self, location: &PathBuf) -> Result<(), Error> {
        let p = location.clone().join(&self.name).join(&self.title);

        println!("Downloaded to {} ...", p.to_str().unwrap());

        let mut resp = blocking::get(&self.uri).unwrap();

        let mut out = File::create(p).expect("failed to create file");
        copy(&mut resp, &mut out).expect("failed to copy content");

        Ok(())
    }
}

pub struct Podcast {
    name: String,
    uri: String,
}

impl Podcast {
    pub fn new(name: String, uri: String) -> Podcast {
        Podcast {
            name: name,
            uri: uri,
        }
    }

    pub fn entries(&self) -> Vec<PodcastEntry> {
        let body = blocking::get(&self.uri).unwrap().text().unwrap();
        let feed_from_xml = parser::parse(body.as_bytes()).unwrap();

        let ents: Vec<PodcastEntry> = feed_from_xml
            .entries
            .into_iter()
            .map(|e| PodcastEntry {
                uri: String::from(e.content.unwrap().src.unwrap().href),
                title: e.title.unwrap().content.to_string(),
                date: e.published.unwrap().to_string(),
                name: self.name.clone(),
            })
            .collect();

        ents
    }

    pub fn setup_tree(&self, parent_dir: &Path) -> Result<(), PodError> {
        let location = parent_dir.clone().join(&self.name);

        if location.exists() {
            if !location.is_dir() {
                return Err(PodError::DirectoryError(format!(
                    "Specified directory {} is not a directory",
                    location.join(&self.name).to_str().unwrap()
                )));
            }
        } else {
            match fs::create_dir(location.clone()) {
                Ok(_o) => _o,
                Err(e) => {
                    return Err(PodError::DirectoryError(format!(
                        "Failed to create directory {}:\n{:?}",
                        location.join(&self.name).to_str().unwrap(),
                        e
                    )));
                }
            }
        }

        Ok(())
    }
}
