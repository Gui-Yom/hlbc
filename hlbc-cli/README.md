# hlbc-cli

![Crates.io](https://img.shields.io/crates/v/hlbc-cli?label=hlbc-cli)

A CLI to navigate through [Hashlink](https://hashlink.haxe.org/) bytecode files.

## Usage

`hlbc-cli <file> [-c <command>] [-w <command>]`

You get access to a prompt where you can enter commands.

You can execute commands on startup with the `-c` switch.
e.g. Dump all strings from the bytecode then exit : `hlbc-cli main.hl -c "s ..; exit"`.
If you omit the `exit` command, the app will simply launch the normal prompt after executing the startup commands.

With `-w`, the given command will execute each time the file changes. The cli won't show a command prompt.

## Commands

- `info` General information about the bytecode
- `help` Help message
- `entrypoint` Get the bytecode entrypoint
- `explain <op>` Get information about an opcode
- `i|int <idx>` Get the int at index
- `f|float <idx>` Get the float at index
- `s|string <idx>` Get the string at index
- `sstr <str>` Find a string
- `d|debugfile <idx>` Get the debug file name at index
- `sfile <str>` Find the debug file named
- `t|type <idx>` Get the type at index
- `g|global <idx>` Get global at index
- `c|constant <idx>` Get constant at index
- `n|native <idx>` Get native at index
- `fnh <findex>` Get header of function (findex)
- `fn <findex>` Get function (findex)
- `sfn <str>` Get function named
- `infile <idx|str>` Find functions in file
- `fileof <findex>` Get the file where findex is defined
- `refto <any@idx>` Find references to a given bytecode element
- `saveto <filename>` Serialize the bytecode to a file
- `callgraph <findex> <depth>` Create a dot call graph from a function and a max depth

### Indexes

In most of the commands that accept an index, you can pass a Rust style range too : `a..b`, `..b`, `a..`, `a..=b`, `..`.
Where `..10` means '*select the first 10 items*' and `..` means '*display everything*'.
