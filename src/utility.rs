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

use std::time::{Instant};

use super::{Nanoseconds};

//================================================
// Traits
//================================================

// Statistics ____________________________________

/// A type that supports various statistical operations.
pub trait Statistics {
    /// Returns the sum of the items in this value using the Kahan summation algorithm.
    fn kahan_sum(self) -> f64;

    /// Returns the mean of the items in this value.
    fn mean(self) -> f64;
}

impl<I> Statistics for I where I: ExactSizeIterator<Item=f64> {
    fn kahan_sum(self) -> f64 {
        self.fold((0.0, 0.0), |(sum, correction), f| {
            let y = f - correction;
            let t = sum + y;
            (t, (t - sum) - y)
        }).0
    }

    fn mean(self) -> f64 {
        let len = self.len() as f64;
        self.kahan_sum() / len
    }
}

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

// Stopwatch _____________________________________

/// A high-precision stopwatch.
#[derive(Clone, Copy, Debug)]
pub struct Stopwatch {
    start: Instant,
}

impl Stopwatch {
    //- Constructors -----------------------------

    /// Constructs a new `Stopwatch`.
    pub fn new() -> Self {
        Stopwatch { start: Instant::now() }
    }

    //- Accessors --------------------------------

    /// Returns the elapsed nanoseconds since this stopwatch was constructed or reset.
    pub fn elapsed(&self) -> Nanoseconds<u64> {
        (Instant::now() - self.start).into()
    }
}

//================================================
// Functions
//================================================

#[cfg(feature="nightly")]
pub fn black_box<T>(dummy: T) -> T {
    use test;
    test::black_box(dummy)
}
#[cfg(not(feature="nightly"))]
pub fn black_box<T>(dummy: T) -> T {
    use std::mem;
    use std::ptr;
    unsafe {
        let value = ptr::read_volatile(&dummy);
        mem::forget(dummy);
        value
    }
}

/// Formats the supplied floating-point number with the supplied thousands separator.
pub fn thousands(number: f64, separator: char) -> String {
    let mut integral = String::new();
    let mut counter = 0;
    for digit in (number as u64).to_string().chars().rev() {
        if counter == 3 {
            integral.insert(0, separator);
            counter = 0;
        }
        counter += 1;
        integral.insert(0, digit);
    }
    let fractional = format!("{:.3}", number.fract());
    format!("{}.{}", integral, &fractional[2..])
}
