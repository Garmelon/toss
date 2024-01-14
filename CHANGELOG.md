# Changelog

All notable changes to this project will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

Procedure when bumping the version number:
1. Update dependencies in a separate commit
2. Set version number in `Cargo.toml`
3. Add new section in this changelog
4. Commit with message `Bump version to X.Y.Z`
5. Create tag named `vX.Y.Z`
6. Push `master` and the new tag

## Unreleased

### Fixed
- Crash when drawing `widgets::Predrawn` with width 0

## v0.2.1 - 2024-01-05

### Added
- `Frame::set_title`
- `WidgetExt::title`
- `widgets::title`

## v0.2.0 - 2023-08-31

### Changed
- **(breaking)** Updated dependencies

## v0.1.0 - 2023-05-14

Initial release
