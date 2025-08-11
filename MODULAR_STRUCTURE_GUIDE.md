# Module Structure Guide - Version 2.0.0

## Overview

romaudit_cli v2.0.0 features a complete modular architecture redesign for better maintainability, testing, and feature development. Each module has a single responsibility and clear interfaces.

## Module Hierarchy

```
src/
├── main.rs           # Entry point, orchestration
├── config.rs         # Configuration management
├── error.rs          # Error types and handling  
├── types.rs          # Shared data structures
├── parser/           # DAT/XML parsing
│   ├── mod.rs        # Parser trait and orchestration
│   ├── xml.rs        # XML/DAT parser implementation
│   └── detector.rs   # DAT type detection
├── scanner/          # File scanning and hashing
│   ├── mod.rs        # Scanner orchestration
│   ├── hasher.rs     # Hash calculation
│   └── collector.rs  # File collection
├── organizer/        # ROM organization
│   ├── mod.rs        # Organizer orchestration
│   ├── rules.rs      # Organization rules
│   ├── processor.rs  # File processing logic
│   └── folders.rs    # Folder management
├── database/         # Persistence
│   └── mod.rs        # JSON database operations
└── logger/           # Logging and reports
    └── mod.rs        # Log generation
```

## Module Responsibilities

### `main.rs`
- Application entry point
- Signal handling (Ctrl+C)
- High-level orchestration
- Error handling

### `config.rs`
- Configuration structure
- Default values (always available)
- Config file loading (optional - requires uncommenting toml dependency)
- Works without any config file using sensible defaults

### `error.rs`
- Custom error types
- Error conversions
- Result type alias

### `types.rs`
- Shared data structures (RomEntry, RomHashes, etc.)
- Type aliases (RomDb, KnownRoms)
- DAT type enumeration
- Scan results structure

### `parser/`
#### `parser/mod.rs`
- Parser trait definition
- DAT file discovery
- Parser selection

#### `parser/xml.rs`
- XML/DAT parsing implementation
- No-Intro format support
- MAME format support
- Header parsing for DAT type detection

#### `parser/detector.rs`
- DAT type detection from filename
- DAT type detection from content
- DAT type display names

### `scanner/`
#### `scanner/mod.rs`
- File scanning orchestration
- Progress tracking
- Game identification
- Interruption handling

#### `scanner/hasher.rs`
- SHA1 calculation
- MD5 calculation
- CRC32 calculation
- Buffered file reading

#### `scanner/collector.rs`
- Recursive file collection
- File filtering (skip DAT/XML, temp files)
- Generated directory detection

### `organizer/`
#### `organizer/mod.rs`
- Organization orchestration
- Progress tracking
- Result aggregation

#### `organizer/rules.rs`
- Folder requirement detection
- ROM name similarity checking
- Base name extraction
- Word similarity algorithms

#### `organizer/processor.rs`
- Individual file processing
- DAT type-specific filtering
- ROM placement logic
- Duplicate/unknown handling

#### `organizer/folders.rs`
- Numbered folder creation
- Empty folder removal
- Folder traversal

### `database/`
#### `database/mod.rs`
- Known ROMs loading
- Known ROMs saving
- JSON serialization
- Atomic file operations

### `logger/`
#### `logger/mod.rs`
- Have/missing log generation
- Shared ROMs log
- Folders log
- Summary statistics

## Adding New Features

### To add a new DAT format:
1. Create a new parser in `parser/` (e.g., `parser/tosec.rs`)
2. Implement the `DatParser` trait
3. Update `parser/mod.rs` to use the new parser

### To add ZIP support:
1. Add zip crate to dependencies
2. Create `scanner/archive.rs` for archive handling
3. Update `scanner/collector.rs` to handle archives
4. Update `scanner/hasher.rs` to hash from memory

### To add parallel processing:
1. Add rayon crate to dependencies
2. Update `scanner/mod.rs` to use parallel iteration
3. Ensure thread-safe access to shared state

### To add a new database format:
1. Create `database/sqlite.rs` (or other format)
2. Define a trait in `database/mod.rs`
3. Implement for both JSON and new format

### To add command-line arguments:
1. Add clap crate to dependencies
2. Parse arguments in `main.rs`
3. Override config values as needed

## Testing

The modular structure enables unit testing of individual components:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hash_calculation() {
        // Test hasher module
    }
    
    #[test]
    fn test_rom_name_similarity() {
        // Test rules module
    }
    
    #[test]
    fn test_dat_parsing() {
        // Test parser module
    }
}
```

## Benefits of Modular Structure

1. **Maintainability**: Each module has a single, clear purpose
2. **Testability**: Components can be tested in isolation
3. **Reusability**: Modules can be used in different contexts
4. **Parallel Development**: Multiple developers can work on different modules
5. **Feature Isolation**: New features don't affect existing code
6. **Debugging**: Issues are isolated to specific modules
7. **Documentation**: Each module documents its own functionality

## Migration from Monolithic Code

The functionality remains identical to the monolithic version, but the code is now organized into logical modules. All existing features are preserved:

- DAT/XML parsing
- MAME DAT type detection
- Hash calculation
- Smart organization rules
- Duplicate/unknown handling
- Progress tracking
- Signal handling
- Database persistence
- Log generation

## Future Enhancements

The modular structure makes it easier to add:

- Additional DAT formats (TOSEC, Redump, etc.)
- Archive support (ZIP, 7Z, RAR)
- Parallel processing
- Different database backends
- GUI frontend
- Network support
- Cloud storage integration
- Verification-only mode
- Repair functionality