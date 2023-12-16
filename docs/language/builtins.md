# Builtins

## Builtin functions

The following table lists the builtin functions available.

| Name | Description |
|------|-------------|
| [**len**](#len) | Find the length of a string, an array, or a map |
| [**puts**](#puts) | Display a comma-separated list of objects |
| [**first**](#first) | Get the first element in an array |
| [**last**](#last) | Get the last element in an array |
| [**rest**](#rest) | Get all but the first element in an array |
| [**push**](#push) | Add an element to the end of an array |
| [**pop**](#pop) | Remove an element from the end of an array |
| [**get**](#get) | Get an array element or a value in map |
| [**contains**](#contains) | Check if a map contains a specific element |
| [**insert**](#insert) | Insert a key-value pair into a map |
| [**str**](#str) | Convert a value to a string |
| [**int**](#int) | Convert a value to an integer |
| [**float**](#float) | Convert a value to a floating-point number |
| [**char**](#char) | Convert a value to a character |
| [**byte**](#byte) | Convert a value to a byte |
| [**time**](#time) | Get the current time |
| [**exit**](#exit) | Exit the program |
| [**flush**](#flush) | Flush stdout, stderr or a file handle |
| [**format**](#format) | Format a string with format specifiers |
| [**print**](#print) | Display a string with format specifiers to stdout |
| [**println**](#println) | `print` followed by a line break |
| [**eprint**](#eprint) | Display a string with format specifiers to stderr |
| [**eprintln**](#eprintln) | `eprint` followed by a line break |
| [**round**](#round) | Round a floating-point number |
| [**sleep**](#sleep) | Sleep for a specified duration in seconds |
| [**tolower**](#tolower) | Convert a character, a byte or a string to lowercase |
| [**toupper**](#toupper) | Convert a character, a byte or a string to uppercase |
| [**open**](#open) | Open a file for reading or writing |
| [**read**](#read) | Read from a file or stdin |
| [**write**](#write) | Write to a file, stdout or stderr |
| [**read_to_string**](#read_to_string) | Read the contents of a file into a string |
| [**decode_utf8**](#decode_utf8) | Decode a UTF-8 byte sequence to a string |
| [**encode_utf8**](#encode_utf8) | Encode a string to a UTF-8 byte sequence |
| [**read_line**](#read_line) | Read a line from the standard input or a file |
| [**input**](#input) | Read a line from the standard input and return it as a string |
| [**get_errno**](#get_errno) | get last os error number |
| [**strerror**](#strerror) | convert an os error number to a string |
| [**is_error**](#is_error) | Check if an object is an error object |
| [**sort**](#sort) | Sort an object |
| [**chars**](#chars) | Convert a string to an array of chars |
| [**join**](#join) | Join an array of characters |

### Description

### <a name="len"></a>len
Find the length of a string, an array, or a map. Takes only one argument.

Example:
```
len("example string")
```
### <a name="puts"></a>puts
Display a comma-separated list of objects, followed by a line break.
The function takes zero or more arguments; when no arguments are passed,
it simply prints a new line.

Example:
```
puts(1, 1.1, "test")
```

### <a name="first"></a>first
Get the first element in an array. It accepts only an array as argument.
When the array is empty, it returns null.

Example:
```
first([1, 2, 3])
```

### <a name="last"></a>last
Get the last element in an array. It accepts only an array as argument.
When the array is empty, it returns null.

Example:
```
last([1, 2, 3])
```

### <a name="rest"></a>rest
Get all but the first element in an array. When the array is empty, it returns null.

Example:
```
rest([1, 2, 3])
```

### <a name="push"></a>push
Add an element to the end of an array.

Example:
```
let a = [1, 2, 3];
push(a, 4);
```

### <a name="pop"></a>push
Remove an element from the end of an array

Example:
```
let a = [1, 2, 3];
pop(a);
```

### <a name="get"></a>get
Get the element at a specific index in an array or a value in a map
indexed by a key. If there is no element at the index or no value for
the specified key, it returns null.

Examples:
```
get([1, 2, 3], 1);
get(map {"a": 1, "b": 2}, "a");
```

### <a name="contains"></a>contains
Check if a map contains a pairs indexed by the specified key.
It returns true if present otherwise, it returns false.

Example:
```
contains(map {"a": 1, "b": 2}, "a");
```

### <a name="insert"></a>insert
Insert a key-value pair into a map. If the key already exists,
the old value is returned, otherwise a null is returned.

### <a name="str"></a>str
Convert a value to a string.
It can be an Null, an integer, a floating-point number, a character, a byte,
a boolean value, an array, a map or a string itself.

Example:
```
str(123)
```

### <a name="int"></a>int
Convert a value to an integer.
It can be a string, a floating-point number, a character, a byte,
a boolean value, or an integer itself.

Example:
```
int("123")
```

### <a name="float"></a>float
Convert a value to a floating-point number.
It can be a string, an integer, a character, a byte, a boolean value,
or a floating-point number itself.

Example:
```
float("123.4")
```

### <a name="char"></a>char
Convert a value to a character.
It can be a string, an integer, a floating-point number, a byte,
a boolean value, or a character itself.

### <a name="byte"></a>byte
Convert a value to a byte.
It can be a string, an integer, a floating-point number, a character,
a boolean value, or a byte itself.

### <a name="time"></a>time
Get the current time.

### <a name="exit"></a>exit
Exit the program with an exit code passed in as the argument.

Example:
```
exit(0)
```

### <a name="flush"></a>flush
Flush stdout, stderr or a file handle.

Example:
```
flush(stdout)
```

### <a name="format"></a>format
Format a string with format specifiers.
```
format(<specifier>, <comma-separated-list-of-values>)
```
Refer the examples for more details.

### <a name="print"></a>print
Display a string with format specifiers to stdout.
Refer the examples for more details.

### <a name="println"></a>println
Display a string with format specifiers to stdout, followed by a line break.
Refer the examples for more details.

### <a name="eprint"></a>eprint
Display a string with format specifiers to stderr.
Refer the examples for more details.

### <a name="eprintln"></a>eprintln
Display a string with format specifiers to stderr, followed by a line break.
Refer the examples for more details.

### <a name="round"></a>round
Round a floating-point number.
It accepts two arguments - the number to round and the precision.

Example:
```
round(3.11111, 2)
```

### <a name="sleep"></a>sleep
Sleep for a specified duration in seconds

Example:
```
sleep(2)
```

### <a name="tolower"></a>tolower
Convert a character, a byte or a string to lowercase

Example:
```
tolower("A")
```

### <a name="toupper"></a>toupper
Convert a character, a byte or a string to uppercase

Example:
```
toupper("a")
```

### <a name="open"></a>open
Open a file for reading or writing.
It accepts a file path and an optional mode as a string and returns a file handle.

Example:
```
let f = open("/path/to/file", "r");
```

Modes supported
| mode | Description |
|------|-------------|
| r | Open file for reading. Return null if the file does not exist |
| w | Open or create file for writing. Truncate if exists |
| a | Open file for writing to the end of the file. Create it if it does not exist |
| x | Create a file and open it for writing. Return null if it exits |

### <a name="read"></a>read
Read from a file or stdin.
It accepts a file handle as first argument and an optional number of bytes
as the second argument. It returns an array of bytes encoded as UTF-8.

```
let f = open("test");
let bytes = read(f);
```

### <a name="write"></a>write
Write to a file, stdout or stderr.
It accepts a file handle as first argument and a string or an array of bytes
as the second argument. It returns the number of bytes written.

```
let f = open("test", "w");
write(f, "hello");
```

### <a name="read_to_string"></a>read_to_string
Read the contents of a file into a string
It accepts a file handle as first argument and returns the content of the
file as a string.

```
let f = open("test");
let s = read_to_string(f);
```

### <a name="decode_utf8"></a>decode_utf8
Decode a UTF-8 byte sequence to a string

Example:
```
let f = open("test");
let bytes = read(f);
let s = decode_utf8(bytes);
```

### <a name="encode_utf8"></a>encode_utf8
Encode a string to a UTF-8 byte sequence

Example:
```
let bytes = encode_utf8("hello");
let f = open("test", "w");
write(f, bytes);
```

### <a name="read_line"></a>read_line
Read a line from the standard input or a file handle into a string.

Example:
```
let f = open("test");
let line = read_line(f);
```

Note that the newline character is not removed from the string read.

### <a name="input"></a>input
Read a line from the standard input and return it as a string.
It accepts an optional prompt argument.

Example:
```
let line = input("-->> ");
```

### <a name="get_errno"></a>get_errno
Get the last os error number

Example:
```
get_errno()
```

### <a name="strerror"></a>strerror
Convert an os error number to a string

Example:
```
strerror(2)
```

### <a name="is_error"></a>is_error
Check if the object is an error object

Example:
```
is_error(2)
```

### <a name="sort"></a>sort
Sort an array object

Example:
```
sort([3, 2, 1])
```

### <a name="chars"></a>chars
Sort an array object

Example:
```
chars("Hello")
```

### <a name="join"></a>join
Join an array of characters into a string

Examples:
```
join(['h', 'e', 'l', 'l', 'o']);
join(['h', 'e', 'l', 'l', 'o'], ' ');
join(['h', 'e', 'l', 'l', 'o'], ", ");
```

## Builtin variables

The following table lists the builtin variables.

| Name | Description |
|------|-------------|
| [**argv**](#argv) | The command line arguments vector |

### <a name="argv"></a>argv
This is a variable available to the script by default and contains
an array of strings that represent the command line arguments
passed to the script at runtime.

In REPL, the argv is an empty array.