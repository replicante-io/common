# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## Unreleased
### Added
- Add `cluster_display_name` attribute to datastore info.

### Changed
- **BREAKING**: Replace datastore info attribute `cluster` with `cluster_id`.
- **BREAKING**: Replace datastore info attribute `name` with `node_id`.

### Removed
- **BREAKING**: Removed nonsensical ordering on some models.

## 0.2.0 - 2019-02-20
### Changed
- **BREAKING**: Encode shard roles as lower case strings
- **BREAKING**: Replaced shard's `last_op` with a `commit_offset`
- **BREAKING**: Replication lag has a unit (no longer assumed to be seconds)

## 0.1.1 - 2018-06-28
### Fixed
- Updated readme in crate metadata

## 0.1.0 - 2018-06-28
### Added
- Agent info and version models
- Datastore info and status model
- Shard info model
- Shard role enum
