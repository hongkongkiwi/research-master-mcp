# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.2] - 2026-01-23

### Added
- Europe PMC source implementation for searching European PubMed Central
- lopdf fallback for PDF extraction (when poppler unavailable)
- Debian, RedHat, and Alpine package builds to release workflow
- Docker and GitHub Packages publishing to release workflow
- is-terminal crate for terminal detection (replaces atty)
- cargo-husky for Rust-native git hooks

### Fixed
- Replace atty with is-terminal crate
- Code formatting applied to recent changes
- macOS runner updated from deprecated macos-13 to macos-14

## [0.1.0] - 2024-01-XX

### Added
- Initial release
