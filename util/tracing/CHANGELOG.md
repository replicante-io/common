<!-- markdownlint-disable MD022 MD024 MD032 -->
# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## Unreleased
### Changed
- Remove `replicante_utils_failure` dependency.
- Updated dependencies.

## 0.4.1 - 2022-09-12
### Changed
- Fix new warnings from clippy.

## 0.4.0 - 2020-05-28
### Changed
- **BREAKING**: Drop support for kafka collector.
- Update dependencies.

## 0.3.2 - 2020-03-07
### Added
- `MaybeTracer` type to more easily support optional tracers.

## 0.3.1 - 2019-07-15
### Added
- Span collector for the noop tracer (to silence confusing error).

## 0.3.0 - 2019-06-16
### Added
- Iron and reqwest headers carries.
- Helper to fail spans on error.
- Zipkin HTTP transport support.

### Fixed
- Error encoding in `fail_span` no longer breaks HTTP reporter.

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
