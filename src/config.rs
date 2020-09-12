use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    repositories: HashMap<String, PathBuf>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            repositories: HashMap::new(),
        }
    }
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let string = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&string)?;
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
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        Ok(std::fs::write(path.as_ref(), toml::to_vec(self)?)?)
    }
}
