# The p2sh Language Reference

This document serves as a reference to the language and describes the syntax and core semantics of the language.
For an informal introduction to the language refer to the [tutorial](../tutorial/README.md)

Note that this is a work in progress.

## Language constructs

- [Program structure](./program.md)
- [Keywords](./keywords.md)
- [Data model](./data-model.md)
- [Operators](./operators.md)
- [Expression precedence](./expression-precedence.md)
- [Builtin Functions](./builtins.md)
- [Builtin Functions for packet processing ](./builtins-packet.md)
- [Property expressions](./property.md)

## Feature roadmap

  - Macros
  - Modules
  - DFA-based Scanner
  - Intermediate Representation
  - Register-Based Virtual Machine
  - Operations on Strings
  - String Escape Sequences and Raw Strings
  - Iterators and For Loops
  - Structs and Traits
  - Error Handling Operator (?)
  - Range Overlaps and Exhaustiveness Checks in Match Expressions
  - Utilize Variable Stack Size Beyond 4k
