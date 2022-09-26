<!-- markdownlint-disable MD022 MD024 MD032 -->
# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## Unreleased
### Changed
- Remove `failure` and `replicante_utils_failure` dependency.
- Updated dependencies.

### Removed
- **BREAKING** Previously deprecated `request_span` has been removed.
- **BREAKING** The `sentry` module was removed in favour of `sentry-actix` crate.

## 0.2.1 - 2022-09-12
### Changed
- Removed needless `drop` as warned by clippy.

## 0.2.0 - 2020-05-28
### Added
- ActixWeb late `AppConfig` manager.

### Changed
- **BREAKING**: Removed `RootDescriptor::resource` (use `AppConfigContext::scoped_service`).
- **BREAKING**: Update dependencies.

## 0.1.0 - 2020-03-07
### Added
- Logging middleware.
- Metrics middleware and exposition helper.
- Sentry middleware.
- Tracing middleware.
