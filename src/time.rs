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

//! Time-related utilities.

use std::fmt;
use std::time::{Duration, Instant};

//================================================
// Structs
//================================================

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
