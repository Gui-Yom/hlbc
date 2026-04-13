# hlbc [![Crates.io](https://img.shields.io/crates/v/hlbc?label=hlbc)](https://crates.io/crates/hlbc)

[**H**ash**l**ink](https://hashlink.haxe.org/) **b**yte**c**ode disassembler and analyzer.

*This crate is a library, see [hlbc-cli](https://crates.io/crates/hlbc-cli) for an actual program to use.*

---

## Features

- Parse the whole bytecode file or any bytecode element
- Display any bytecode element
- Restore all possible names
- Link elements between them (with manual references for flexibility)
- Link elements with their debug information
- Serialize bytecode back to bytes
- Decompiler to haxe source code for classes and functions
- Commandline interface to use the features of this library ([hlbc-cli](https://crates.io/crates/hlbc-cli))

## Planned features

- Properly test serialization, ensure we get a byte-to-byte equivalent.
- Integrate with the Haxe/Hashlink standard library to restore more names, exclude them from analysis and such
- C API to integrate with other tools
- Text search engine to search for strings and names
- Assemble and inject bytecode or inject haxe source code directly

## Changelog

See [CHANGELOG.md](CHANGELOG.md).

## Decompiler

The decompiler is present in the `hlbc-decompiler` crate and is currently incomplete (and might never be 100% complete).
[See the wiki](https://github.com/Gui-Yom/hlbc/wiki/Decompilation) for examples of decompilation output.

## Wiki

A wiki detailing the specifics of Hashlink bytecode is available [here](https://github.com/Gui-Yom/hlbc/wiki).
The wiki also details the specifics of Haxe to hashlink compilation and decompilation.

## Alternatives

This library is made in Rust, a C API is in the works which could permit using this lib in many other projects, but for
now it is only Rust friendly.

Other alternatives include :

- Tinkering directly with the [hashlink](https://github.com/HaxeFoundation/hashlink) source code in C (my first attempt
  was doing this, but it turned out that everything in C is pain in the ass)
- Using the in-progress [**_*dashlink*_**](https://github.com/Steviegt6/dashlink) made in Haxe but probably compilable
  to many other languages (seems broken as of 10/13/24)
- Using [crashlink](https://github.com/N3rdL0rd/crashlink), written in pure Python and portable to many platforms
  (also supports bytecode patching)
- Using the also in-development [hlbc-python](https://github.com/N3rdL0rd/hlbc-python) to access a (very small) subset of this crate from Python (not reccomended, use crashlink instead)
- Using [Haxelink](https://github.com/LebiFur/Haxelink), a C# implementation

## Macros

There are 98 different bytecode instructions, there is no way I manually write code for them. Most of the serialization
code for these opcodes is generated through a proc macro (see [hlbc-derive](../derive)). Making that a declarative macro
would probably turn out really hard. The only time I needed to write 98 different branches was for the formatting used
when displaying the instructions ([src/fmt.rs](src/fmt.rs)).
