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

//! Miscellaneous utilities.

/// Generates unique values from a geometric sequence.
#[derive(Copy, Clone, Debug)]
pub struct GeometricSequence {
    current: f64,
    factor: f64,
}

impl GeometricSequence {
    /// Constructs a new `GeometricSequence`.
    pub fn new(start: u64, factor: f64) -> Self {
        GeometricSequence { current: start as f64, factor }
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

/// A function that prevents the optimizer from eliminating the supplied value.
#[cfg(feature="nightly")]
pub fn black_box<T>(dummy: T) -> T {
    use test;
    test::black_box(dummy)
}

/// A function that prevents the optimizer from eliminating the supplied value.
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

/// Returns the supplied floating-point number formatted with the supplied
/// precision and thousands separator.
pub fn format_number(number: f64, precision: usize, separator: char) -> String {
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

    let fractional = format!("{:.*}", precision, number.fract());
    format!("{}.{}", integral, &fractional[2..])
}
