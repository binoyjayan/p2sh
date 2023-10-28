let fibonacci = fn(x) {
    if x == 0 {
        return 0;
    } else {
        if x == 1 {
            return 1;
        } else {
            fibonacci(x - 1) + fibonacci(x - 2);
        }
    }
};

let n = if len(argv) > 1 {
  int(argv[1])
} else {
 30
};

let t1 = time();
let fib = fibonacci(n);
let t2 = time();
let secs = t2 - t1;

println("fib({}) = {} [took {} secs] ", n, fib, secs);


