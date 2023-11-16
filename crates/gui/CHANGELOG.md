# Changelog

This is the changelog for `hlbc-gui`, other crates have their own changelog.
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased](https://github.com/Gui-Yom/hlbc/compare/gui-v0.3.0...HEAD)

## [0.3.0](https://github.com/Gui-Yom/hlbc/compare/gui-v0.2.1...gui-v0.3.0) - 2023-11-16

### Added

- Full text search with `hlbc-indexing`.
- Status bar
- View any type in the inspector
- Clickable register types in inspector
- Opcode description on hover in inspector
- Navigation history for the sync inspector

### Changed

- Better gui styling

## [0.2.1](https://github.com/Gui-Yom/hlbc/compare/gui-v0.2.0...gui-v0.2.1) - 2023-05-13

### Added

- String inspector text is now selectable

### Fixed

- 2 crashes related to formatting bytecode (fixed in core *hlbc* crate)

## [0.2.0](https://github.com/Gui-Yom/hlbc/compare/gui-v0.1.0...gui-v0.2.0) - 2023-05-07

### Added

- `hlbc-gui` now runs on the web !

### Changed

- Load bytecode on a background thread instead of blocking the ui.
- Updated to latest hlbc, it now works with the latest hashlink version

## [0.1.0](https://github.com/Gui-Yom/hlbc/compare/v0.4.0...gui-v0.1.0) - 2021-09-15
