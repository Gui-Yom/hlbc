# hlbc ![Crates.io](https://img.shields.io/crates/v/hlbc?label=hlbc) ![Crates.io](https://img.shields.io/crates/v/hlbc-cli?label=hlbc-cli)

[**H**ash**l**ink](https://hashlink.haxe.org/) **b**yte**c**ode disassembler, analyzer, decompiler and assembler.

*This crate is a library, see [hlbc-cli](hlbc-cli) for an actual program to use.*

---

## Features

- Parse the whole bytecode file or any bytecode element
- Display any bytecode element
- Restore all possible names
- Link elements between them (references are manual for flexibility)
- Link elements with their debug information
- Serialize bytecode back to bytes

## Planned features

- Handcrafted error types instead of `anyhow::Result`
- Restore and analyze variable names
- Integrate with the Haxe/Hashlink standard library to restore more names, exclude them from analysis and such
- Decompile parts of the code to potentially working haxe code (much easier to understand than individual instructions)
- Gui to render the callgraph in real time
- Gui to render instructions and decompiled code
- C API to integrate with other tools
- Text search engine to search for strings and names

## Changelog

See [CHANGELOG.md](CHANGELOG.md).

## Repository structure

- `/` : hlbc (lib)
- `hlbc-derive/` : hlbc-derive, helper proc macros for hlbc
- `hlbc-cli/` : CLI using hlbc
- `data/` : Haxe source files to test the decompiler

## Macros

There are 98 different bytecode instructions, there is no way I manually write code for it each time. Most of the code
for these opcodes is generated through a proc macro (see [hlbc-derive](/hlbc-derive)).
The only time I needed to write 98 different branches was for the formatting used when displaying the
instructions ([src/fmt.rs](src/fmt.rs)).

## Wiki

A wiki detailing the specifics of Hashlink bytecode is available [here](https://github.com/Gui-Yom/hlbc/wiki).

## Credits

Development of this crate wouldn't be possible without the [hashlink](https://github.com/HaxeFoundation/hashlink) source
code. Some algorithms are directly derived from the original C code reading bytecode files.

## Alternatives

This library is made in Rust, a C API is in the works which could permit using this lib in many other projects, but for
now it is only Rust friendly.

Other alternatives include :

- Tinkering directly with the [hashlink](https://github.com/HaxeFoundation/hashlink) source code in C
- Using the in-progress [**_*dashlink*_**](https://github.com/Steviegt6/dashlink) made in Haxe but probably compilable
  to many other languages.
