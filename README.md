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
one-cool-project = "/home/remi/Projects/one-cool-project"
another-idea = "/home/remi/Projects/ideas/another-idea"
```

Finally, running the program without a subcommand results in the display of their status:

```text
$ glim
 one-cool-project      main        +_      ==    Update README.md
 another-idea          develop     *       <<    Initial commit
```

In the third column, the symbols indicate that the repo has:

+ `+`: staged changes
+ `*`: unstaged changes
+ `_`: untracked files

In the 4th column, the ahead/behind status of the current local branch with respect to its remote is indicated:

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
