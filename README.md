# glim

`glim` is a CLI tool to track the state of multiple local git repositories.

This is very much an early work in progress 🏗️.

It is inspired by [gita](https://github.com/nosarthur/gita.git).

## TODO

+ [x] Error handling
+ [ ] Integration testing
+ [ ] CI
+ [ ] Publish to [crates.io](https://crates.io/)
+ [x] Better handling of large repositories
    + [x] Parallelization
    + [x] Progress feedback

## Usage

The program keeps a list of repositories (in `~/.config/glim/config.toml`) on Linux.
It assigns a default name which is the repository's directory name.

The following subcommands are available:

+ `glim add <REPO_PATH>...`: add new repositories (by path)
+ `glim remove <NAME>...`: remove repositories (by name)
+ `glim rename <NAME> <NEW_NAME>`: rename a repository
+ `glim path <NAME>`: read the path of a repository

This will produce a `config.toml` file of this form (which can also be edited manually):

```text
[repositories]
first-repo = "/home/remi/Projects/first-repo"
second-repo = "/home/remi/Projects/ideas/another-repo"
```

Finally, running the program without a subcommand results in the display of their status:

```text
$ glim
 first-repo     +_    main       ==    origin/main     Update README.md
 second-repo    *     develop    <<    fork/develop    Initial commit
```

The following symbols indicate the status of the repository:

+ `+` if it contains staged changes
+ `*` if it contains unstaged changes
+ `_` if it contains untracked files

Those symbols show whether the local branch is ahead and/or behind its tracked remote:

+ `==` if they are the same
+ `<<` if the local branch is behind the remote branch
+ `>>` if the local is ahead of the remote branch
+ `<>` if the local is both ahead and behind of the remote branch

## Installation

Currently no binary releases are available, so Rust needs to be installed.
This can be done [following the instructions on this page](https://www.rust-lang.org/tools/install).

At the moment, `glim` can be installed from source as follows:

```sh
cargo install --git https://github.com/remigourdon/glim.git --branch main
```
