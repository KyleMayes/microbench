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

/// A number of nanoseconds.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Nanoseconds<T>(pub T);

impl fmt::Display for Nanoseconds<u64> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:.1}s", self.0 as f64 / 1_000_000_000.0)
    }
}

impl From<Duration> for Nanoseconds<u64> {
    fn from(duration: Duration) -> Nanoseconds<u64> {
        let nanos = u64::from(duration.subsec_nanos());
        Nanoseconds((duration.as_secs() * 1_000_000_000) + nanos)
    }
}

/// A high-precision stopwatch.
#[derive(Clone, Copy, Debug)]
pub struct Stopwatch(Instant);

impl Stopwatch {
    /// Returns the number of nanoseconds that have elapsed since this stopwatch
    /// was last constructed or reset.
    pub fn elapsed(self) -> Nanoseconds<u64> {
        (Instant::now() - self.0).into()
    }
}

impl Default for Stopwatch {
    fn default() -> Self {
        Stopwatch(Instant::now())
    }
}
