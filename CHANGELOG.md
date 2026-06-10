# Changelog

## [2026.6.10] - 2026-06-10

### Changed
- **4.2 Path Modernization**: Updated path imports to align with the `library` 4.2 restructured API (using simplified flat namespaces `apps` and `toolkit`).
- **AppData Directory Realignment**: Moved user configuration, database, and log files into a nested %APPDATA%\local76\app\pulse structure to organize the ecosystem's configuration space.
- **Repository Rename**: Renamed repository and local directory to app-pulse for cleaner ecosystem taxonomy.

## [2026.6.9] - 2026-06-09

### Renamed
- **Project rename**: `pulse` was previously `pulse-App` / `rMonitor`. The Cargo package name, binary name, file paths, registry keys, and docs are now lowercase `pulse`. Behavior and features are unchanged.

### Refactored
- **App Blueprint alignment**: Re-architected directory and module tree to standard App layout. Moved `src/app.rs` to `src/app/mod.rs` and `src/event_handler.rs` to `src/app/keys.rs`, and created `src/app/mouse.rs`. Created `src/backend/mod.rs` to dispatch system telemetry queries and moved `src/win32.rs` to `src/backend/win32.rs`. Modularized panels into `src/ui/` by renaming `src/panels/cards.rs` to `src/ui/cards.rs`, `src/panels/details.rs` to `src/ui/widgets.rs`, `src/panels/processes.rs` to `src/ui/processes.rs`, and `src/modals.rs` to `src/ui/overlays.rs`.

### Changed
- README rewritten in the new register: live monitor feature list, install matrix, CLI flags, configuration, build instructions, license.
- Drop the legacy "r*" and "Local freedom" branding throughout.
- Drop the per-repo `rApps` umbrella and `build_all.ps1` from this repo; build orchestration lives in [`toolkit`](https://github.com/local76/toolkit).

## [3.1.0] - 2026-06-08

### Changed
- Renamed project back from `pulse-App` to `pulse` (crate name: `pulse`, binary name: `pulse`).
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