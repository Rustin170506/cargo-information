# Cargo Information

This project is a response to [issue #948](https://github.com/rust-lang/cargo/issues/948) on the Rust Lang Cargo repository.

[![asciicast](https://asciinema.org/a/624572.svg)](https://asciinema.org/a/624572)

## Description

`cargo-info` is a tool that provides detailed information about a Rust package. It fetches data from the package's `Cargo.toml` file and presents it in a user-friendly format.

## Features

- Works with all registries that are compatible with Cargo
- Fetches and displays basic package information (name, version, owners, etc.)
- Shows package dependencies and their versions
- Provides information about package features

## Installation

### Cargo

To install `cargo-info`, run the following command:

```bash
cargo install cargo-information
```

or if you want to install the latest version from the main branch:

```bash
cargo install cargo-information --git https://github.com/hi-rustin/cargo-information.git
```

This library depends on OpenSSL. You may need to install OpenSSL on your system before installing `cargo-info`. Or you can use the `vendored-openssl` feature to use the vendored version of OpenSSL:

```bash
cargo install cargo-information --features vendored-openssl
```

### Arch Linux

`cargo-info` can be installed from the [AUR](https://aur.archlinux.org/packages?O=0&SeB=nd&K=cargo-information&outdated=&SB=p&SO=d&PP=50&submit=Go) using an [AUR helper](https://wiki.archlinux.org/title/AUR_helpers). For example:

```bash
paru -S cargo-information
```

## Usage

After installation, you can use the `cargo info` command followed by the package name to get information about a package:

```console
$ cargo info --help
Display info about a package in the registry

Usage: cargo info [OPTIONS] <SPEC>

Options:
      --index <INDEX>        Registry index URL to search packages in
      --registry <REGISTRY>  Registry to search packages in
  -v, --verbose...           Use verbose output (-vv very verbose/build.rs output)
  -q, --quiet                Do not print cargo log messages
      --color <WHEN>         Coloring: auto, always, never
      --config <KEY=VALUE>   Override a configuration value
  -Z <FLAG>                  Unstable (nightly-only) flags to Cargo, see 'cargo -Z help' for details
  -h, --help                 Print help

Package Selection:
  <SPEC>  Package to inspect

Manifest Options:
      --frozen   Require Cargo.lock and cache are up to date
      --locked   Require Cargo.lock is up to date
      --offline  Run without accessing the network

Run `cargo help info` for more detailed information.

```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the [MIT License](./LICENSE).
