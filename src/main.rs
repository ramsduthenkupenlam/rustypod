extern crate clap;
extern crate feed_rs;

use clap::{App, Arg};
use reqwest::blocking;
use feed_rs::parser;

fn main() {
    let matches = App::new("rustpod")
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

    let body = blocking::get("https://pythonbytes.fm/episodes/rss")
        .unwrap().text().unwrap();

    let feed_from_xml = parser::parse(body.as_bytes()).unwrap();

    for e in feed_from_xml.entries {
        println!("Date: {}", e.published.unwrap());
        println!("Title: {}", e.title.unwrap().content);
        println!("Link: {}", e.content.unwrap().src.unwrap().href);
        println!("{}", "");
    }
}
