extern crate clap;
extern crate feed_rs;

use std::path::PathBuf;
use std::process;

use anyhow::{Context, Result};
use clap::{App, Arg};
use rustypod::{expected_config_location, find_config, run};

fn main() -> Result<()> {
    let matches = App::new("rustypod")
        .version("0.0.0")
        .author("John Ramsden and Joe Puthenkulam")
        .about("Podcast downloader")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true),
        )
        .get_matches();

    let config_file = match matches.value_of("config") {
        Some(c) => PathBuf::from(c),
        None => {
            let cfg = find_config();
            match cfg {
                Ok(c) => match c {
                    Some(c) => c,
                    None => {
                        let conf_expected = expected_config_location();
                        eprintln!("No config file found in system or user directories.");
                        eprintln!("{} or {}", conf_expected.0, conf_expected.1);
                        process::exit(1);
                    }
                },
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "Error occurred looking for config file {:?}",
                        e
                    ));
                }
            }
        }
    };

    let config_file = config_file
        .to_str()
        .context("Config file is invalid unicode.")?;

    match run(config_file) {
        Ok(()) => Ok(()),
        Err(e) => {
            return Err(anyhow::anyhow!("{}", e));
        }
    }
}
