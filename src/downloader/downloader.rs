use std::path::{Path, PathBuf};

use crate::PodError;
use anyhow::Error;
use feed_rs::parser;
use reqwest::blocking;
use std::fs;
use std::fs::File;
use std::io::copy;

pub struct PodcastEntry {
    uri: String,
    date: String,
    title: String,
    name: String,
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

    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn title(&self) -> &String {
        &self.title
    }
    pub fn uri(&self) -> &String {
        &self.uri
    }
    pub fn date(&self) -> &String {
        &self.date
    }

    pub fn download(&self, location: &PathBuf) -> Result<(), Error> {
        let mut resp = blocking::get(&self.uri).unwrap();

        let ext: Vec<&str> = self.uri.rsplit('.').collect::<Vec<&str>>();
        let mut p = location.clone().join(&self.name);

        if ext.len() > 0 {
            p = p.join(vec![self.title.clone(), ext[0].to_string()].join("."));
        } else {
            p = p.join(&self.title.clone());
        };

        println!("Downloaded to {} ...", p.to_str().unwrap());

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

    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn uri(&self) -> &String {
        &self.uri
    }

    pub fn entries(&self, episodes: usize) -> Vec<PodcastEntry> {
        let body = blocking::get(&self.uri).unwrap().text().unwrap();
        let feed_from_xml = parser::parse(body.as_bytes()).unwrap();

        let mut ents = Vec::new();

        let mut n = 0;

        for e in feed_from_xml.entries {
            if n >= episodes {
                break;
            }

            ents.push(PodcastEntry {
                uri: String::from(e.content.unwrap().src.unwrap().href),
                title: e.title.unwrap().content.to_string(),
                date: e.published.unwrap().to_string(),
                name: self.name.clone(),
            });
            n += 1;
        }

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
