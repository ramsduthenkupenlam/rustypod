use std::collections::HashMap;
use std::path::Path;

use crate::PodError;
use feed_rs::parser;
use reqwest::blocking;
use std::fs;
use std::fs::File;
use std::io::copy;

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

    pub fn download(&self, location: &Path) -> Result<(), PodError> {
        let body = blocking::get(&self.uri).unwrap().text().unwrap();

        let feed_from_xml = parser::parse(body.as_bytes()).unwrap();

        if location.join(&self.name).exists() {
            if !location.join(&self.name).is_dir() {
                return Err(PodError::DirectoryError(format!(
                    "Specified directory {} is not a directory",
                    location.join(&self.name).to_str().unwrap()
                )));
            }
        } else {
            match fs::create_dir(location.join(&self.name).clone()) {
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

        for e in feed_from_xml.entries {
            let link = e.content.unwrap().src.unwrap();
            let title = e.title.unwrap().content;
            println!("Date: {}", e.published.unwrap());
            println!("Title: {}", title);
            println!("Link: {}", link.href);
            let p = location.join(&self.name).join(title);

            println!("Downloading to {} ...", p.to_str().unwrap());

            let mut resp = blocking::get(&link.href).unwrap();

            println!("{:#?}", resp);

            let mut out = File::create(p).expect("failed to create file");
            copy(&mut resp, &mut out).expect("failed to copy content");
        }

        Ok(())
    }
}
