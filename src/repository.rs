use anyhow::{anyhow, Result};
use git2::Status as FileStatus;
use std::collections::HashSet;
use std::fmt;
use std::path::Path;

pub struct Repository {
    inner: git2::Repository,
    name: String,
    status_options: git2::StatusOptions,
    status: Status,
}

impl Repository {
    pub fn open<P: AsRef<Path>>(name: &str, path: P) -> Result<Self> {
        let repository = git2::Repository::open(path)?;
        let mut status_options = git2::StatusOptions::new();
        status_options
            .show(git2::StatusShow::IndexAndWorkdir)
            .include_untracked(true)
            .include_ignored(false);
        Ok(Self {
            inner: repository,
            name: name.into(),
            status_options,
            status: Status::Unknown,
        })
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn status<'a>(&'a mut self) -> Result<&'a Status> {
        if let Status::Unknown = self.status {
            let status = self
                .inner
                .statuses(Some(&mut self.status_options))?
                .iter()
                .fold(HashSet::new(), |mut set, s| {
                    set.insert(s.status());
                    set
                });
            self.status = Status::Known(status);
        }
        Ok(&self.status)
    }
    pub fn branch_name(&self) -> Option<String> {
        if let Ok(head) = self.inner.head() {
            head.shorthand().map(str::to_string)
        } else {
            None
        }
    }
    fn head_branch(&self) -> Result<git2::Branch> {
        Ok(self.inner.find_branch(
            &self
                .branch_name()
                .ok_or_else(|| anyhow!("head is not a branch"))?,
            git2::BranchType::Local,
        )?)
    }
    fn head_oid(&self) -> Result<git2::Oid> {
        Ok(self
            .head_branch()?
            .into_reference()
            .target()
            .ok_or_else(|| anyhow!("reference is indirect"))?)
    }
    fn remote_branch(&self) -> Result<git2::Branch> {
        Ok(self.head_branch()?.upstream()?)
    }
    fn remote_reference(&self) -> Result<git2::Reference> {
        Ok(self.remote_branch()?.into_reference())
    }
    pub fn remote_name(&self) -> Option<String> {
        if let Ok(remote) = self.remote_reference() {
            remote.shorthand().map(str::to_string)
        } else {
            None
        }
    }
    fn remote_oid(&self) -> Result<git2::Oid> {
        Ok(self
            .remote_reference()?
            .target()
            .ok_or_else(|| anyhow!("remote is not a branch"))?)
    }
    pub fn distance(&self) -> Option<Distance> {
        if let (Ok(head), Ok(remote)) = (self.head_oid(), self.remote_oid()) {
            let distance = match self.inner.graph_ahead_behind(head, remote).ok()? {
                (0, 0) => Distance::Same,
                (a, b) if a > 0 && b == 0 => Distance::Ahead,
                (a, b) if a == 0 && b > 0 => Distance::Behind,
                _ => Distance::Both,
            };
            Some(distance)
        } else {
            None
        }
    }
    pub fn commit_summary(&self) -> Result<String> {
        let commit = self.inner.find_commit(self.head_oid()?)?;
        Ok(commit
            .summary()
            .ok_or_else(|| anyhow!("commit summary is not valid UTF-8"))?
            .into())
    }
}

pub enum Status {
    Known(HashSet<git2::Status>),
    Unknown,
}

impl Status {
    pub fn has_staged_files(&self) -> bool {
        if let Status::Known(status) = self {
            status.contains(&FileStatus::INDEX_NEW)
                || status.contains(&FileStatus::INDEX_MODIFIED)
                || status.contains(&FileStatus::INDEX_DELETED)
                || status.contains(&FileStatus::INDEX_RENAMED)
                || status.contains(&FileStatus::INDEX_TYPECHANGE)
        } else {
            false
        }
    }
    pub fn has_unstaged_files(&self) -> bool {
        if let Status::Known(status) = self {
            status.contains(&FileStatus::WT_MODIFIED)
                || status.contains(&FileStatus::WT_DELETED)
                || status.contains(&FileStatus::WT_RENAMED)
                || status.contains(&FileStatus::WT_TYPECHANGE)
        } else {
            false
        }
    }
    pub fn has_untracked_files(&self) -> bool {
        if let Status::Known(status) = self {
            status.contains(&FileStatus::WT_NEW)
        } else {
            false
        }
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut string = String::with_capacity(3);
        if self.has_staged_files() {
            string.push('+');
        }
        if self.has_unstaged_files() {
            string.push('*');
        }
        if self.has_untracked_files() {
            string.push('_');
        }
        write!(f, "{}", string)
    }
}

pub enum Distance {
    Same,
    Ahead,
    Behind,
    Both,
}

impl fmt::Display for Distance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            Distance::Same => "==",
            Distance::Ahead => ">>",
            Distance::Behind => "<<",
            Distance::Both => "<>",
        };
        write!(f, "{}", symbol)
    }
}
