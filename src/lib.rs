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

/// A function that prevents the optimizer from eliminating the supplied value.
pub fn retain<T>(value: T) -> T {
    test::black_box(value)
}
