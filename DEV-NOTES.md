# Development Notes

## Current Version: 2.1.0

### Recent Changes (v2.1.0)
- **Performance**: Implemented multi-threaded file scanning and hashing using the `rayon` crate. This provides a significant performance boost on multi-core systems.
- **Bug Fixes**:
    - Corrected logic for MAME non-merged sets to properly create self-contained game directories.
    - Fixed incorrect placement of MAME CHD (disk) files.
    - Resolved an issue where the scanner could incorrectly skip a user's `roms` directory.
- **Refactoring**:
    - Simplified the complex folder-creation logic for predictability.
    - Made the database saving process more efficient.
- **Dependencies**:
    - Added `rayon = "1.11.0"` for parallel processing.

## License Notice

This software is licensed for personal, non-commercial use only. By working on this project, you agree to maintain this licensing model. See LICENSE file for details.

## Building romaudit_cli

### Recent Changes (v2.0.0)
- **MAJOR**: Complete architectural refactoring to modular design
- Code split into 14 specialized modules
- Improved maintainability and testability
- 100% backward compatible with v1.x

### Recent Changes (v1.6.4)
- **MAME DAT type detection**: Automatically detects merged/split/non-merged
- Space-efficient handling for split and merged sets
- Parent/clone relationship tracking
- DAT type-specific organization logic

### Recent Changes (v1.6.3)
- **CRITICAL FIX**: Tool now only organizes games actually present in collection
- **PERFORMANCE**: Single-pass scanning with cached hashes (massive speed improvement)
- Added progress bar with ETA for better user feedback
- Prevents creation of incomplete ROM folders for games not in collection
- ROMs for absent games are now properly treated as unknown

### Recent Changes (v1.6.2)
- Added support for XML files in addition to DAT files
- Implemented graceful shutdown handling with Ctrl+C
- Added ctrlc dependency (v3.4.7) for signal handling  
- Optimized parsing for large MAME XML files (45MB+)
- Added adaptive buffer sizing based on file size

### Recent Changes (v1.6.1)
- Simplified folder organization logic
- Removed system file pattern detection
- Fixed overly aggressive folder creation

### Prerequisites
- Rust 1.75+ with edition 2024 support (install from https://rustup.rs/)
- Cargo (included with Rust)

### Quick Build
```bash
cargo build --release
```

### Dependencies Note

The project uses `md-5` crate (with hyphen) in Cargo.toml, but imports as `md5` (without hyphen) in the code. This is correct - the package name and the crate name can differ.

New dependency added in v1.6.2:
- `ctrlc = "3.4.7"` - For graceful shutdown handling

### Rust Edition

The project uses Rust edition 2024, which provides the latest language features and improvements. This edition includes:
- Enhanced pattern matching capabilities
- Improved async/await syntax
- Better const evaluation
- More ergonomic error handling

Make sure your Rust toolchain is up to date with `rustup update` to use edition 2024.

### Performance Optimizations

The release profile is optimized for small binary size:
- `opt-level = "z"` - Optimize for size
- `lto = true` - Link Time Optimization
- `codegen-units = 1` - Better optimization at cost of compile time
- `strip = true` - Strip symbols from binary
- `panic = "abort"` - Smaller binary, no unwinding

Additional optimizations in v1.6.2:
- Adaptive buffer sizing for XML parsing (8MB for files >10MB)
- Progress indicators for large file parsing

### Configuration

The project has a simple configuration structure that allows customization of:
- Directory names (rom_dir, logs_dir)
- File names (db_file)
- Folder prefixes (duplicate_prefix, unknown_prefix)
- Buffer size for hashing
- Stop words for name comparison

### Signal Handling (New in v1.6.2)

The application now properly handles interruption signals:
- **Ctrl+C**: Triggers graceful shutdown, saves progress to `rom_db.json`
- Uses `Arc<AtomicBool>` for thread-safe communication
- Progress is preserved - can continue from where it left off

### Development Tips

1. For faster iterative development:
   ```bash
   cargo run
   ```

2. For testing with optimizations:
   ```bash
   cargo run --release
   ```

3. Check for warnings:
   ```bash
   cargo clippy
   ```

4. Format code:
   ```bash
   cargo fmt
   ```

5. Testing with large XML files:
   - The tool now shows progress for files >5MB
   - Uses larger buffers (8MB) for files >10MB
   - Test with MAME XML files (45MB+) to verify performance

### Project Structure

```
romaudit_cli/
├── src/
│   └── main.rs         # All source code in single file
├── Cargo.toml          # Project manifest
├── Cargo.lock          # Dependency lock file (generated)
├── README.md           # User documentation
├── LICENSE             # Personal use license
├── LICENSE-FAQ.md      # License questions and answers
├── NOTICE              # License notice
├── CHANGELOG.md        # Version history
├── MIGRATION.md        # Upgrade guide
├── DEV-NOTES.md        # This file
├── CONTRIBUTING.md     # Contribution guidelines
├── Example-config.toml # Configuration example
└── target/            # Build artifacts (git ignored)
```

### File Format Support

As of v1.6.2, the tool supports:
- `.dat` files (No-Intro format)
- `.xml` files (MAME format)
- Case-insensitive extension matching
- Automatic detection of first DAT/XML file in directory

### Testing

Currently, the project has no automated tests. Consider adding:
- Unit tests for hash calculation
- Integration tests for DAT/XML parsing
- Tests for file organization logic
- Tests for signal handling
- Tests for large file handling

### Publishing Note

This project uses a personal-use-only license and should NOT be published to crates.io, as it would violate their terms of service which require open source licenses. Distribution should be through:
- GitHub releases
- Direct downloads
- Personal repositories

### Future Enhancements

- Add support for compressed ROM files (ZIP, 7Z, RAR)
- Split code into modules for better organization
- Add command-line arguments for configuration
- Support for reading config.toml
- Parallel file processing for better performance
- Support for additional hash algorithms
- GitHub Actions for automated builds and releases
- Cross-platform binary releases
- Automated testing suite
- Support for multiple DAT/XML files simultaneously