## [0.4.0] - UNRELEASED

### Removed
- Removed `GeometricSequence` struct

### Changed
- Upgraded to Rust 2018
- Replaced `regression` function with `Model` struct
- Replaced `Statistics` trait with `Kahan` trait
- Replaced `Stopwatch::new` method with `Default` implementation

## [0.3.3] - 2018-08-18

### Removed
- Removed optional `clippy` dependency

## [0.3.2] - 2017-02-28

### Fixed
- Fixed integer overflow caused by benchmarking extremely fast functions

## [0.3.1] - 2017-01-29

### Added
- Added support for the `stable` and `beta` channels of Rust

### Changed
- Renamed `maximum` option to `time`

## [0.3.0] - 2016-12-28

### Added
- Added `memory` option
- Added specialized benchmarking functions (`bench_drop` and `bench_setup`)
- Added specialized measuring functions (`measure_drop` and `measure_setup`)

### Removed
- Removed `time` dependency
- Removed `GeometricSequence` struct
- Removed `Stopwatch` struct

### Changed
- Renamed `maximum` option to `time`

## [0.2.1] - 2016-12-26

### Changed
- Improved accuracy of results by using the Kahan summation algorithm

## [0.2.0] - 2016-12-25

### Added
- Added `alpha` field to `Anaylsis` struct

### Changed
- Improved accuracy of results by using the Kahan summation algorithm
- Improved formatting of printed results

## [0.1.0] - 2016-12-24
- Initial release
