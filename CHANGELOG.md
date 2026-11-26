# Changelog

All notable changes to this project are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-11-26

### Added

- Discord Rich Presence for the track playing in Aurral's web player, read from `GET /api/play-queue` with a personal API key.
- Pause and resume detection from the player's queue saves; the status clears on pause and returns on resume.
- `config.json` customization: activity type, member-list text, name, text templates with `{title}`, `{artist}`, `{album}` and `{duration}`, images, artwork and timestamp toggles, up to two buttons and the poll interval.
- Album art from the iTunes Search API with a configurable fallback image.
- Per-user settings folder with `--init`, plus `--help` and `--version`.
- Prebuilt binaries for macOS, Linux and Windows on GitHub Releases.

[Unreleased]: https://github.com/samarth3301/aurral-presence/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/samarth3301/aurral-presence/releases/tag/v0.1.0
