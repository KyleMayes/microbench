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
//! `microbench` uses linear regression to estimate the execution time of code segments. For
//! example, the following table might represent data collected by `microbench` about a code
//! segment.
//!
//! | Iterations | Time (ns) |
//! |------------|-----------|
//! | 1          | 19        |
//! | 2          | 25        |
//! | 3          | 37        |
//! | 4          | 47        |
//! | 5          | 56        |
//!
//! `microbench` of course takes many more than 5 samples and the number of iterations grows
//! geometrically rather than linearly, but the idea remains the same. After collecting data like
//! this, `microbench` uses ordinary least squares (OLS) linear regression to estimate the actual
//! execution time of the code segment. Using OLS with the above data would yield an estimated
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

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![cfg_attr(feature="clippy", warn(clippy))]

#[cfg(feature="nightly")]
extern crate test;

mod utility;

use std::cmp;
use std::fmt;
use std::mem;
use std::time::{Duration};

use utility::{GeometricSequence, Statistics, Stopwatch};

//================================================
// Structs
//================================================

// Analysis ______________________________________

/// A statistical analysis of a set of timing data.
#[derive(Copy, Clone, Debug)]
pub struct Analysis {
    /// The y-intercept of the linear regression estimator.
    pub alpha: Nanoseconds<f64>,
    /// The slope of the linear regression estimator.
    pub beta: Nanoseconds<f64>,
    /// The goodness of fit of the linear regression estimator.
    pub r2: f64,
}

// Bytes _________________________________________

/// A number of bytes.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bytes(pub u64);

impl Bytes {
    //- Constructors -----------------------------

    /// Constructs a new `Bytes` with the supplied number of kibibytes (2¹⁰ bytes).
    pub fn kibibytes(kibibytes: u64) -> Self {
        Bytes(kibibytes * 1024)
    }

    /// Constructs a new `Bytes` with the supplied number of mebibytes (2²⁰ bytes).
    pub fn mebibytes(mebibytes: u64) -> Self {
        Bytes(mebibytes * 1024 * 1024)
    }

    /// Constructs a new `Bytes` with the supplied number of gibibytes (2³⁰ bytes).
    pub fn gibibytes(gibibytes: u64) -> Self {
        Bytes(gibibytes * 1024 * 1024 * 1024)
    }
}

// Measurement ___________________________________

/// A measurement of the execution time of a function.
#[derive(Copy, Clone, Debug)]
pub struct Measurement {
    /// The number of times the function was executed.
    pub iterations: u64,
    /// The amount of time that elapsed while executing the function.
    pub elapsed: Nanoseconds<u64>,
}

// Nanoseconds ___________________________________

/// A number of nanoseconds.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Nanoseconds<T>(pub T);

impl fmt::Display for Nanoseconds<u64> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:.1}s", self.0 as f64 / 1_000_000_000.0)
    }
}

impl From<Duration> for Nanoseconds<u64> {
    fn from(duration: Duration) -> Nanoseconds<u64> {
        Nanoseconds((duration.as_secs() * 1_000_000_000) + duration.subsec_nanos() as u64)
    }
}

// Options _______________________________________

/// Benchmarking options.
#[derive(Copy, Clone, Debug)]
pub struct Options {
    factor: f64,
    memory: Bytes,
    time: Nanoseconds<u64>,
}

impl Options {
    //- Consumers --------------------------------

    /// Sets the geometric growth factor for benchmark sample iterations.
    ///
    /// **Default:** `1.01`
    pub fn factor(mut self, factor: f64) -> Self {
        self.factor = factor;
        self
    }

    /// Sets the maximum amount of memory a benchmark will allocate.
    ///
    /// **Default:** `Bytes::mebibytes(512)`
    pub fn memory(mut self, memory: Bytes) -> Self {
        self.memory = memory;
        self
    }

    /// Sets the maximum amount of time a benchmark will run for.
    ///
    /// **Default:** `Duration::new(5, 0)`
    pub fn time(mut self, time: Duration) -> Self {
        self.time = time.into();
        self
    }
}

impl Default for Options {
    fn default() -> Options {
        Options { factor: 1.01, memory: Bytes::mebibytes(512), time: Duration::new(5, 0).into() }
    }
}

//================================================
// Functions
//================================================

fn bench_impl<F>(name: &str, f: F) where F: FnOnce() -> Vec<Measurement> {
    let stopwatch = Stopwatch::new();
    let measurements = f();
    let analysis = analyze(&measurements);
    let prefix = format!("{} ({}) ...", name, stopwatch.elapsed());
    if measurements.len() < 2 || analysis.beta.0 < 0.0 {
        println!("{:<32} {:>15}", prefix, "           not enough measurements");
    } else {
        let beta = utility::thousands(analysis.beta.0, '_');
        println!("{:<32} {:>15} ns/iter ({:.3} R²)", prefix, beta, analysis.r2);
    }
}

fn measure_impl<F>(
    options: &Options, mut f: F
) -> Vec<Measurement> where F: FnMut(u64) -> Option<Nanoseconds<u64>> {
    let mut measurements = vec![];
    let mut sequence = GeometricSequence::new(1, options.factor);
    let stopwatch = Stopwatch::new();
    while stopwatch.elapsed() < options.time {
        let iterations = sequence.next().unwrap();
        if let Some(elapsed) = f(iterations) {
            measurements.push(Measurement { iterations: iterations, elapsed: elapsed });
        } else {
            break;
        }
    }
    measurements
}

