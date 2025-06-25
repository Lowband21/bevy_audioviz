# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- GitHub Actions CI/CD workflow for automated testing and cross-platform builds
- CHANGELOG.md for tracking version history  
- Comprehensive unit tests for audio_processing module (12 test cases)
- rustfmt.toml and clippy.toml for consistent code formatting and linting
- GitHub issue templates for bug reports and feature requests
- Proper logging system using `log` and `env_logger` crates

### Changed
- **MAJOR**: Updated Bevy from 0.13.0 to 0.16.1 (3 major version upgrade)
- **MAJOR**: Updated bevy_egui from 0.25.0 to 0.34.0
- Updated spectrum-analyzer from 1.5.0 to 1.7.0
- Updated cpal from 0.15.2 to 0.15.3
- Migrated from `MaterialMesh2dBundle` to component-based spawning system
- Replaced all `println!` statements with proper structured logging
- Updated Color API from `Color::Rgba` to `Color::srgba()`
- Migrated window and entity query APIs to new Bevy 0.16 patterns

### Fixed
- All breaking changes from Bevy 0.13 → 0.16 migration
- Audio processing error handling with proper logging
- Window resizing and query compatibility issues
- Plugin initialization for new Bevy version

## [0.1.0] - 2024-02-17

### Added
- Initial release with 5 visualization modes (bar, string, circle split, polygon, wave)
- GUI integration with egui for real-time parameter adjustment
- Support for multiple audio input/output devices
- Shader hot reloading in debug mode
- Cross-platform support (Windows, Linux, macOS)
- Configurable audio processing parameters (smoothing, interpolation, frequency range)
- Low-latency audio processing (<0.5ms per frame)

### Changed
- Updated to Bevy 0.13
- Updated bevy_egui to 0.25.0
- Improved shader binding groups

### Fixed
- Linux audio capture using pactl source output
- Disabled asset watcher for release builds (AUR compatibility)
- Windows-specific audio capture issues
- Audio gating for signals below threshold

## Previous Versions

### Notable Updates
- **Bevy 0.13 Update**: Fixed shader binding groups and removed globals binding
- **Linux Support**: Improved audio device detection using binary name
- **Performance**: Added optimized release profile configurations
- **UI Improvements**: Added macro-based UI generation for cleaner code