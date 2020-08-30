use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg, SubCommand};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    repositories: HashSet<PathBuf>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            repositories: HashSet::new(),
        }
    }
    pub fn add_repository<P: AsRef<Path>>(&mut self, path: P) -> bool {
        let path = path.as_ref();
        self.repositories.insert(path.to_owned())
    }
    pub fn save<P: AsRef<Path>>(&self, path: P) -> bool {
        std::fs::write(path.as_ref(), toml::to_vec(self).unwrap()).is_err()
    }
}

fn main() {
    // Get default config path
    let app_name = crate_name!();
    let project_dirs = ProjectDirs::from("com", app_name, app_name).unwrap();
    let default_config_path = project_dirs.config_dir().join("config.toml");

    // Create clap app
    let matches = App::new(app_name)
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .default_value_os(&default_config_path.as_os_str())
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

    // Load config from file or create new one
    let config_path = Path::new(matches.value_of("config").unwrap());
    let mut config: Config = if config_path.is_file() {
        let config = std::fs::read_to_string(&config_path).unwrap();
        toml::from_str(&config).unwrap()
    } else if config_path == default_config_path {
        let config_dir = project_dirs.config_dir();
        // Create the app's config directory if it doesn't exist
        if !config_dir.is_dir() {
            std::fs::create_dir(config_dir).unwrap();
        }
        Config::new()
    } else {
        panic!("Invalid config file");
    };

    // Run subcommand
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
