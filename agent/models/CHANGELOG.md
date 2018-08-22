# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Changed
- **BREAKING**: Replaced shard's `last_op` with a `commit_offset`.
- **BREAKING**: Replication lag has a unit (no longer assumed to be seconds).

## [0.1.1] - 2018-06-28
### Fixed
- Updated readme in crate metadata

## 0.1.0 - 2018-06-28
### Added
- Agent info and version models
- Datastore info and status model
- Shard info model
- Shard role enum


[Unreleased]: https://github.com/replicante-io/common/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/replicante-io/common/compare/v0.1.0...v0.1.1
