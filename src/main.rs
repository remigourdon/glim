mod config;

use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg, SubCommand};
use config::Config;
use directories::ProjectDirs;
use git2::{Repository, Status, StatusOptions};
use prettytable::{cell, format, row, Table};
use std::path::Path;

fn main() {
    // Get default config path
    let app_name = crate_name!();
    let project_dirs = ProjectDirs::from("com", app_name, app_name).unwrap();
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
        std::fs::create_dir(config_dir).unwrap();
    }

    // Load config from file or create new one
    let config_path = Path::new(matches.value_of("config").unwrap());
    let mut config: Config = if config_path.is_file() {
        Config::from_path(config_path)
    } else {
        Config::new()
    };

    // Save config
    config.save(config_path);

    if let Some(matches) = matches.subcommand_matches("add") {
        let mut added = 0;
        for path in matches.values_of("path").unwrap() {
            let path = Path::new(path);
            if path.is_dir() && config.add_repository(path) {
                added += 1;
            }
        }
        if added > 0 {
            config.save(config_path);
        }
    }

    if let Some(matches) = matches.subcommand_matches("remove") {
        let mut removed = 0;
        for name in matches.values_of("name").unwrap() {
            if config.remove_repository_by_name(name) {
                removed += 1;
            }
        }
        if removed > 0 {
            config.save(config_path);
        }
    }

    if let Some(matches) = matches.subcommand_matches("rename") {
        let name = matches.value_of("name").unwrap();
        let new_name = matches.value_of("new_name").unwrap();
        if config.rename_repository(name, new_name) {
            config.save(config_path);
        }
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

    // Prepare status options
    let mut status_opts = StatusOptions::new();
    status_opts
        .show(git2::StatusShow::IndexAndWorkdir)
        .include_untracked(true)
        .include_ignored(false);

    // Fill table
    for (name, path) in config.repositories().iter() {
        let repository = Repository::open(path).unwrap();
        let statuses = repository.statuses(Some(&mut status_opts)).unwrap();
        let head = repository.head().unwrap();
        let branch = head.shorthand().unwrap();
        let staged = statuses.iter().any(|s| {
            s.status() == Status::INDEX_NEW
                || s.status() == Status::INDEX_MODIFIED
                || s.status() == Status::INDEX_DELETED
                || s.status() == Status::INDEX_RENAMED
                || s.status() == Status::INDEX_TYPECHANGE
        });
        let unstaged = statuses.iter().any(|s| {
            s.status() == Status::WT_MODIFIED
                || s.status() == Status::WT_DELETED
                || s.status() == Status::WT_RENAMED
                || s.status() == Status::WT_TYPECHANGE
        });
        let untracked = statuses.iter().any(|s| s.status() == Status::WT_NEW);
        let mut state = String::new();
        if staged {
            state.push('+');
        }
        if unstaged {
            state.push('*');
        }
        if untracked {
            state.push('_');
        }
        table.add_row(row![name, branch, state]);
    }

    // Display table
    table.printstd();
}
