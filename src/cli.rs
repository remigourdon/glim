use crate::config::Config;
use crate::repository::Repository;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::mpsc::channel;

use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use prettytable::{cell, format, row, Table};
use structopt::StructOpt;
use threadpool::ThreadPool;

#[derive(StructOpt)]
#[structopt(about)]
pub struct CLI {
    /// Set a custom config file
    #[structopt(short, long, default_value)]
    config: Config,

    /// Do not fetch
    #[structopt(short = "F", long)]
    no_fetch: bool,

    /// Number of workers
    #[structopt(short, long, default_value = "4")]
    workers: usize,

    #[structopt(subcommand)]
    command: Option<Command>,
}

#[derive(StructOpt)]
enum Command {
    /// Add new repositories
    Add { path: Vec<PathBuf> },
    /// Remove repositories
    Remove { name: Vec<String> },
    /// Rename repository
    Rename { name: String, new_name: String },
    /// Get repository's path
    Path { name: String },
}

impl CLI {
    pub fn run(&mut self) -> Result<()> {
        self.run_command()?;
        self.run_process()
    }
    fn run_command(&mut self) -> Result<()> {
        let mut modified = false;
        match &self.command {
            Some(Command::Add { path }) => {
                for path in path {
                    self.config.add_repository(path)?;
                    modified = true;
                }
            }
            Some(Command::Remove { name }) => {
                for name in name {
                    if self.config.remove_repository_by_name(&name) {
                        modified = true;
                    }
                }
            }
            Some(Command::Rename { name, new_name }) => {
                self.config.rename_repository(&name, &new_name)?;
                modified = true;
            }
            Some(Command::Path { name }) => {
                let path = self
                    .config
                    .repositories()
                    .get(name)
                    .context("name does not exist")?;
                println!("{:?}", path);
            }
            None => {}
        }

        // Save config
        if modified {
            self.config.save().context("failed to save config")?;
        }
        Ok(())
    }
    fn run_process(&self) -> Result<()> {
        // Attempt to open repositories
        let mut repositories = Vec::with_capacity(self.config.repositories().len());
        for (name, path) in self.config.repositories() {
            match Repository::open(name, path) {
                Ok(repository) => repositories.push(repository),
                Err(e) => eprintln!("Could not open '{}': {}", name, e),
            }
        }

        // Create thread pool
        let pool = ThreadPool::new(self.workers);
        let (tx, rx) = channel();
        let num_jobs = repositories.len();

        // Create progress bar
        let pb = ProgressBar::new(num_jobs as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{prefix} [{bar:60}] {pos}/{len}: {msg}")
                .progress_chars("=> "),
        );
        pb.set_prefix("Processing...");

        let do_fetch = !self.no_fetch;

        // Process repositories on thread pool
        for repository in repositories.into_iter() {
            let mut repository = repository;
            let tx = tx.clone();
            let pb = pb.clone();

            pool.execute(move || {
                // Attempt to fetch from repository
                if do_fetch {
                    let _ = repository.fetch();
                }
                // Compute status now since it can be slow
                let _ = repository.compute_status();

                // Update progress bar
                pb.set_message(repository.name());
                pb.inc(1);

                tx.send(repository).unwrap();
            });
        }

        // Join threads and collect data in a sorted map
        let sorted_map = rx
            .iter()
            .take(num_jobs)
            .fold(BTreeMap::new(), |mut map, repository| {
                map.insert(repository.name().to_string(), repository);
                map
            });

        // Clear progress bar
        pb.finish_and_clear();

        // Create table
        let mut table = Table::new();

        // Format table
        let format = format::FormatBuilder::new()
            .column_separator(' ')
            .borders(' ')
            .padding(0, 3)
            .build();
        table.set_format(format);

        // Add rows to table
        for (name, repository) in sorted_map.iter() {
            // Get status
            let status = if let Some(status) = repository.status() {
                status.to_string()
            } else {
                String::new()
            };
            // Get distance between local and upstream
            let distance = if let Some(distance) = repository.distance() {
                distance.to_string()
            } else {
                String::new()
            };
            table.add_row(row![
                name,
                status,
                repository.branch_name().unwrap_or_default().to_string(),
                distance,
                repository.remote_name().unwrap_or_default().to_string(),
                repository
                    .commit_summary()
                    .unwrap_or_default()
                    .chars()
                    .take(50)
                    .collect::<String>()
            ]);
        }

        // Display table
        table.printstd();

        Ok(())
    }
}
