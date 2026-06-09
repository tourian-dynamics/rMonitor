# Changelog

All notable changes to this project will be documented in this file.

## [3.1.0] - 2026-06-08

### Changed
- Renamed project back from `pulse-tui` to `pulse` (crate name: `pulse`, binary name: `pulse`).
- Split monolithic `src/panels.rs` (647 lines) into modular files in `src/panels/` subdirectory, ensuring all source files are strictly under 500 lines.
- Suppressed unused/deprecated compiler warnings to achieve a clean compilation.

## [3.0.1] - 2026-06-06
### Added
- Added author and maintainer metadata for packaging.

## [3.0.0] - 2026-06-06
### Changed
- Renamed organization to `local76`.
- Renamed executable from `rtem` to `pulse`.
- Reorganized directory structure to group packaging files inside `dist/packages/`.