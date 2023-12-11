# Introduction

## The interactive mode

When p2sh is executed without any arguments, it enters interactive mode,
allowing users to execute commands on the fly.

```
The p2sh Programming language v0.4.0
Ctrl+D to quit
>> puts("Hello, World!")
Hello, World!
```

## Using p2sh as a calculator

```
>> 2 + 2
4
>> (50 - 5 * 6) / 4
5
>> 10 / 3               # Integer division
3
>> 10 % 3               # Integer modulo
1
>> 10 / 3.              # Floating point division
3.3333333333333335
>> 10. / 3              # Floating point division
3.3333333333333335

>> 10 / 0
[line 1] Runtime error: Division by zero.
```

## Defining variables

Variables need to be defined before use.

```
n = 1
[line 1] compile error: undefined identifier 'n'
```

### The let keyword

```
>> let n = 10;
10
>> n
10
>> let s = "Hello";
"Hello"
```

## Comments

```
>> # this is a comment
>> // this is also a comment
>> n // n is not part of the comment
10
>> n # n is not part of the comment
10
>> 
```

## Arrays

```
>> let a = [1, 2, 3, 4, 5]
[1, 2, 3, 4, 5]

>> a[0]
1
```

### Out of bound array access

```
>> a[5]
[line 1] Runtime error: IndexError: array index out of range.

>> get(a, 5)             # A null result is not echoed

>> get(a, 5) == null
true

>> get(a, 1)
2
```

## Maps

Note that maps are not ordered. Note the `map` keyword before the curly braces.

```
>> let m = map {"a": 1, "b": 2, "c": 3}
map {"b": 2, "a": 1, "c": 3}

>> map["b"]
1
```

### Access unknown keys

```
>> m["d"]
[line 1] Runtime error: KeyError: key not found.

>> get(m, "d")
>> get(m, "a")
1
```

## Some programming

```
let a = 0; let b = 1; let t = 0;
0

>> while a < 1000 { print("{},", a); t = a; a = b; b = t + b; } puts();
0,1,1,2,3,5,8,13,21,34,55,89,144,233,377,610,987,
```
