# Changelog

This is the changelog for `hlbc` the library, `hlbc-cli` have its own [changelog](hlbc-cli/CHANGELOG.md).
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased](https://github.com/Gui-Yom/hlbc/compare/v0.2.0...HEAD)

### Added

- Get an Opcode description (generated from its doc comment) and create an Opcode from its name.
- Derive Default on a lot of types
- Global initializer map (global -> constant)
- Correctly handle bytes pool
- Store a reference to the parent type in the function struct

### Changed

- Callgraph generation is now a default feature
- Improve opcode display
- Make bytecode elements serialization and deserialization functions public
- Global function indexes are resolved through a vec instead of a map
