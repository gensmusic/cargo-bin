# cargo-bin

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