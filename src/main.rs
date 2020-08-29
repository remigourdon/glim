use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg, SubCommand};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    repositories: HashSet<PathBuf>,
}

impl Config {
    pub fn add_repository<P: AsRef<Path>>(&mut self, path: P) -> bool {
        let path = path.as_ref();
        self.repositories.insert(path.to_owned())
    }
    pub fn save<P: AsRef<Path>>(&self, path: P) -> bool {
        std::fs::write(path.as_ref(), toml::to_vec(self).unwrap()).is_err()
    }
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
        .subcommand(
            SubCommand::with_name("add")
                .about("Add new repository")
                .arg(
                    Arg::with_name("path")
                        .value_name("PATH")
                        .multiple(true)
                        .help("Paths to the repositories to add")
                        .required(true),
                ),
        )
        .get_matches();

    let config_path = matches.value_of("config").unwrap_or("default.conf");
    let config = std::fs::read_to_string(&config_path).unwrap();
    let mut config: Config = toml::from_str(&config).unwrap();

    if let Some(matches) = matches.subcommand_matches("add") {
        let mut added = 0;
        for path in matches.values_of("path").unwrap() {
            let path = Path::new(path);
            if path.is_dir() && config.add_repository(path) {
                added += 1;
            }
        }
        if added > 0 {
            config.save(config_path);
        }
    }
}
