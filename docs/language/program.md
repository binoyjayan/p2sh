# Program structure

A p2sh program must be written entirely in a single file. Although it
supports functions and closures, the program typically starts execution
from the top of the file, outside of any function definitions.

It supports various modes of operation.

## The script mode

This is the default mode of operation, and it is what happens when the
interpreter is run with a file as an argument. The script can also
accept additional arguments from the command line. The interpreter then
executes the script from top to bottom, unless there are control
statements, in which case, it will act accordingly.

The script mode can be invoked as:

```bash
p2sh <script.p2>
```

OR if a shebang is used at the top of the script,

```bash
./script.p2
```

## The REPL

The REPL mode is an interactive CLI interface that allows users to type
program statements into a CLI interface and see immediate output. When
run without any arguments, the interpreter defaults to using a REPL

```bash
p2sh

>> 1 + 2
3
```

## The command mode

The command mode allows users to execute statements as commands through
command-line arguments.

```bash
p2sh -c1+2
3
```

## The filter mode

This represents a more advanced usage of the interpreter. In this mode,
the script executes from top to bottom initially. Additionally, it
features specialized filter statements. For more details about this
mode, refer to [filters](./filters.md).







