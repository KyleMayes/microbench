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
//! # Example
//!
//! ```
//! use std::time::{Duration};
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
//! let options = Options::default().maximum(Duration::new(1, 0));
//! microbench::bench(&options, "iterative_16", || fibonacci_iterative(16));
//! microbench::bench(&options, "recursive_16", || fibonacci_recursive(16));
//! ```
//!
//! Example output:
//!
//! ```console
//! iterative_16 ... bench:          284.161 ns/iter (1.000 R²)
//! recursive_16 ... bench:          9222.037 ns/iter (1.000 R²)
//! ```

#![feature(test)]

#![warn(missing_copy_implementations, missing_debug_implementations, missing_docs)]

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![cfg_attr(feature="clippy", warn(clippy))]
#![cfg_attr(feature="clippy", allow(new_without_default_derive))]

extern crate test;
extern crate time;

use std::time::{Duration};

//================================================
// Structs
//================================================

// Analysis ______________________________________

/// An analysis of a set of timing data.
#[derive(Copy, Clone, Debug)]
pub struct Analysis {
    /// The slope of the linear regression estimator.
    pub beta: f64,
    /// The goodness of fit of the linear regression estimator.
    pub r2: f64,
}

// GeometricSequence _____________________________

/// Generates unique values from a geometric sequence.
#[derive(Copy, Clone, Debug)]
pub struct GeometricSequence {
    current: f64,
    factor: f64,
}

impl GeometricSequence {
    //- Constructors -----------------------------

    /// Constructs a new `GeometricSequence`.
    pub fn new(start: u64, factor: f64) -> Self {
        GeometricSequence { current: start as f64, factor: factor }
    }
}

impl Iterator for GeometricSequence {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.current as u64;
        while self.current as u64 == start { self.current *= self.factor; }
        Some(start)
    }
}

// Measurement ___________________________________

/// A measurement of the execution time of a function.
#[derive(Copy, Clone, Debug)]
pub struct Measurement {
    /// The number of times the function was called.
    pub iterations: u64,
    /// The number of nanoseconds that elapsed while calling the function.
    pub nanoseconds: u64,
}

impl Measurement {
    //- Constructors -----------------------------

    /// Constructs a new `Measurement`.
    pub fn new(iterations: u64, nanoseconds: u64) -> Self {
        Measurement { iterations: iterations, nanoseconds: nanoseconds }
    }
}

// Options _______________________________________

/// Micro-benchmarking options.
#[derive(Copy, Clone, Debug)]
pub struct Options {
    factor: f64,
    maximum: u64,
}

impl Default for Options {
    fn default() -> Options {
        Options { factor: 1.01, maximum: 5_000_000_000 }
    }
}

impl Options {
    //- Consumers --------------------------------

    /// Sets the geometric growth factor.
    ///
    /// **Default:** `1.01`
    pub fn factor(mut self, factor: f64) -> Self {
        self.factor = factor;
        self
    }

    /// Sets the maximum amount of time a micro-benchmark will run for.
    ///
    /// **Default:** `Duration::new(5, 0)`
    pub fn maximum(mut self, maximum: Duration) -> Self {
        self.maximum = (maximum.as_secs() * 1_000_000_000) + maximum.subsec_nanos() as u64;
        self
    }
}

// Stopwatch _____________________________________

/// A high-precision stopwatch.
#[derive(Clone, Copy, Debug)]
pub struct Stopwatch {
    start: u64,
}

impl Stopwatch {
    //- Constructors -----------------------------

    /// Constructs a new `Stopwatch`.
    pub fn new() -> Self {
        Stopwatch { start: time::precise_time_ns() }
    }

    //- Accessors --------------------------------

    /// Returns the elapsed nanoseconds since this stopwatch was constructed or reset.
    pub fn elapsed(&self) -> u64 {
        time::precise_time_ns() - self.start
    }

    //- Mutators ---------------------------------

    /// Resets this stopwatch.
    pub fn reset(&mut self) {
        self.start = time::precise_time_ns();
    }
}

//================================================
// Functions
//================================================

fn mean<I>(iterator: I) -> f64 where I: Iterator<Item=f64> {
    let (sum, len) = iterator.fold((0.0, 0), |a, f| (a.0 + f, a.1 + 1));
    sum / len as f64
}

/// Analyzes the supplied timing data and returns the resulting analysis.
pub fn analyze(measurements: &[Measurement]) -> Analysis {
    let xmean = mean(measurements.iter().map(|m| m.iterations as f64));
    let ymean = mean(measurements.iter().map(|m| m.nanoseconds as f64));

    // Ordinary least squares estimator.
    let numerator = measurements.iter().map(|m| {
        (m.iterations as f64 - xmean) * (m.nanoseconds as f64 - ymean)
    }).sum::<f64>();
    let denominator = measurements.iter().map(|m| {
        (m.iterations as f64 - xmean).powf(2.0)
    }).sum::<f64>();
    let beta = numerator / denominator;
    let alpha = ymean - (beta * xmean);
    let estimator = |x: u64| (beta * x as f64) + alpha;

    // Ordinary least squares goodness of fit.
    let numerator = measurements.iter().map(|m| {
        (estimator(m.iterations) - ymean).powf(2.0)
    }).sum::<f64>();
    let denominator = measurements.iter().map(|m| {
        (m.nanoseconds as f64 - ymean).powf(2.0)
    }).sum::<f64>();
    let r2 = numerator / denominator;

    Analysis { beta: beta, r2: r2 }
}

/// Micro-benchmarks the supplied function.
pub fn bench<T, F>(options: &Options, name: &str, f: F) where F: FnMut() -> T {
    let analysis = analyze(&measure(options, f));
    let prefix = format!("{} ... bench:", name);
    println!("{:<32} {:.3} ns/iter ({:.3} R²)", prefix, analysis.beta, analysis.r2);
}

/// Measures the execution time of the supplied function and returns the resulting samples.
pub fn measure<T, F>(options: &Options, mut f: F) -> Vec<Measurement> where F: FnMut() -> T {
    let mut measurements = vec![];
    let mut sequence = GeometricSequence::new(1, options.factor);
    let total = Stopwatch::new();
    while total.elapsed() < options.maximum {
        let iterations = sequence.next().unwrap();
        let sample = Stopwatch::new();
        for _ in 0..iterations { retain(f()); }
        measurements.push(Measurement::new(iterations, sample.elapsed()));
    }
    measurements
}

/// A function that prevents the optimizer from eliminating the supplied value.
pub fn retain<T>(value: T) -> T {
    test::black_box(value)
}
