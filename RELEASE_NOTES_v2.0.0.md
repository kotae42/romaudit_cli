# romaudit_cli Version 2.0.0

## Release Date: August 11, 2025

## Overview

Version 2.0.0 represents a complete architectural redesign of romaudit_cli, transforming it from a monolithic application into a clean, modular architecture. Despite this major internal change, it maintains **100% compatibility** with all previous versions.

## Why 2.0.0?

Following [Semantic Versioning](https://semver.org/):
- **Major version (2)**: Significant architectural changes
- **Minor version (0)**: No new features added beyond v1.6.4
- **Patch version (0)**: Fresh major release

## Key Changes

### Architectural Transformation

**Before (v1.x):**
- Single 2000+ line `main.rs` file
- All functionality mixed together
- Difficult to maintain and test
- Hard to add new features without breaking existing ones

**After (v2.0.0):**
- 14 specialized modules
- Average module size: ~90 lines
- Clear separation of concerns
- Each module has a single responsibility

### Module Structure

```
src/
├── main.rs          # Entry point (106 lines)
├── config.rs        # Configuration (41 lines)
├── error.rs         # Error handling (52 lines)
├── types.rs         # Shared types (57 lines)
├── parser/          # DAT/XML parsing
├── scanner/         # File scanning
├── organizer/       # ROM organization
├── database/        # Persistence
└── logger/          # Report generation
```

## What This Means for Users

### Nothing Changes!

- ✅ Same command-line interface
- ✅ Same configuration format
- ✅ Same database format
- ✅ Same output and logs
- ✅ Same performance
- ✅ All v1.6.4 features intact

### Just Better

- 🚀 Faster bug fixes
- 🛡️ More stable (isolated modules)
- 📊 Better error messages
- 🔧 Easier to customize

## What This Means for Developers

### Easier Contributions

- Add new DAT formats without touching existing parsers
- Add archive support without affecting organization logic
- Fix bugs in isolation
- Test individual components

### Better Code Quality

- Strong typing throughout
- Clear module boundaries
- Single responsibility principle
- Improved error handling

## Migration

**No migration needed!** Just replace your binary:

```bash
# Build v2.0.0
cargo build --release

# Use exactly as before
./target/release/romaudit_cli
```

Your existing:
- `rom_db.json` - Works as-is
- `config.toml` - Works as-is
- Directory structure - Unchanged
- Workflow - Unchanged

## Features (All Preserved from v1.6.4)

- ✅ DAT/XML file support
- ✅ MAME DAT type detection (merged/split/non-merged)
- ✅ Smart organization rules
- ✅ Hash verification (SHA1/MD5/CRC32)
- ✅ Duplicate/unknown handling
- ✅ Progress tracking with ETA
- ✅ Graceful shutdown (Ctrl+C)
- ✅ Database persistence
- ✅ Comprehensive logging
- ✅ Parent/clone relationships

## Future Benefits

The modular architecture enables:

- **Easier Testing**: Each module can be unit tested
- **Parallel Processing**: Add to scanner module only
- **Archive Support**: Modify scanner without touching organizer
- **New DAT Formats**: Just add new parser implementations
- **GUI Frontend**: Use modules as a library
- **Better Debugging**: Issues isolated to specific modules

## Performance

No performance impact! The modular version:
- Compiles to the same optimized binary
- Uses identical algorithms
- Maintains all v1.6.4 optimizations
- Same memory usage

## Support

- Issues: GitHub Issues
- Documentation: See README.md
- Migration Help: See MIGRATION.md

## Credits

- Original monolithic design: v1.0.0 - v1.6.4
- Modular refactoring: v2.0.0
- Maintained by: [Kotae42]

## License

Personal use only. Commercial use prohibited.
See LICENSE file for details.

---

*"Same tool, better foundation."*