// Copyright 2016 Kyle Mayes
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! A micro-benchmarking library.
//!
//! # Overview
//!
//! `microbench` uses linear regression to estimate the execution time of code
//! segments. For example, the following table might represent data collected by
//! `microbench` about a code segment.
//!
//! | Iterations | Time (ns) |
//! |------------|-----------|
//! | 1          | 19        |
//! | 2          | 25        |
//! | 3          | 37        |
//! | 4          | 47        |
//! | 5          | 56        |
//!
//! `microbench` of course takes many more than 5 samples and the number of
//! iterations grows geometrically rather than linearly, but the idea remains
//! the same. After collecting data like this, `microbench` uses ordinary least
//! squares (OLS) linear regression to estimate the actual execution time of the
//! code segment. Using OLS with the above data would yield an estimated
//! execution time of `9.6` nanoseconds with a goodness of fit (R²) of `0.992`.
//!
//! # Example
//!
//! ```
//! use microbench::{self, Options};
//!
//! fn fibonacci_iterative(n: u64) -> u64 {
//!     let (mut x, mut y, mut z) = (0, 1, 1);
//!     for _ in 0..n { x = y; y = z; z = x + y; }
//!     x
//! }
//!
//! fn fibonacci_recursive(n: u64) -> u64 {
//!     if n < 2 {
//!         n
//!     } else {
//!         fibonacci_recursive(n - 2) + fibonacci_recursive(n - 1)
//!     }
//! }
//!
//! let options = Options::default();
//! microbench::bench(&options, "iterative_16", || fibonacci_iterative(16));
//! microbench::bench(&options, "recursive_16", || fibonacci_recursive(16));
//! ```
//!
//! Example output:
//!
//! ```console
//! iterative_16 (5.0s) ...                  281.733 ns/iter (0.998 R²)
//! recursive_16 (5.0s) ...                9_407.020 ns/iter (0.997 R²)
//! ```

#![cfg_attr(feature="nightly", feature(test))]

#![warn(missing_copy_implementations, missing_debug_implementations, missing_docs)]

mod utility;
pub mod statistics;
pub mod time;

use std::cmp;
use std::mem;
use std::time::{Duration};

use crate::statistics::{Model};
use crate::time::{Nanoseconds, Stopwatch};
use crate::utility::{GeometricSequence, black_box, format_number};

/// The maximum number of benchmark sample iterations.
const ITERATIONS: u64 = 1_000_000_000_000_000;

/// A number of bytes.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bytes(pub u64);

impl Bytes {
    /// Returns the number of bytes in the supplied number of kibibytes (2¹⁰ bytes).
    pub fn kibibytes(kibibytes: u64) -> Self {
        Bytes(kibibytes * 1024)
    }

    /// Returns the number of bytes in the supplied number of mebibytes (2²⁰ bytes).
    pub fn mebibytes(mebibytes: u64) -> Self {
        Bytes(mebibytes * 1024 * 1024)
    }

    /// Returns the number of bytes in the supplied number of gibibytes (2³⁰ bytes).
    pub fn gibibytes(gibibytes: u64) -> Self {
        Bytes(gibibytes * 1024 * 1024 * 1024)
    }
}

/// A set of benchmarking options.
#[derive(Copy, Clone, Debug)]
pub struct Options {
    factor: f64,
    memory: Bytes,
    time: Nanoseconds<u64>,
}

impl Options {
    /// Sets the geometric growth factor for benchmark sample iterations.
    ///
    /// **Default:** `1.01`
    pub fn factor(mut self, factor: f64) -> Self {
        self.factor = factor;
        self
    }

    /// Sets the maximum amount of memory benchmarks will allocate.
    ///
    /// **Default:** `Bytes::mebibytes(512)`
    pub fn memory(mut self, memory: Bytes) -> Self {
        self.memory = memory;
        self
    }

    /// Sets the maximum amount of time benchmarks will run for.
    ///
    /// **Default:** `Duration::new(5, 0)`
    pub fn time(mut self, time: Duration) -> Self {
        self.time = time.into();
        self
    }
}

impl Default for Options {
    fn default() -> Self {
        let factor = 1.01;
        let memory = Bytes::mebibytes(512);
        let time = Duration::new(5, 0).into();
        Options { factor, memory, time }
    }
}

/// A sample of the execution time of a function.
#[derive(Copy, Clone, Debug)]
pub struct Sample {
    /// The number of times the function was executed.
    pub iterations: u64,
    /// The number of nanoseconds that elapsed while executing the function.
    pub elapsed: Nanoseconds<u64>,
}

/// A statistical analysis of a set of execution time samples.
#[derive(Copy, Clone, Debug)]
pub struct Analysis {
    /// The y-intercept of the simple linear regression model function.
    pub alpha: Nanoseconds<f64>,
    /// The slope of the simple linear regression model function.
    pub beta: Nanoseconds<f64>,
    /// The goodness of fit of the simple linear regression model function.
    pub r2: f64,
}

impl Analysis {
    /// Returns a new analysis for the supplied samples.
    fn new(samples: &[Sample]) -> Self {
        let Model { alpha, beta, r2 } = samples.iter()
            .map(|m| (m.iterations as f64, m.elapsed.0 as f64))
            .collect::<Model>();
        Self { alpha: Nanoseconds(alpha), beta: Nanoseconds(beta), r2 }
    }
}

