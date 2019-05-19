# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## Unreleased
### Added
- Zipkin HTTP transport support.

### Changed
- **BREAKING**: Remove service name from configuration (code should set it).
- **BREAKING**: Replace `ReporterThread` with `humthreads` and `Upkeep`.
- **BREAKING**: Zipkin configuration supports (and defaults to) HTTP transport.
- Use `capture_fail!` to handle errors from the reporter thread.

## 0.2.0 - 2019-03-29
### Changed
- **BREAKING**: Convert `error-chain` to `failure`.

## 0.1.1 - 2018-06-28
### Fixed
- Updated readme in crate metadata

## 0.1.0 - 2018-06-28
### Added
- Tracers configuration
- Support for noop tracer
- Support for zipkin tracer
