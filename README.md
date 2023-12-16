# p2sh

![Rust](https://github.com/binoyjayan/p2sh/actions/workflows/rust.yml/badge.svg)

🦀 🦀 🦀 Interpreter for the p2sh programming language 🦀 🦀 🦀

The project served as a way for me to learn Rust, so many parts of the code may not be idiomatic.

## Dependencies

The p2sh interpreter is compiled using a Rust compiler v1.65.0. Any version later than that should work.
Please visit the Rust website for installation.

## Build and test

### Run tests

It has decent test coverage for all the modules, including tests for
the scanner, parser, opcode definitions, compiler, virtual machine,
and built-ins.

```bash
cargo test
```

### Run examples

```bash
cargo run --release examples/formatting/formatted-output.p2
cargo run --release examples/algorithms/fibonacci-recursive.p2
cargo run --release examples/algorithms/fibonacci-iterative.p2
```

### Run the REPL

Run an interactive REPL loop to execute program statements
on the terminal one at a time and see the output immediately.

```bash
cargo run --release
```
```
>> 1 + 2
3
```


### Create a release build

Build release with default options

```bash
cargo build --release
```

This will create a release binary in the './target/release' directory.

### Additional build options

#### debug_print_code

The option helps building source with source code disassembly enabled.

Build with the option:

```bash
cargo build --release --features  'debug_print_code'
```

Run the REPL:

```bash
cargo run --release --features  'debug_print_code'
```
```
...

>> 1 + 2
--------- Instructions [len: 8   ] -------------------
0000 OpConstant 0
0003 OpConstant 1
0006 OpAdd
0007 OpPop
------------------------------------------------------
----------- Constants [len: 2   ] --------------------
[0] 1
[1] 2
------------------------------------------------------
3
```

Run an example:

```bash
cargo run --release --features  'debug_print_code' examples/fibonacci-recursive.p2
```

#### debug_trace_execution

This option helps build the release for tracing program execution and
examining the state of the stack.

```bash
cargo build --release --features  'debug_trace_execution'
```

## Installation

The p2sh intepreter can be installed by copying the binary to a directory
that is in your path such as `/usr/local/bin`. Once installed, it can be
executed by running it like so

```
p2sh script.p2 <args>
```

###  shebang

A p2sh script may also be executed using a shebang.
The first line of a p2sh script must be a line that looks like this:

```
#!/usr/bin/env p2sh
```

Then, the script may be run as:

```
./script.p2 <args>
```

OR

```
/path/to/script.p2 <args>
```

## More Information

- **Examples** may be found in the  [examples](./examples) directory
- **Documentation** can be found [here](./docs)
