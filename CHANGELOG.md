# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

### Changed
- **Renamed project** from `rMonitor` to `rMonitor-tui`. The GitHub repository, Cargo package name, binary name, and all user-facing labels now use the `-tui` suffix to make the program's role as a terminal user interface explicit (matching `rTemplate-tui`).
  - Repository: `local76/rMonitor` → `local76/rMonitor-tui`
  - Crate/binary: `rmonitor` → `rmonitor-tui`
  - Config file: `%APPDATA%\rmonitor\config.yaml` → `%APPDATA%\rmonitor-tui\config.yaml`
  - Log file: `%APPDATA%\rmonitor\rmonitor.log` → `%APPDATA%\rmonitor-tui\rmonitor-tui.log`
  - Linux package names: `rmonitor` → `rmonitor-tui`

## [3.0.1] - 2026-06-06
### Added
- Added author and maintainer metadata for packaging.

## [3.0.0] - 2026-06-06
### Changed
- Renamed organization to `local76`.
- Renamed executable from `rtem` to `rmonitor`.
- Reorganized directory structure to group packaging files inside `dist/packages/`.