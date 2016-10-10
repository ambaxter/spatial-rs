# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/) 
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]
### Added
- Added rust-geo dependency and leafgeometry calculations

### Changed

### Fixed

## [0.2.0-rc2] - 2016-09-27
### Changed
- Updated itertools to v0.5

## [0.2.0-rc1] - 2016-07-29
### Added
- R-Tree with Quadratic and Linear seed picking options

### Changed
- Removed parking_lot dependency

### Fixed 
- iter\_mut and query\_mut now take `&mut self` to provide isolation guarantees

## [0.1.1] - 2016-07-07
### Fixed
- Splitting root resulted in incorrect MBRs

## [0.1.0] - 2016-06-27
### Added
- R* Tree