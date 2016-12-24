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

//================================================
// Structs
//================================================

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
