# Changelog

This is the changelog for `hlbc-decompiler`, other crates have their own changelogs.
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased](https://github.com/Gui-Yom/hlbc/compare/v0.7.0...HEAD)

## [0.7.0](https://github.com/Gui-Yom/hlbc/compare/v0.6.1...v0.7.0) - 2023-11-16

Basically nothing changed. Just keeping version in line with the core crate.

## [0.6.0](https://github.com/Gui-Yom/hlbc/compare/v0.5.0...v0.6.0) - 2023-05-07

### Changed

- Updated to latest hlbc

### Fixed

- Fix `trace` ast post-processing step

## [0.5.0](https://github.com/Gui-Yom/hlbc/compare/v0.4.0...v0.5.0) - 2021-09-15

### Changed

- AST visitor now makes a single pass
- Restore `trace` calls
- Do not show object ids (findex and type index) in decompilation output

## [0.4.0](https://github.com/Gui-Yom/hlbc/compare/v0.3.0...v0.4.0) - 2021-08-03

### Added

- If and else statements are unified for better formatting and easier post-processing
- New (currently hidden) ast post-processing step (AST-PP) to improve the decompiler output
- New AST-PP : if-expressions
- New AST-PP : string concatenations :(`__add__("a", "b")` to `"a" + "b"`)
- New AST-PP : Hide calls to itos. Int to strings conversions are usually hidden.
- Ability to generate comments in the AST
- Display closure if InstanceClosure on an enum (the enum is the closure capture)

### Fixed

- Remove excessive `;` in constructor calls

---

Previous releases of the decompiler can be found in the `hlbc` crate changelog.
