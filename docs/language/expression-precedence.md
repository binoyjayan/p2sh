# Expression precedence

The operators are listed in order of precedence, from the highest to the lowest.

| Operators / Expression     | Associativity |
|----------------------------|---------------|
| [] . ()                    | left to right |
| ! - ~ (Unary)              | left to right |
| * / %                      | left to right |
| + -                        | left to right |
| << >>                      | left to right |
| &                          | left to right |
| ^                          | left to right |
| \|                         | left to right |
| == != < > <= >=            | left to right |
| &&                         | left to right |
| \|\|                       | left to right |
| .. ..=                     | left to right |
| \| (in match pattern)      | left to right |
| =                          | right to left |

