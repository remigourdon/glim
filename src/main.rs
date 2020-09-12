mod config;
mod repository;

use anyhow::{anyhow, Context, Result};
use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg, SubCommand};
use config::Config;
use directories::ProjectDirs;
use indicatif::{ProgressBar, ProgressStyle};
use prettytable::{cell, format, row, Table};
use repository::Repository;
use std::collections::BTreeMap;
use std::path::Path;
use std::str::FromStr;
use std::sync::mpsc::channel;
use threadpool::ThreadPool;

fn main() -> Result<()> {
    // Get default config path
    let app_name = crate_name!();
    let project_dirs = ProjectDirs::from("com", app_name, app_name)
        .ok_or_else(|| anyhow!("could not retrieve home directory from system"))?;
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
        .arg(
            Arg::with_name("nofetch")
                .short("F")
                .long("no-fetch")
                .help("Do not fetch"),
        )
        .arg(
            Arg::with_name("workers")
                .long("workers")
                .value_name("NUM_WORKERS")
                .default_value("4")
                .help("Number of workers")
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

    // Create config directory if it doesn't exist
    let config_dir = project_dirs.config_dir();
    if !config_dir.is_dir() {
        std::fs::create_dir(config_dir).context("failed to create config directory")?;
    }

    // Load config from file or create new one
    let config_path = Path::new(matches.value_of("config").unwrap());
    let mut config: Config = if config_path.is_file() {
        Config::from_path(config_path).context("failed to load config")?
    } else {
        Config::new()
    };

    // Track config modification
    let mut modified = false;

    if let Some(matches) = matches.subcommand_matches("add") {
        for path in matches.values_of("path").unwrap() {
            let path = Path::new(path);
            config.add_repository(path)?;
            modified = true;
        }
    }

    if let Some(matches) = matches.subcommand_matches("remove") {
        for name in matches.values_of("name").unwrap() {
            if config.remove_repository_by_name(name) {
                modified = true;
            }
        }
    }

    if let Some(matches) = matches.subcommand_matches("rename") {
        let name = matches.value_of("name").unwrap();
        let new_name = matches.value_of("new_name").unwrap();
        config.rename_repository(name, new_name)?;
        modified = true;
    }

    // Save config
    if modified {
        config.save(config_path).context("failed to save config")?;
    }

    // Create thread pool
    let num_workers = usize::from_str(matches.value_of("workers").unwrap())
        .context("invalid number of workers")?;
    let pool = ThreadPool::new(num_workers);
    let (tx, rx) = channel();
    let num_jobs = config.repository_count();

    // Create progress bar
    let pb = ProgressBar::new(num_jobs as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{prefix} [{bar:60}] {pos}/{len}: {msg}")
            .progress_chars("=> "),
    );
    pb.set_prefix("Processing...");

    // To fetch or not to fetch
    let do_fetch = !matches.is_present("nofetch");

    // Open repositories and process on thread pool
    for (name, path) in config.repositories().iter() {
        let name = name.clone();
        let path = path.clone();
        let tx = tx.clone();
        let pb = pb.clone();

        pool.execute(move || {
            // Attempt to open the repository
            let repository = if let Ok(mut repository) = Repository::open(path) {
                // Attempt to fetch from repository
                if do_fetch {
                    let _ = repository.fetch();
                }
                // Compute status now since it can be slow
                let _ = repository.compute_status();
                Some(repository)
            } else {
                None
            };

            // Update progress bar
            pb.set_message(&name);
            pb.inc(1);

            tx.send((name, repository)).unwrap();
        });
    }

    // Join threads and collect data in a sorted map
    let sorted_map =
        rx.iter()
            .take(num_jobs)
            .fold(BTreeMap::new(), |mut map, (name, repository)| {
                map.insert(name, repository);
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
        if let Some(repository) = repository {
            // Get distance between local and upstream
            let distance = if let Some(distance) = repository.distance() {
                distance.to_string()
            } else {
                String::new()
            };
            table.add_row(row![
                name,
                repository.status().to_string(),
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
    }

    // Display table
    table.printstd();

    Ok(())
}
