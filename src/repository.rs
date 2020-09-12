use anyhow::{anyhow, Result};
use git2::Status as FileStatus;
use std::collections::HashSet;
use std::fmt;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct Repository {
    inner: Arc<Mutex<git2::Repository>>,
    status: Status,
}

impl Repository {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let repository = git2::Repository::open(path)?;
        Ok(Self {
            inner: Arc::new(Mutex::new(repository)),
            status: Status::Unknown,
        })
    }
    pub fn fetch(&self) -> Result<()> {
        let inner = self.inner.lock().unwrap();
        let local_name = inner
            .head()?
            .name()
            .ok_or_else(|| anyhow!("local name is not valid UTF-8"))?
            .to_owned();
        let remote_name = inner.branch_upstream_remote(&local_name)?;
        let mut remote = inner.find_remote(
            remote_name
                .as_str()
                .ok_or_else(|| anyhow!("remote name is not valid UTF-8"))?,
        )?;

        // Create credentials callback for SSH authentication
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_, _, _| {
            git2::Cred::ssh_key(
                "git",
                None,
                std::path::Path::new(&format!("{}/.ssh/id_rsa", std::env::var("HOME").unwrap())),
                None,
            )
        });
        let mut fo = git2::FetchOptions::new();
        fo.remote_callbacks(callbacks);

        // Fetch
        Ok(remote.fetch(&[&local_name], Some(&mut fo), None)?)
    }
    pub fn compute_status(&mut self) -> Result<()> {
        let inner = self.inner.lock().unwrap();
        let mut status_options = git2::StatusOptions::new();
        status_options
            .show(git2::StatusShow::IndexAndWorkdir)
            .include_untracked(true)
            .include_ignored(false);
        let statuses = inner.statuses(Some(&mut status_options))?;
        let status = statuses.iter().fold(HashSet::new(), |mut set, s| {
            set.insert(s.status());
            set
        });
        self.status = Status::Known(status);
        Ok(())
    }
    pub fn status(&self) -> &Status {
        &self.status
    }
    pub fn branch_name(&self) -> Option<String> {
        let inner = self.inner.lock().unwrap();
        let head_branch = git2::Branch::wrap(inner.head().ok()?);
        head_branch.name().ok()?.map(String::from)
    }
    pub fn remote_name(&self) -> Option<String> {
        let inner = self.inner.lock().unwrap();
        let head_branch = git2::Branch::wrap(inner.head().ok()?);
        let remote_branch = head_branch.upstream().ok()?;
        remote_branch.name().ok()?.map(String::from)
    }
    pub fn distance(&self) -> Option<Distance> {
        let inner = self.inner.lock().unwrap();
        let local_ref = inner.head().ok()?;
        let local_oid = local_ref.target()?;
        let upstream_oid = git2::Branch::wrap(local_ref)
            .upstream()
            .ok()?
            .into_reference()
            .target()?;
        match inner.graph_ahead_behind(local_oid, upstream_oid) {
            Ok((0, 0)) => Some(Distance::Same),
            Ok((a, b)) if a > 0 && b == 0 => Some(Distance::Ahead),
            Ok((a, b)) if a == 0 && b > 0 => Some(Distance::Behind),
            Ok((_, _)) => Some(Distance::Both),
            Err(_) => None,
        }
    }
    pub fn commit_summary(&self) -> Option<String> {
        let inner = self.inner.lock().unwrap();
        let head_oid = inner.head().ok()?.target()?;
        let commit = inner.find_commit(head_oid).ok()?;
        commit.summary().map(String::from)
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
