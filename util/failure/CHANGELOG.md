# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## Unreleased
### Changed
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
