# microbench

[![crates.io](https://img.shields.io/crates/v/microbench.svg)](https://crates.io/crates/microbench)
[![Travis CI](https://travis-ci.org/KyleMayes/microbench.svg?branch=master)](https://travis-ci.org/KyleMayes/microbench)

[Documentation](https://kylemayes.github.io/microbench/microbench)

A micro-benchmarking library.

Inspired by [core_bench](https://github.com/janestreet/core_bench).

Released under the Apache License 2.0.

# Overview

`microbench` uses linear regression to estimate the execution time of code segments. For
example, the following table might represent data collected by `microbench` about a code
segment.

| Iterations | Time (ns) |
|------------|-----------|
| 1          | 19        |
| 2          | 25        |
| 3          | 37        |
| 4          | 47        |
| 5          | 56        |

`microbench` of course takes many more than 5 samples and the number of iterations grows
geometrically rather than linearly, but the concept remains the same. After collecting data like
this, `microbench` uses ordinary least squares (OLS) linear regression to estimate the actual
execution time of the code segment. Using OLS with the above data would yield an estimated
execution time of `9.6` nanoseconds with a goodness of fit (R²) of `0.992`.

# Example

```rust
use std::time::{Duration};
use microbench::{self, Options};

fn fibonacci_iterative(n: u64) -> u64 {
    let (mut x, mut y, mut z) = (0, 1, 1);
    for _ in 0..n { x = y; y = z; z = x + y; }
    x
}

fn fibonacci_recursive(n: u64) -> u64 {
    if n < 2 {
        n
    } else {
        fibonacci_recursive(n - 2) + fibonacci_recursive(n - 1)
    }
}

let options = Options::default().maximum(Duration::new(1, 0));
microbench::bench(&options, "iterative_16", || fibonacci_iterative(16));
microbench::bench(&options, "recursive_16", || fibonacci_recursive(16));
```

Example output:

```console
iterative_16 ... bench:                  273.757 ns/iter (0.999 R²)
recursive_16 ... bench:                9_218.530 ns/iter (0.999 R²)
```
