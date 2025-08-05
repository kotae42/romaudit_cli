# Development Notes

## Current Version: 1.6.1

## License Notice

This software is licensed for personal, non-commercial use only. By working on this project, you agree to maintain this licensing model. See LICENSE file for details.

## Building romaudit_cli

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

### Configuration

The project has a simple configuration structure that allows customization of:
- Directory names (rom_dir, logs_dir)
- File names (db_file)
- Folder prefixes (duplicate_prefix, unknown_prefix)
- Buffer size for hashing
- Stop words for name comparison

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
├── DEVELOPMENT.md      # This file
└── target/            # Build artifacts (git ignored)
```

### Testing

Currently, the project has no automated tests. Consider adding:
- Unit tests for hash calculation
- Integration tests for DAT parsing
- Tests for file organization logic

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