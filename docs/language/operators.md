# Operators

## Arithmetic Operators

| Name | Description |
|------|-------------|
| + | Addition (works with numeric and string types) |
| - | Subtraction (works with numeric types) |
| * | Multiplication (works with numeric types) |
| / | Division (works with numeric types) |
| % | Modulo (works with numeric types) |

## Logical Operators

| Name | Description |
|------|-------------|
| && | The logical 'and' operator |
| \|\| | The logical 'or' operator |
| ! | The logical 'or' operator |

The result of the logical operations depend on the truthiness of
the operands of these operators.

## Truthiness

| Value | Truthiness |
|-------|------------|
| false | falsey |
| 0 | falsey |
| null | falsey |
| 0.0 | falsey |
| '\0' | falsey |
| b'\0' | falsey |
| "" | falsey |
| [] | falsey |
| map {} | falsey |
| everything else | truthy |


## Relational Operators

| Name | Description |
|------|-------------|
| == | Equality |
| != | Non equality |
| < | Less than |
| > | Greater than |
| <= | Less than or equal to |
| >= | Greater than or equal to |

## Bitwise Operators

| Name | Description |
|------|-------------|
| & | Performs 'and' operations on each bit |
| \| | Performs 'or' operations on each bit |
| ^ | Performs 'xor' operations on each bit |
| ~ | Flips all bits |
| << | Arithmetic left shift |
| >> | Arithmetic right shift |

## Assignment

Only one operator is currently supported for assignments.

| Name | Description |
|------|-------------|
| = | Assignment |

