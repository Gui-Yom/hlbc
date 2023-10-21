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

## About

[Hashlink](https://hashlink.haxe.org/) is a VM used as a compilation target for the Haxe language. Hashlink is
successfully used to run popular games from developer [**Shiro Games**](https://shirogames.com/) like **Northgard**, *
*Dune: Spice Wars** and **Wartales**.

*hlbc* intends to help the motivated to develop mods and tools for those games.

Those games are packaged with the following model :

- `<game>.exe`: A very light executable that contains the Hashlink VM
- `hlboot.dat`: The actual bytecode file Hashlink is configured to load on startup. This is the file you want to load in
  *hlbc*. It doesn't contain any game assets, this is just code.
- `sdlboot.dat`: This is the same code, except it fallbacks to SDL.
- `<lib>.hdll`: This is your average native code dll, except it can work with the VM.

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

- Finishing the decompiler (for loops mainly)
- C API

## Credits

Development of this project would not have been possible without
the [hashlink](https://github.com/HaxeFoundation/hashlink) source code. Most of the deserialization code is directly
adapted from the original C code reading bytecode files.

## Why Rust

I should probably have used Haxe in the first place, that would have been the logical choice as tooling for a language
is best done (I suppose) in that same language. But Rust makes it a joy to develop with its enums, match statements and
macros (I think those features are present in Haxe too, although I'm not at all familiar with this language).
Also, the Rust ecosystem feels much more alive.

One of the downside of using Rust here is that I can't pass references everywhere. The bytecode is a large graph where
every element can reference another, this by definition does not play well with Rust borrow-checking rules. To cope with
this, bytecode handling is working with the arena pattern. The `Bytecode` struct owns every element and we use indexes (
wrapped in custom types) throughout the codebase. This might be a bit cumbersome to pass `code: &Bytecode` and
calling `code.get()` everywhere but it works without ever dealing with lifetimes anywhere.
