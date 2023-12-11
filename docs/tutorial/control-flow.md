# Control Flow

## if expression

Save the following script in a file. e.g. 'if.p2'.
On posix systems, you can use 'cat' with 'EOF' to create this file
on your terminal. Alternatively, an editor may also be used.

```

cat > if.p2 <<EOF

if len(argv) < 2 {
  exit(1);
}

let n = int(argv[1]);

let s = if n < 0 {
  "Negative"
} else if n == 0 {
  "Zero"
} else {
  "+ve"
};

puts(s);

EOF
```

Note that `if` is not a statement but an expression that produces a value.

Execute the script as follows:

```
p2sh if.p2 0
Zero

p2sh if.p2 1
+ve

p2sh if.p2 -1
-ve
```

## match expressions

The match expressions work similar to the one found in `Rust`. It matches a
condition expression with multiple match arms using patterns. If a match is
successful, the body of the match arm is executed. Otherwise, it continues until
a match is found. Once a match is found, the control flow breaks out of the check.
Note that there is no fallthrough mechanism similar to a switch statment in c.

```

cat > match.p2 <<EOF

let s = input("-->> ");
let r = match tolower(s) {
  "a" | "e" | "i" | "o" | "u" => {  "vowel" }
  "b"..="z" => "consonant",
  _ => {  "others" }
}
puts(r);


EOF
```


Execute the program as follows:

```
p2sh  match.p2
-->> a
vowel

p2sh  match.p2
-->> b
consonant

p2sh  match.p2
-->> ?
others

```

## Loop statements

The loop statement executes the loop body infinitely. To break out of the loop,
use the break statement.

```

cat > loop.p2 <<EOF

let n = 1;
loop {
  if n > 5 {
    break;
  }
  puts(n);
  n = n + 1;
}

EOF
```

Execution

```
p2sh loop.p2
1
2
3
4
5
```

## While

The same code can be written using while

```
>> let n = 1;
1

>> while n <= 5 { puts(n); n = n + 1; }
1
2
3
4
5
false
```

Note that the `false` at the end is the value of the last expression
evaluated and can be ignored. In this case it is the result of
the condition in the while statement.


### loop labels

break and continue statements can also have labels in them. This is helpful
when loops are nested.

```
cat >> while.p2 <<EOF

let i = 1;
outer: while i <= 3 {
  let j = 1;
  while j <= 3 {
    if j == 2 {
      j = j + 1;
      i = i + 1;
      continue outer;
    }
    println("{},{}", i, j);
    j = j + 1;
 }
 i = i + 1;
}

EOF
```

Execution

```
p2sh while.p2
1,1
2,1
3,1
```

### continue statements

The continue statement can be used to continue to the execution of the
next iteration of the loop without executing the rest of the loop body.
Save the following file in `loop.p2`

```
cat > loop.p2 <<EOF

let n = 1;
while n <= 5 {
  if n == 3 {
    n = n + 1;
    continue;
  }
  puts(n);
  n = n + 1;
}

EOF
```

Execution

```
p2sh loop.p2
1
2
4
5
```

## Functions

### Using the let keyword

```
>> let add = fn(a, b) { a + b };

>> add(1,2)
3
```

### Function statements

```
>> fn f1() {  1; }
>> fn f2() { return 2; }

>> f1();
1

>> f2();
2
```

### Closures

Closures are functions that can capture the surrounding variables at the time
of its definitions. In the following example, the inner anonymous function is
a closure that closes over the variable 'value' that is passed as an argument.
In this case, the values of that variable are `doughnut` and `bagel`.
Note that the inner anonymous function uses the surrounding variable `value`.

```

>> fn make_closure(value) {  return fn() { puts(value);  } }
<closure>

>> let doughnut = make_closure("doughnut");
<closure>

>> let bagel = make_closure("bagel");
<closure>

>> doughnut();
doughnut

>> bagel();
bagel
```

A more useful example

```
cat >> filter.p2 <<EOF

fn filter(a) {
  let i = 0;
  let t = [];
  while i < len(a) {
    let is_even = fn() { a[i] % 2 == 0 };
    if is_even() {
      push(t, a[i]);
    }
    i = i + 1;
  }
  t
}

let numbers = filter([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
puts(numbers);

EOF

```

Execution

```
p2sh filter.p2
[2, 4, 6, 8, 10]
```