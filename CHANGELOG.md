# Change Log

<!-- next-header -->

## [Unreleased] - ReleaseDate

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
