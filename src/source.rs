use crate::repository::Repository;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Error, Result};
use serde::{Deserialize, Serialize};

pub struct Source<'a> {
    pub name: &'a str,
    pub path: &'a Path,
}

impl<'a> Source<'a> {
    pub fn open(self) -> Result<Repository> {
        Repository::from_source(self)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Sources(HashMap<String, PathBuf>);

impl Sources {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn get<'a>(&'a self, name: &'a str) -> Option<Source> {
        let path = self.0.get(name).map(PathBuf::as_path)?;
        Some(Source { name, path })
    }
    pub fn iter(&self) -> impl Iterator<Item = Source> + '_ {
        self.0.iter().map(|(name, path)| Source { name, path })
    }
    pub fn add<P: AsRef<Path>>(&mut self, path: P, name: Option<&str>) -> Result<()> {
        let path = path.as_ref();
        let name = match name {
            Some(name) => name,
            None => path
                .components()
                .last()
                .ok_or_else(|| Error::msg("path is too short"))?
                .as_os_str()
                .to_str()
                .ok_or_else(|| Error::msg("path is not valid UTF-8"))?,
        };
        if self.0.contains_key(name) {
            Err(anyhow!("name '{}' already exists", name))
        } else {
            self.0.insert(name.to_owned(), path.to_owned());
            Ok(())
        }
    }
    pub fn remove(&mut self, name: &str) -> Result<PathBuf> {
        Ok(self
            .0
            .remove(name)
            .ok_or_else(|| anyhow!("name '{}' does not exist", name))?)
    }
    pub fn rename(&mut self, name: &str, new_name: &str) -> Result<()> {
        if self.0.contains_key(new_name) {
            return Err(anyhow!("name '{}' already exists", new_name));
        }
        let path = self
            .0
            .remove(name)
            .ok_or_else(|| anyhow!("name '{}' does not exist", name))?;
        self.0.insert(new_name.to_owned(), path);
        Ok(())
    }
}
