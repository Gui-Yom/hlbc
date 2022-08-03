# Changelog

This is the changelog for `hlbc-decompiler`, other crates have their own changelogs.
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased](https://github.com/Gui-Yom/hlbc/compare/v0.4.0...HEAD)

## [0.4.0](https://github.com/Gui-Yom/hlbc/compare/v0.3.0...v0.4.0) - 2021-08-03

### Added

- If and else statements are unified for better formatting and easier post-processing
- New (currently hidden) ast post-processing step (AST-PP) to improve the decompiler output
- New AST_PP : if-expressions
- New AST-PP : string concatenations :(`__add__("a", "b")` to `"a" + "b"`)
- New AST-PP : Hide calls to itos. Int to strings conversions are usually hidden.
- Ability to generate comments in the AST
- Display closure if InstanceClosure on an enum (the enum is the closure capture)

### Fixed

- Remove excessive `;` in constructor calls

---

Previous releases of the decompiler can be found in the `hlbc` crate changelog.
