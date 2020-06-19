extern crate clap;

use clap::{App, Arg};

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

}
