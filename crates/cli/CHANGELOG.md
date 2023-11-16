# Changelog

This is the changelog for `hlbc-cli`, other crates have their own changelog.
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased](https://github.com/Gui-Yom/hlbc/compare/v0.7.0...HEAD)

## [0.7.0](https://github.com/Gui-Yom/hlbc/compare/v0.6.1...v0.7.0) - 2023-11-16

Basically nothing changed. Just keeping version in line with the core crate.

## [0.6.0](https://github.com/Gui-Yom/hlbc/compare/v0.5.0...v0.6.0) - 2023-05-07

### Changed

- Updated to latest hlbc

## [0.5.0](https://github.com/Gui-Yom/hlbc/compare/v0.4.0...cli-v0.5.0) - 2021-09-15

### New

- Updated to latest `hlbc-decompiler`

## [0.4.0](https://github.com/Gui-Yom/hlbc/compare/v0.3.0...v0.4.0) - 2021-08-03

### New

- Updated to `hlbc` 0.4 ([Changelog](../CHANGELOG.md))
- Use `hlbc-decompiler`

### Fixed

- `callgraph` command parsing wasn't working

## [0.3.0](https://github.com/Gui-Yom/hlbc/compare/v0.2.0...v0.3.0) - 2022-07-31

### Added

- Add explain command to show an opcode description
- Input many commands separated with a `;`
- Execute a command on startup with the `-c` flag
- Auto-reload with the `-w` flag
- Add a proper cli parser and app with clap
- Support passing a haxe source file to be compiled on the fly
- New `decomp` command to decompile a function
- New `decomptype` command to decompile a class
- New `wiki` command to open the bytecode wiki page in a browser

### Changed

- Callgraph generation is now a default feature
