# Change Log

<!-- next-header -->

## [Unreleased] - ReleaseDate

## [0.4.0] - 2024-01-05

### Added

- Prioritize selection of Minimum Supported Rust Version (MSRV) compatible crate versions. The following hierarchy is used to determine the MSRV:
  - First, the MSRV of the parent directory package is checked, if it exists.
  - If the parent directory package does not specify an MSRV, the minimal MSRV of the workspace is checked.
  - If neither the workspace nor the parent directory package specify an MSRV, the version of the current Rust compiler (`rustc --version`) is used.

### Changed

- Dependency updates and patches, including updates to rust crate cargo(0.74.0), clap(4.4.12), anyhow(1.0.79) and semver(1.0.21).

## [0.3.0] - 2023-12-20

### Added

- Allow to inspect a package from the local workspace.

### Fixed

- Generate `Cargo.lock` file if it does not exist and pick the correct matching version.

## [0.2.1] - 2023-12-11

### Fixed

- Do now allow empty spec name.
- Bail out an error for non remote registry.

## [0.2.0] - 2023-12-06

### Added

- Support for version selection in package specification.
- Support for custom registry.
- Pretty owners feature for listing package owners in a more readable format.
- Enhanced CLI with global options in the info command and added subcommands to make it as a Cargo subcommand.
- Corrected various issues related to feature detection, display, and handling.
- Fixed issues related to offline and frozen argument handling.
- Cargo tree suggestion feature.
- Basic information retrieval and display features.

### Changed

- Refactored various parts of the code for clarity and efficiency, including extra views, pretty printing methods, and use of package APIs.
- Various test cases for URL package specifications, registry matching, workspace inclusion, and lockfile consistency.
- Dependency updates and patches, including updates to rust crate clap (4.4.9, 4.4.10, 4.4.11) and cargo-util (0.2.7).

### Removed

- Removal of unused or redundant dependencies and files for cleaner codebase.

### Chore

- General maintenance updates, including clippy fixes and setting the Rust version.
- Continuous integration improvements with GitHub actions and caching.

### Documentation

- Updates to README for better user guidance.
- Documentation updates including asciicast demo and mention of registries support.
