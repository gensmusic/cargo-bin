# cargo-bin

[<img alt="github" src="https://img.shields.io/badge/github-gensmusic/cargo--bin-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/gensmusic/cargo-bin)
[<img alt="crates.io" src="https://img.shields.io/crates/v/cargo-bin.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/cargo-bin)
![Rust](https://github.com/gensmusic/cargo-bin/workflows/Rust/badge.svg)

The `cargo bin` subcommand provides some operations to manage binaries in Cargo.toml.

# install

```shell script
# install
cargo install cargo-bin
```

# usage

## create a new binary main file and add into Cargo.toml

Create a new binary `abc` and add into Cargo.toml.
The following will create a file abc.rs with a default `fn main()` in current folder.
And a `[[bin]]` will be added into the Cargo.toml.

```shell script
cd src
cargo bin new abc
# or
cargo bin new abc.rs
```

The Cargo.toml file.

```toml
[[bin]]
name = "abc"
path = "src/abc.rs"
```

## tidy

`cargo bin tidy` will add all `.rs` file with a `main` function into Cargo.toml.
It will also clean up all the invalid `[[bin]]`s which doesn't exists.

```shell script
cargo bin tidy
```