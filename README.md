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

```shell script
# Create a new binary bin1 and add into Cargo.toml
# The following will create a file bin1.rs with a default main in current folder.
# And a [[bin]] will be added into the Cargo.toml
cargo bin new bin1 # or cargo bin new bin1.rs
```