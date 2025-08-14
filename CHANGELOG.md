# Changelog

All notable changes to romaudit_cli will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.2.0] - 2025-08-14

### Changed
- **BREAKING**: Removed all MAME-specific handling to focus on standard DAT files
- **BREAKING**: Removed support for `.xml` files - now only supports `.dat` files
- Simplified codebase by ~30% for better maintainability
- All games now treated independently (no parent/clone relationships)
- Shared ROMs always copied to each game needing them

### Removed
- MAME XML detection and special handling
- Parent/clone relationship tracking
- DAT type detection (merged/split/non-merged)
- Support for `<machine>` tags
- `cloneof` and `romof` attribute parsing
- `src/parser/detector.rs` file (MAME detection only)

### Improved
- Cleaner, more focused codebase
- Simpler organization logic
- Better performance without MAME detection overhead
- More predictable behavior for all DAT files

## [2.1.0] - 2025-08-14

### Added
- **Performance**: File scanning and hashing is now multi-threaded to take advantage of modern multi-core processors. This significantly speeds up the most time-consuming part of the audit process, especially for large collections.
- Added `rayon` dependency for parallel processing.

### Fixed
- **Critical Bug (MAME)**: Corrected the logic for handling MAME non-merged sets. The tool no longer assumes files are pre-sorted and will correctly copy files to create self-contained game directories from a flat structure.
- **Critical Bug (MAME)**: Fixed incorrect placement of CHD (disk) files. CHDs are now correctly placed within a subdirectory named after the disk inside the game's main folder, as MAME expects.
- **Major Bug**: Fixed an issue where the scanner could ignore the user's entire collection if it was located in a folder named `roms`. The directory skipping logic is now more precise.
- **Logic Simplification**: Replaced the complex, heuristic-based logic for folder creation with a simpler, more predictable rule. A game gets a folder only if it contains multiple files or if its single file's name doesn't match the game's name.
- **Efficiency**: Improved the database saving logic to be more efficient and performant by using sorted maps, avoiding unnecessary computation.

### Changed
- Updated `rayon` dependency to version `1.11.0`.

## [2.0.1] - 2025-01-11

### Changed
- **MAME XML Support**: Separated logic for MAME XML files vs standard DAT files
  - MAME XMLs are now parsed literally without parent/clone file sharing
  - Each game entry in MAME XML is treated as independent
  - Only existing folders are organized, no new folders created for MAME
  - Users choose the appropriate MAME XML (merged/split/non-merged) for their collection
- **Standard DAT Support**: Maintained intelligent parent/clone handling for non-MAME DATs
  - Shared files are still handled automatically
  - Missing files can be copied from parent sets
  - Clone games inherit parent ROMs when needed

### Added
- `is_mame_dat` field to `ParsedDat` struct to distinguish MAME XMLs
- `is_mame_xml()` function to detect MAME-specific XML files
- Separate organization logic paths for MAME vs standard collections
- Clear console output indicating which mode is being used

### Fixed
- Spurious folder creation for MAME non-merged sets
- Incorrect file copying between folders in MAME collections
- Individual ROM files being treated as complete sets
- Parent/clone relationships incorrectly applied to MAME XMLs

## [2.0.0] - 2025-08-11

### Changed
- **MAJOR**: Complete architectural refactoring from monolithic to modular design
- Code split into 14 specialized modules for better maintainability
- Average module size reduced from 2000+ lines to ~90 lines
- Improved separation of concerns with single-responsibility modules

### Added
- Modular architecture with clear boundaries:
  - `parser/` - DAT/XML parsing
  - `scanner/` - File scanning and hashing
  - `organizer/` - ROM organization logic
  - `database/` - Persistence layer
  - `logger/` - Report generation
- Better error handling with centralized error types
- Improved testability - modules can be unit tested independently
- Foundation for easier feature additions

### Compatibility
- **100% functional compatibility with v1.6.4**
- Same configuration format
- Same database format (`rom_db.json`)
- Same command-line interface
- Same output format
- No user-facing changes required

### Technical Improvements
- Better code organization and navigation
- Reduced coupling between components
- Easier debugging with isolated modules
- Simplified contribution process for new features
- Type safety improvements with centralized type definitions

## [1.6.4] - 2025-08-11

### Added
- **MAME DAT type detection**: Automatically detects merged, split, and non-merged DAT types
- **Space-efficient handling for split/merged sets**: No longer duplicates shared ROMs unnecessarily
- Parent/clone relationship tracking for proper MAME organization
- DAT type displayed in console output and logs

### Changed
- Split sets: Clone games no longer get duplicate parent ROMs (saves significant space)
- Merged sets: Clone ROMs stay with parent games only
- Non-merged sets: Continue to create self-contained games (original behavior)
- Better warnings about disk space requirements specific to DAT type

### Fixed
- Massive space waste when using split or merged MAME DATs
- Improved organization logic for MAME parent/clone relationships

## [1.6.3] - 2025-08-11

### Fixed
- **CRITICAL**: Fixed bug where tool created incomplete ROM folders for games not in collection
- Tool now only organizes games that are actually present (have at least one ROM file)
- ROMs that match games not in your collection are now properly treated as unknown
- Prevents creation of hundreds of empty/incomplete folders in MAME collections

### Changed
- **Performance**: Single-pass scanning with cached hashes (no more duplicate hash calculations)
- Improved progress bar with ETA for long operations
- Better feedback showing how many games are present in collection
- More efficient memory usage for large collections

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