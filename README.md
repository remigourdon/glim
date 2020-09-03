# glim

`glim` is a CLI tool to track the state of multiple local git repositories.

This is very much an early work in progress 🏗️.

It is inspired by [gita](https://github.com/nosarthur/gita.git).

## TODO

+ [x] Error handling
+ [ ] Integration testing
+ [ ] CI
+ [ ] Publish to [crates.io](https://crates.io/)
+ [ ] Better handling of large repositories
    + [ ] Parallelization
    + [ ] Progress feedback (spinners?)

## Usage

The program keeps a list of repositories (in `~/.config/glim/config.toml`) on Linux.

Programs can be added/removed/renamed using the following subcommands:

+ `glim add <REPO_PATH>...`
+ `glim remove <NAME>...`
+ `glim rename <NAME> <NEW_NAME>` (the default name is the repository's directory)

This will produce a `cargo.toml` file of this form (which can also be edited manually):

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

+ `+`: staged changes
+ `*`: unstaged changes
+ `_`: untracked files

Those symbols show whether the local branch is ahead and/or behind its tracked remote:

+ `==`: they are the same
+ `<<`: local is behind remote
+ `>>`: local is ahead of remote
+ `<>`: local is both ahead and behind of remote

## Installation

At the moment, `glim` can be installed from source as follows:

```sh
git clone https://github.com/remigourdon/glim.git
cd glim/
cargo install --path .
```
