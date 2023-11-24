# Cargo Information

This project is a response to [issue #948](https://github.com/rust-lang/cargo/issues/948) on the Rust Lang Cargo repository.

## Description

`cargo-info` is a tool that provides detailed information about a Rust package. It fetches data from the package's `Cargo.toml` file and presents it in a user-friendly format.

## Features

- Fetches and displays basic package information (name, version, authors, etc.)
- Shows package dependencies and their versions
- Provides information about package features

## Installation

To install `cargo-info`, run the following command:

```bash
cargo install cargo-information ---git https://github.com/hi-rustin/cargo-information.git
```

## Usage

After installation, you can use the `cargo info` command followed by the package name to get information about a package:

```bash
cargo info <package-name>
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the [MIT License](./LICENSE).
