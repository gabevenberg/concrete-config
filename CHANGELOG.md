# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/2.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Support for tuple structs

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.2.0] - 2026-07-06

### Added

- `&'static [T]` slice fields, for any supported element type.
- Tuple fields, with per-element types (e.g. `(u8, &'static str)`).

## [0.1.0] - 2026-07-03

Initial release.

### Added

- `#[concrete_toml("...")]` attribute macro that fills a `const` struct from a
  TOML file at compile time.
- User-defined structs (`#[root]` marks the root table) and unit enums.
- All integer and float widths, with range checks against the target type.
- Booleans and `&'static str`.
- Fixed-size arrays of any supported type.

[Unreleased]: https://github.com/gabevenberg/concrete-config/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/gabevenberg/concrete-config/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/gabevenberg/concrete-config/releases/tag/v0.1.0
