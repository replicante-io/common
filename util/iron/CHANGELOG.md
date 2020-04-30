# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## Unreleased
### Changed
- **BREAKING**: Update dependencies.

## 0.2.0 - 2019-06-16
### Added
- Advanced `Router`.
- Helper to convert opentracing error into an Iron error.
- Sentry error response middlewere.
- `Router` support for OpenTracing span creation and propagation.

### Removed
- **BREAKING**: Headers tracing moved to `replicante_util_tracing::carriers::iron`.

## 0.1.3 - 2019-03-29
### Added
- `Fail` to `IronError` conversion utility. 

## 0.1.1 - 2018-06-28
### Fixed
- Updated readme in crate metadata

## 0.1.0 - 2018-06-28
### Added
- OpenTracingRust carrier for Iron headers.
- Middleware for handler metrics collection.
- MetricsHandler to expose prometheus metrics.
- Request logging middleware.
