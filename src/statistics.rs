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

use std::iter::{FromIterator};

/// A collection of floating-point numbers that supports Kahan summation.
pub trait Kahan {
    /// Returns the mean of the numbers in this collection using the Kahan
    /// summation algorithm.
    fn kahan_mean(self) -> f64;
    /// Returns the sum of the numbers in this collection using the Kahan
    /// summation algorithm.
    fn kahan_sum(self) -> f64;
}

impl<I> Kahan for I where I: ExactSizeIterator<Item=f64> {
    fn kahan_mean(self) -> f64 {
        let size = self.len() as f64;
        self.kahan_sum() / size
    }

    fn kahan_sum(self) -> f64 {
        self.fold((0.0, 0.0), |(sum, correction), f| {
            let y = f - correction;
            let t = sum + y;
            (t, (t - sum) - y)
        }).0
    }
}

/// A simple linear regression model.
#[derive(Copy, Clone, Debug)]
pub struct Model {
    /// The y-intercept of the model function.
    pub alpha: f64,
    /// The slope of the model function.
    pub beta: f64,
    /// The goodness of fit of the model function.
    pub r2: f64,
}

impl Model {
    /// Returns a new model for the supplied data using OLS linear regression.
    fn new(data: &[(f64, f64)]) -> Self {
        let xmean = data.iter().map(|d| d.0).kahan_mean();
        let ymean = data.iter().map(|d| d.1).kahan_mean();

        // OLS linear regression.
        let numerator = data.iter().map(|m| (m.0 - xmean) * (m.1 - ymean)).kahan_sum();
        let denominator = data.iter().map(|m| (m.0 - xmean).powf(2.0)).kahan_sum();
        let beta = numerator / denominator;
        let alpha = ymean - (beta * xmean);
        let estimator = |x| (beta * x) + alpha;

        // OLS goodness of fit.
        let numerator = data.iter().map(|m| (estimator(m.0) - ymean).powf(2.0)).kahan_sum();
        let denominator = data.iter().map(|m| (m.1 - ymean).powf(2.0)).kahan_sum();
        let r2 = numerator / denominator;

        Self { alpha, beta, r2 }
    }
}

impl FromIterator<(f64, f64)> for Model {
    fn from_iter<I>(iter: I) -> Self where I: IntoIterator<Item=(f64, f64)> {
        Model::new(&iter.into_iter().collect::<Vec<_>>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kahan() {
        let numbers: &[f64] = &[10000.0, 3.14159, 2.71828];
        assert_eq!(numbers.iter().cloned().kahan_sum(), 10005.85987);
    }

    #[test]
    fn test_model() {
        let data: &[(f64, f64)] = &[
            (1.47, 52.21), (1.50, 53.12), (1.52, 54.48),
            (1.55, 55.84), (1.57, 57.20), (1.60, 58.57),
            (1.63, 59.93), (1.65, 61.29), (1.68, 63.11),
            (1.70, 64.47), (1.73, 66.28), (1.75, 68.10),
            (1.78, 69.92), (1.80, 72.19), (1.83, 74.46),
        ];

        let model = data.iter().cloned().collect::<Model>();
        assert_eq!(model.alpha, -39.06195591884393);
        assert_eq!(model.beta, 61.27218654211062);
        assert_eq!(model.r2, 0.989196922445796);
    }
}
