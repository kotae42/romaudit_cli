# Changelog

All notable changes to romaudit_cli will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.6.0] - 2025-08-01

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
- Fully configurable system patterns
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