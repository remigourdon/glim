use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg, SubCommand};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    repositories: HashMap<String, PathBuf>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            repositories: HashMap::new(),
        }
    }
    pub fn add_repository<P: AsRef<Path>>(&mut self, path: P) -> bool {
        let path = path.as_ref();
        let name = path
            .components()
            .last()
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap();
        if !self.repositories.contains_key(name) {
            self.repositories.insert(name.to_owned(), path.to_owned());
            true
        } else {
            false
        }
    }
    pub fn remove_repository_by_name(&mut self, name: &str) -> bool {
        self.repositories.remove(name).is_some()
    }
    pub fn rename_repository(&mut self, name: &str, new_name: &str) -> bool {
        if !self.repositories.contains_key(name) || self.repositories.contains_key(new_name) {
            false
        } else {
            let value = self.repositories.remove(name).unwrap();
            self.repositories.insert(new_name.to_owned(), value);
            true
        }
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
                .about("Add new repositories")
                .arg(
                    Arg::with_name("path")
                        .value_name("PATH")
                        .multiple(true)
                        .help("Paths to the repositories to add")
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("remove")
                .about("Remove repositories")
                .arg(
                    Arg::with_name("name")
                        .value_name("NAME")
                        .multiple(true)
                        .help("Names of the repositories to remove")
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("rename")
                .about("Rename repository")
                .arg(
                    Arg::with_name("name")
                        .value_name("NAME")
                        .help("Current name of the repository")
                        .required(true),
                )
                .arg(
                    Arg::with_name("new_name")
                        .value_name("NEW_NAME")
                        .help("New name of the repository")
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

    if let Some(matches) = matches.subcommand_matches("remove") {
        let mut removed = 0;
        for name in matches.values_of("name").unwrap() {
            if config.remove_repository_by_name(name) {
                removed += 1;
            }
        }
        if removed > 0 {
            config.save(config_path);
        }
    }

    if let Some(matches) = matches.subcommand_matches("rename") {
        let name = matches.value_of("name").unwrap();
        let new_name = matches.value_of("new_name").unwrap();
        if config.rename_repository(name, new_name) {
            config.save(config_path);
        }
    }
}
