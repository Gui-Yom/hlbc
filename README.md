<div align="center">
    <h1><b>H</b>ash<b>l</b>ink <b>b</b>yte<b>c</b>ode tools</h1>
    <a href="https://crates.io/crates/hlbc">
        <img src="https://img.shields.io/crates/v/hlbc?label=hlbc">
    </a>
    <a href="https://crates.io/crates/hlbc-decompiler">
        <img src="https://img.shields.io/crates/v/hlbc-decompiler?label=hlbc-decompiler">
    </a>
    <a href="https://crates.io/crates/hlbc-cli">
        <img src="https://img.shields.io/crates/v/hlbc-cli?label=hlbc-cli">
    </a>
    <a href="https://crates.io/crates/hlbc-gui">
        <img src="https://img.shields.io/crates/v/hlbc-gui?label=hlbc-gui">
    </a>
    <br/>
    This repository contains a collection of Rust crates and cli tools to load, disassemble, decompile and
    analyze <a href="https://hashlink.haxe.org/">Hashlink</a> bytecode.
    <br/>
    <img src="hlbc-cli/screenshot.png">
    <img src="hlbc-gui/screenshot.png">
</div>

## Repository structure

- `data/` : Haxe source files to test the tools
- `hlbc/` : Core library to load and disassemble bytecode
- `hlbc-cli/` : CLI frontend for `hlbc`
- `hlbc-decompiler/` : Decompiler library
- `hlbc-derive/` : helper proc macros for hlbc
- `hlbc-gui/` : GUI to explore bytecode visually

## Wiki

A wiki detailing the specifics of Hashlink bytecode is available [here](https://github.com/Gui-Yom/hlbc/wiki) or by
using the command `wiki` in the CLI.

## Planned

- C API

## Credits

Development of this project would not have been possible without
the [hashlink](https://github.com/HaxeFoundation/hashlink) source code. Some algorithms are directly derived from the
original C code reading bytecode files.
