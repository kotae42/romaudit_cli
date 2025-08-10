# Changelog

All notable changes to romaudit_cli will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.6.2] - 2025-08-11

### Added
- Support for XML files in addition to DAT files (automatically detects both)
- Graceful shutdown handling for Ctrl+C interruption
- Progress indicator for parsing large DAT/XML files (45MB+ MAME files)
- Adaptive buffer sizing for improved performance with large files

### Fixed
- Application no longer hangs in memory when interrupted with Ctrl+C
- Improved handling of large MAME XML files

### Changed
- File detection now uses case-insensitive matching for .dat and .xml extensions
- Added ctrlc dependency (v3.4.7) for signal handling
- Optimized XML parsing with larger buffers for files over 10MB

## [1.6.1] - 2025-08-05

### Fixed
- Removed overly aggressive folder creation for system/BIOS files
- Single ROM files with matching names now correctly placed directly in roms/ folder
- Fixed issue where files with "[BIOS]" or "kiosk" in name were unnecessarily put in folders

### Changed
- Simplified folder organization logic - folders only created when truly needed:
  - Multiple ROM files
  - Single ROM with name mismatch
- Removed system file pattern detection entirely

## [1.6.0] - 2025-08-02

Initial release of romaudit_cli.

### Features

- Automatic ROM organization based on DAT files
- Support for No-Intro and MAME style DAT formats
- Intelligent folder organization rules
- Multi-hash verification (CRC32, MD5, SHA1)
- Duplicate and unknown file management
- Shared ROM detection and tracking
- Persistent ROM database
- Comprehensive logging system
- Progress bar for scan operations
- Fully configurable paths and settings
- Zero hardcoded values
- System/BIOS files always placed in folders
- Multi-part games automatically organized in folders
- Single ROMs with mismatched names get dedicated folders
- Preserves internal folder structures from DAT files
- Atomic database updates to prevent corruption
- Empty folder cleanup after operations
- Uses Rust edition 2024 for latest language features
- Optimized release profile with size optimization

### Known Limitations

- No support for compressed ROM files (ZIP, 7Z, RAR, etc.) - ROMs must be extracted first
- Licensed for personal use only - commercial use prohibited