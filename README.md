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
    <img src="crates/cli/screenshot.png" width="318" height="215">
    <img src="crates/gui/screenshot.png" width="354" height="249">
</div>

## Repository structure

- `data/` : Haxe source files to test the tools
- `crates/hlbc/` : Core library to load and disassemble bytecode
- `crates/cli/` : CLI frontend for `hlbc`
- `crates/decompiler/` : Decompiler library
- `crates/derive/` : helper proc macros for hlbc
- `crates/gui/` : GUI to explore bytecode visually

## Wiki

A wiki detailing the specifics of Hashlink bytecode is available [here](https://github.com/Gui-Yom/hlbc/wiki) or by
using the command `wiki` in the CLI.

## Planned

- C API

## Credits

Development of this project would not have been possible without
the [hashlink](https://github.com/HaxeFoundation/hashlink) source code. Some algorithms are directly derived from the
original C code reading bytecode files.