/// Benchmarks the supplied function and prints the results.
pub fn bench<T>(options: &Options, name: &str, f: impl FnMut() -> T) {
    bench_impl(name, move || measure(options, f));
}

/// Benchmarks the supplied function ignoring drop time and prints the results.
///
/// See [`measure_drop`](fn.measure_drop.html) for more information.
pub fn bench_drop<T>(options: &Options, name: &str, f: impl FnMut() -> T) {
    bench_impl(name, move || measure_drop(options, f));
}

/// Benchmarks the supplied function ignoring setup time and prints the results.
///
/// See [`measure_setup`](fn.measure_setup.html) for more information.
pub fn bench_setup<I, T>(
    options: &Options,
    name: &str,
    setup: impl FnMut() -> I,
    f: impl FnMut(I) -> T,
) {
    bench_impl(name, move || measure_setup(options, setup, f));
}

/// Measures the execution time of the supplied function.
pub fn measure<T>(
    options: &Options, mut f: impl FnMut() -> T
) -> Vec<Sample> {
    measure_impl(options, |iterations| {
        let stopwatch = Stopwatch::default();
        for _ in 0..iterations { retain(f()); }
        Some(stopwatch.elapsed())
    })
}

/// Measures the execution time of the supplied function ignoring drop time.
///
/// This function does not include the time it takes to drop the values returned
/// by the supplied function in the measurements. This can be useful when you
/// want to exclude the running time of a slow implementation of `Drop` from
/// your benchmark. However, it should be noted that this function introduces a
/// very small amount of overhead which will be reflected in the measurements
/// (typically of the order of a few nanoseconds).
///
/// **Warning:** This function can potentially allocate very large amounts of
/// memory. The `memory` option controls the maximum amount of memory this
/// function is allowed to allocate.
pub fn measure_drop<T>(
    options: &Options, mut f: impl FnMut() -> T
) -> Vec<Sample> {
    measure_impl(options, |iterations| {
        let size = cmp::max(1, mem::size_of::<T>() as u64);
        if options.memory < Bytes(iterations * size) {
            return None;
        }

        let mut outputs = Vec::with_capacity(iterations as usize);
        let stopwatch = Stopwatch::default();
        for _ in 0..iterations { outputs.push(f()); }
        let elapsed = stopwatch.elapsed();
        mem::drop(outputs);
        Some(elapsed)
    })
}

/// Measures the execution time of the supplied function ignoring setup time.
///
/// This function does not include the time it takes to execute the setup
/// function in the measurements. This can be useful when you want to exclude
/// the running time of some non-trivial setup which is needed for every
/// execution of the supplied function. However, it should be noted that this
/// function introduces a very small amount of overhead which will be reflected
/// in the measurements (typically of the order of a few nanoseconds).
///
/// **Warning:** This function can potentially allocate very large amounts of
/// memory. The `memory` option controls the maximum amount of memory this
/// function is allowed to allocate.
pub fn measure_setup<I, T>(
    options: &Options,
    mut setup: impl FnMut() -> I,
    mut f: impl FnMut(I) -> T,
) -> Vec<Sample> {
    measure_impl(options, |iterations| {
        let size = cmp::max(1, mem::size_of::<I>() as u64);
        if options.memory < Bytes(iterations * size) {
            return None;
        }

        let inputs = retain((0..iterations).map(|_| setup()).collect::<Vec<_>>());
        let stopwatch = Stopwatch::default();
        for input in inputs { retain(f(input)); }
        Some(stopwatch.elapsed())
    })
}

/// A function that prevents the optimizer from eliminating the supplied value.
///
/// This function may not operate correctly or may have poor performance on the
/// stable and beta channels of Rust. If you are using a nightly release of
/// Rust, enable the `nightly` crate feature to enable a superior implementation
/// of this function.
pub fn retain<T>(value: T) -> T {
    black_box(value)
}

/// Prints an analysis of the samples produced by the supplied function.
fn bench_impl(name: &str, f: impl FnOnce() -> Vec<Sample>) {
    let stopwatch = Stopwatch::default();
    let samples = f();
    let elapsed = stopwatch.elapsed();
    let analysis = Analysis::new(&samples);

    let prefix = format!("{} ({}) ...", name, elapsed);
    if samples.len() < 2 || analysis.beta.0 < 0.0 {
        println!("{:<32} {:>15}", prefix, "           not enough samples");
    } else {
        let beta = format_number(analysis.beta.0, 3, '_');
        println!("{:<32} {:>15} ns/iter ({:.3} R²)", prefix, beta, analysis.r2);
    }
}

/// Collects samples produced by the supplied function.
fn measure_impl(
    options: &Options, mut f: impl FnMut(u64) -> Option<Nanoseconds<u64>>
) -> Vec<Sample> {
    let stopwatch = Stopwatch::default();
    GeometricSequence::new(1, options.factor)
        .take_while(|i| *i <= ITERATIONS && stopwatch.elapsed() < options.time)
        .filter_map(|i| Some(Sample { iterations: i, elapsed: f(i)? }))
        .collect()
}
