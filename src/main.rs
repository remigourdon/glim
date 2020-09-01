mod config;
mod repository;

use anyhow::{anyhow, Context, Result};
use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg, SubCommand};
use config::Config;
use directories::ProjectDirs;
use prettytable::{cell, format, row, Table};
use repository::Repository;
use std::path::Path;

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

    // Create table
    let mut table = Table::new();

    // Format table
    let format = format::FormatBuilder::new()
        .column_separator(' ')
        .borders(' ')
        .padding(0, 3)
        .build();
    table.set_format(format);

    // Open repositories
    let repositories = config
        .repositories()
        .iter()
        .filter_map(|(name, path)| Repository::open(name, path).ok());

    // Fill table
    for mut repository in repositories {
        // Get distance (ahead / behind remote)
        let distance = match repository.distance() {
            Some(distance) => distance.to_string(),
            None => String::new(),
        };
        table.add_row(row![
            repository.name(),
            repository
                .branch_name()
                .unwrap_or_else(|| String::from("UNKNOWN")),
            repository.status().context("failed to get status")?,
            distance,
            repository
                .commit_summary()
                .unwrap_or_else(|_| String::new())
        ]);
    }

    // Display table
    table.printstd();

    Ok(())
}
