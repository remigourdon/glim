use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
struct Config {
    repositories: HashMap<String, PathBuf>,
}

fn main() {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true),
        )
        .get_matches();

    let config = matches.value_of("config").unwrap_or("default.conf");
    let config = std::fs::read_to_string(&config).unwrap();
    let config: Config = toml::from_str(&config).unwrap();
    println!("{:?}", config);
}
