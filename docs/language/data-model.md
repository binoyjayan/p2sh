# Data types and constants

## builtin data types

| Name | Description |
|------|-------------|
| bool | A boolean type |
| integer | Represented as an 8 byte value |
| float | Represented as a double precision value |
| string | An immutable value in memory |
| char | A character represented using 4 bytes |
| byte | A single byte |
| array | A dynamic array |
| map | A hash-map data structure |

## Constants

| Name | Description |
|------|-------------|
| null | null value |
| boolean constants | boolean true or false |
| integer constants | |
| floating constants | |
| string constants | represented within double quotes |
| character constants | represented within single quotes e.g. 'c' |
| byte constants | represented within single quotes and byte prefix e.g. b'c' |


## Other literals

| Name | Description |
|------|-------------|
| [**array**](#array) | sequence of comma separated list of values enclosed within brackets |
| [**map**](#map) | sequence of comma separated list of key-value pairs enclosed within map {} |


### <a name="array"></a>array literals

An array literals is defined as a sequence of comma separated list of items,
that can each be evaluated as an expression and is enclosed with a pair
of square brackets.

```
[val1, val2, val3]
```


### <a name="map"></a>map literals

A hash literal is defined as a sequence of comma separated list of key-value,
pairs that can each be evaluated as an expression and is enclosed with a pair
of curly braces.

```
map { key1: value1, key2: value2 }
```

The values of the hash literal can be anything. However, they keys can only be
of a few types:

- integer
- floating-point number
- character
- byte
- string
- boolean
- builtin function





