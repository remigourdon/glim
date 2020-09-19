use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use structopt::clap::crate_name;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    #[serde(skip)]
    path: PathBuf,
    repositories: HashMap<String, PathBuf>,
}

impl Config {
    pub fn new() -> Result<Self> {
        // Get default config path
        let app_name = crate_name!();
        let project_dirs = ProjectDirs::from("com", app_name, app_name)
            .ok_or_else(|| anyhow!("could not retrieve home directory from system"))?;
        let default_config_path = project_dirs.config_dir().join("config.toml");
        // Create config directory if it doesn't exist
        let config_dir = project_dirs.config_dir();
        if !config_dir.is_dir() {
            std::fs::create_dir(config_dir).context("failed to create config directory")?;
        }
        Ok(Self {
            path: default_config_path,
            repositories: HashMap::new(),
        })
    }
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let string = std::fs::read_to_string(path)?;
        let mut config: Config = toml::from_str(&string)?;
        config.path = path.to_owned();
        Ok(config)
    }
    pub fn repositories(&self) -> &HashMap<String, PathBuf> {
        &self.repositories
    }
    pub fn add_repository<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();
        let name = path
            .components()
            .last()
            .ok_or_else(|| anyhow!("path is too short"))?
            .as_os_str()
            .to_str()
            .ok_or_else(|| anyhow!("path is not valid UTF-8"))?;
        if !self.repositories.contains_key(name) {
            self.repositories.insert(name.to_owned(), path.to_owned());
            Ok(())
        } else {
            Err(anyhow!("name '{}' already exists", name))
        }
    }
    pub fn remove_repository_by_name(&mut self, name: &str) -> bool {
        self.repositories.remove(name).is_some()
    }
    pub fn rename_repository(&mut self, name: &str, new_name: &str) -> Result<()> {
        if !self.repositories.contains_key(name) {
            Err(anyhow!("name '{}' does not exist", name))
        } else if self.repositories.contains_key(new_name) {
            Err(anyhow!("name '{}' already exists", new_name))
        } else {
            let value = self
                .repositories
                .remove(name)
                .expect("failed to remove repository");
            self.repositories.insert(new_name.to_owned(), value);
            Ok(())
        }
    }
    pub fn save(&self) -> Result<()> {
        Ok(std::fs::write(&self.path, toml::to_vec(self)?)?)
    }
}
