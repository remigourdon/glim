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
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        let string = std::fs::read_to_string(path).unwrap();
        toml::from_str(&string).unwrap()
    }
    pub fn repositories(&self) -> &HashMap<String, PathBuf> {
        &self.repositories
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
