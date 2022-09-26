<!-- markdownlint-disable MD022 MD024 MD032 -->
# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## Unreleased
## Changed
- Deprecation notice.

## 0.1.4 - 2022-09-12
### Changed
- Refactor formatting from clippy warnings.

## 0.1.3 - 2020-05-28
### Changed
- Update dependencies.

## 0.1.2 - 2020-03-07
### Changed
- Add optional `variant` attribute to `SerializableFail`.
- Serialisation of `&Fail` instead of `Fail`.

## 0.1.1 - 2019-06-16
### Added
- Helper function to capture `Fail`s.

### Changed
- Add error and cause names to `failure_info`.

## 0.1.0 - 2019-03-29
### Added
- `Fail` to `IronError` conversion utility. 
- Helper function to log structured error information.
- Helper function to report error in main.
