# hlbc

![Crates.io](https://img.shields.io/crates/v/hlbc?label=hlbc)
![Crates.io](https://img.shields.io/crates/v/hlbc-cli?label=hlbc-cli)

[Hashlink](https://hashlink.haxe.org/) bytecode disassembler and analyzer

## CLI

See [hlbc-cli](hlbc-cli) for an actual program to use.

## Features

- Parse the whole bytecode file
- Get any bytecode element
- Disassemble functions
- Restore nearly all names possible
- Explore debug information

## Planned features

- Restore and analyze variable names
- Generate real code to use real tools on
- Gui to render the graph on in real time
- Gui to render everything
- C API to integrate with other tools
- Text search engine to search for strings and names

## Repository organisation

- *root* : hlbc (lib)
- `hlbc-derive/` : hlbc-derive, helper proc macros for hlbc
- `hlbc-cli/` : CLI using hlbc

## Credits

Development of this crate wouldn't be possible without the [hashlink](https://github.com/HaxeFoundation/hashlink) source
code. Some algorithms are directly derived from the original C code reading bytecode files.
