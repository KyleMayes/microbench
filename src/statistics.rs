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

//! Statistics-related utilities.

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
// Functions
//================================================

/// Returns the alpha, beta, and goodness of fit for the supplied data using OLS linear regression.
pub fn regression(data: &[(f64, f64)]) -> (f64, f64, f64) {
    let xmean = data.iter().map(|d| d.0).mean();
    let ymean = data.iter().map(|d| d.1).mean();

    // Ordinary least squares linear regression.
    let numerator = data.iter().map(|m| (m.0 - xmean) * (m.1 - ymean)).kahan_sum();
    let denominator = data.iter().map(|m| (m.0 - xmean).powf(2.0)).kahan_sum();
    let beta = numerator / denominator;
    let alpha = ymean - (beta * xmean);
    let estimator = |x| (beta * x) + alpha;

    // Ordinary least squares goodness of fit.
    let numerator = data.iter().map(|m| (estimator(m.0) - ymean).powf(2.0)).kahan_sum();
    let denominator = data.iter().map(|m| (m.1 - ymean).powf(2.0)).kahan_sum();
    let r2 = numerator / denominator;

    (alpha, beta, r2)
}