/// Analyzes the supplied timing data and returns the resulting analysis.
pub fn analyze(measurements: &[Measurement]) -> Analysis {
    let xmean = measurements.iter().map(|m| m.iterations as f64).mean();
    let ymean = measurements.iter().map(|m| m.elapsed.0 as f64).mean();

    // Ordinary least squares linear regression.
    let numerator = measurements.iter().map(|m| {
        (m.iterations as f64 - xmean) * (m.elapsed.0 as f64 - ymean)
    }).kahan_sum();
    let denominator = measurements.iter().map(|m| {
        (m.iterations as f64 - xmean).powf(2.0)
    }).kahan_sum();
    let beta = numerator / denominator;
    let alpha = ymean - (beta * xmean);
    let estimator = |x: u64| (beta * x as f64) + alpha;

    // Ordinary least squares goodness of fit.
    let numerator = measurements.iter().map(|m| {
        (estimator(m.iterations) - ymean).powf(2.0)
    }).kahan_sum();
    let denominator = measurements.iter().map(|m| {
        (m.elapsed.0 as f64 - ymean).powf(2.0)
    }).kahan_sum();
    let r2 = numerator / denominator;

    Analysis { alpha: Nanoseconds(alpha), beta: Nanoseconds(beta), r2: r2 }
}

/// Benchmarks the supplied function and prints the results.
pub fn bench<T, F>(options: &Options, name: &str, f: F) where F: FnMut() -> T {
    bench_impl(name, move || measure(options, f));
}

/// Benchmarks the supplied function ignoring drop time and prints the results.
///
/// See [`measure_drop`](fn.measure_drop.html) for more information.
pub fn bench_drop<T, F>(options: &Options, name: &str, f: F) where F: FnMut() -> T {
    bench_impl(name, move || measure_drop(options, f));
}

/// Benchmarks the supplied function ignoring setup time and prints the results.
///
/// See [`measure_setup`](fn.measure_setup.html) for more information.
pub fn bench_setup<I, S, T, F>(
    options: &Options, name: &str, setup: S, f: F
) where S: FnMut() -> I, F: FnMut(I) -> T {
    bench_impl(name, move || measure_setup(options, setup, f));
}

/// Measures the execution time of the supplied function.
pub fn measure<T, F>(options: &Options, mut f: F) -> Vec<Measurement> where F: FnMut() -> T {
    measure_impl(options, |iterations| {
        let stopwatch = Stopwatch::new();
        for _ in 0..iterations { retain(f()); }
        Some(stopwatch.elapsed())
    })
}

/// Measures the execution time of the supplied function ignoring drop time.
///
/// This function does not include the time it takes to drop the values returned by the supplied
/// function in the measurements. This can be useful when you want to exclude the running time of a
/// slow implementation of `Drop` from your benchmark. However, it should be noted that this
/// function introduces a very small amount of overhead which will be reflected in the measurements
/// (typically of the order of a few nanoseconds).
///
/// **Warning:** This function can potentially allocate very large amounts of memory. The `memory`
/// option controls the maximum this function is allowed to allocate.
pub fn measure_drop<T, F>(options: &Options, mut f: F) -> Vec<Measurement> where F: FnMut() -> T {
    measure_impl(options, |iterations| {
        if options.memory < Bytes(iterations * cmp::max(1, mem::size_of::<T>() as u64)) {
            return None;
        }
        let mut outputs = Vec::with_capacity(iterations as usize);
        let stopwatch = Stopwatch::new();
        for _ in 0..iterations { outputs.push(f()); }
        let elapsed = stopwatch.elapsed();
        mem::drop(outputs);
        Some(elapsed)
    })
}

/// Measures the execution time of the supplied function ignoring setup time.
///
/// This function does not include the time it takes to execute the setup function in the
/// measurements. This can be useful when you want to exclude the running time of some non-trivial
/// setup which is needed for every execution of the supplied function. However, it should be noted
/// that this function introduces a very small amount of overhead which will be reflected in the
/// measurements (typically of the order of a few nanoseconds).
///
/// **Warning:** This function can potentially allocate very large amounts of memory. The `memory`
/// option controls the maximum this function is allowed to allocate.
pub fn measure_setup<I, S, T, F>(
    options: &Options, mut setup: S, mut f: F
) -> Vec<Measurement> where S: FnMut() -> I, F: FnMut(I) -> T {
    measure_impl(options, |iterations| {
        if options.memory < Bytes(iterations * cmp::max(1, mem::size_of::<I>() as u64)) {
            return None;
        }
        let inputs = retain((0..iterations).map(|_| setup()).collect::<Vec<_>>());
        let stopwatch = Stopwatch::new();
        for input in inputs { retain(f(input)); }
        Some(stopwatch.elapsed())
    })
}

/// A function that prevents the optimizer from eliminating the supplied value.
///
/// This function may not operate correctly or may have poor performance on the stable and beta
/// channels of Rust. If you are using a nightly release of Rust, enable the `nightly` crate feature
/// to enable a better implementation of this function.
pub fn retain<T>(value: T) -> T {
    utility::black_box(value)
}
