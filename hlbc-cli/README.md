# hlbc-cli

A CLI to navigate through Hashlink bytecode files.

## Commands

- `info` General information about the bytecode
- `help` Help message
- `entrypoint` Get the bytecode entrypoint
- `i|int <idx>` Get the int at index
- `f|float <idx>` Get the float at index
- `s|string <idx>` Get the string at index
- `fstr <str>` Find a string
- `d|debugfile <idx>` Get the debug file name at index
- `ffile <str>` Find the debug file named
- `t|type <idx>` Get the type at index
- `g|global <idx>` Get global at index
- `c|constant <idx>` Get constant at index
- `n|native <idx>` Get native at index
- `fnh <findex>` Get header of function (findex)
- `fn <findex>` Get function (findex)
- `fname <str>` Get function named
- `infile <@idx|str>` Find functions in file
- `fileof <findex>` Get the file where findex is defined
- `refto <any@idx>` Find references to a given bytecode element
- `saveto <filename>` Serialize the bytecode to a file
- `callgraph <findex> <depth>` Create a dot call graph from a function and a max depth

## Indexes

In most of the commands that accept an index, you can pass a range too : `a..b`, `..b`, `a..`, `a..=b` ...
