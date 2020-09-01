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
    pub fn open<P: AsRef<Path>>(name: &str, path: P) -> Self {
        let repository = git2::Repository::open(path).unwrap();
        let mut status_options = git2::StatusOptions::new();
        status_options
            .show(git2::StatusShow::IndexAndWorkdir)
            .include_untracked(true)
            .include_ignored(false);
        Self {
            inner: repository,
            name: name.into(),
            status_options,
            status: Status::UNKNOWN,
        }
    }
    pub fn name<'a>(&'a self) -> &'a str {
        &self.name
    }
    pub fn status<'a>(&'a mut self) -> &'a Status {
        if let Status::UNKNOWN = self.status {
            let status = self
                .inner
                .statuses(Some(&mut self.status_options))
                .unwrap()
                .iter()
                .fold(HashSet::new(), |mut set, s| {
                    set.insert(s.status());
                    set
                });
            self.status = Status::KNOWN(status);
        }
        &self.status
    }
    pub fn branch_name(&self) -> String {
        let head = self.inner.head().unwrap();
        head.shorthand().unwrap().into()
    }
    fn head_branch(&self) -> git2::Branch {
        self.inner
            .find_branch(&self.branch_name(), git2::BranchType::Local)
            .unwrap()
    }
    fn head_oid(&self) -> git2::Oid {
        self.head_branch().into_reference().target().unwrap()
    }
    fn remote_oid(&self) -> git2::Oid {
        let remote_branch = self.head_branch().upstream().unwrap();
        remote_branch.into_reference().target().unwrap()
    }
    pub fn distance(&self) -> Distance {
        match self
            .inner
            .graph_ahead_behind(self.head_oid(), self.remote_oid())
            .unwrap()
        {
            (0, 0) => Distance::Same,
            (a, b) if a > 0 && b == 0 => Distance::Ahead,
            (a, b) if a == 0 && b > 0 => Distance::Behind,
            _ => Distance::Both,
        }
    }
    pub fn commit_summary(&self) -> String {
        let commit = self.inner.find_commit(self.head_oid()).unwrap();
        commit.summary().unwrap().into()
    }
}

pub enum Status {
    KNOWN(HashSet<git2::Status>),
    UNKNOWN,
}

impl Status {
    pub fn has_staged_files(&self) -> bool {
        if let Status::KNOWN(status) = self {
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
        if let Status::KNOWN(status) = self {
            status.contains(&FileStatus::WT_MODIFIED)
                || status.contains(&FileStatus::WT_DELETED)
                || status.contains(&FileStatus::WT_RENAMED)
                || status.contains(&FileStatus::WT_TYPECHANGE)
        } else {
            false
        }
    }
    pub fn has_untracked_files(&self) -> bool {
        if let Status::KNOWN(status) = self {
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
