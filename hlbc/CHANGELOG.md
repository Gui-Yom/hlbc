# Changelog

This is the changelog for `hlbc`, other crates have their own changelog.
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased](https://github.com/Gui-Yom/hlbc/compare/v0.3.0...HEAD)

### Changed

- The decompiler (`hlbc::decompiler`) has been moved to its own crate

## [0.3.0](https://github.com/Gui-Yom/hlbc/compare/v0.2.0...v0.3.0) - 2022-07-31

### Added

- Get an Opcode description (generated from its doc comment) and create an Opcode from its name.
- Derive Default on a lot of types
- Global initializer map (global -> constant)
- Correctly handle bytes pool
- Store a reference to the parent type in the function struct

#### Decompiler

- Handle expressions and statements
- Generate code with proper indentation
- Handle branches and while loops
- Handle early returns, constructors, closures and methods
- break and continue statements
- Partial result with \[missing expr]
- Initial support for primitive array accesses
- Decompile whole classes
- Anonymous structures
- Initial support for enums
- Initial support for switch

### Changed

- Callgraph generation is now a default feature
- Improve opcode display
- Make bytecode elements serialization and deserialization functions public
- Global function indexes are resolved through a vec instead of a map
- Return a custom error type instead of using anyhow
