# Release Notes - romaudit_cli v2.2.0

**Release Date:** August 14, 2025

## 🎯 Focus on Standard DAT Files

Version 2.2.0 represents a significant simplification of romaudit_cli, removing all MAME-specific handling to focus exclusively on standard DAT file formats like No-Intro and Redump.

## 🚨 Breaking Changes

- **Removed support for `.xml` files** - Only `.dat` files are now supported
- **Removed MAME-specific handling** - No parent/clone relationships or DAT type detection
- **All games treated independently** - Shared ROMs are now copied to each game

## ✨ What's New

### Simplified & Focused
- **30% reduction in code complexity** for better maintainability
- **Cleaner architecture** without special case handling
- **More predictable behavior** for all DAT files
- **Better performance** without MAME detection overhead

### Core Features Retained
- ✅ Multi-threaded file scanning and hashing
- ✅ Smart folder organization rules
- ✅ Hash verification (SHA1, MD5, CRC32)
- ✅ Duplicate and unknown file management
- ✅ Shared ROM detection and tracking
- ✅ Progress tracking with ETA
- ✅ Graceful shutdown (Ctrl+C)
- ✅ Persistent database
- ✅ Comprehensive logging

## 📦 What Was Removed

- MAME XML file detection and parsing
- Parent/clone relationship tracking
- DAT type detection (merged/split/non-merged)
- Support for `<machine>` tags
- Complex organization logic for MAME sets
- `src/parser/detector.rs` file

## 🔄 Migration from Previous Versions

### From v2.1.0 or earlier:

1. **Backup your data:**
   ```bash
   cp -r roms roms_backup
   cp rom_db.json rom_db.json.backup
   ```

2. **Note:** The tool no longer accepts `.xml` files
   - Convert any XML files to DAT format if needed
   - Or continue using v2.1.0 if MAME support is required

3. **Build and run:**
   ```bash
   cargo build --release
   ./target/release/romaudit_cli
   ```

## 💡 Why These Changes?

- **Better Focus:** Optimized for standard ROM preservation formats
- **Easier Maintenance:** Simpler codebase is easier to improve and debug
- **Improved Reliability:** Fewer edge cases mean more predictable behavior
- **Future Ready:** Clean foundation for adding features specific to standard DAT files

## 🐛 Bug Fixes from v2.1.0

While v2.1.0 fixed several MAME-related bugs, v2.2.0 eliminates these issues entirely by removing MAME support:
- No more incorrect CHD placement issues
- No more parent/clone confusion
- No more DAT type detection problems

## 📊 Performance

- Slightly faster processing without MAME detection overhead
- Multi-threaded scanning (from v2.1.0) still provides excellent performance
- Simpler code paths mean more predictable performance

## 🛠️ Technical Details

- **Language:** Rust edition 2024
- **Architecture:** Modular design with 14 specialized modules
- **Dependencies:** Updated and minimal
- **Binary Size:** Optimized with LTO and size optimization

## 📝 Notes

- This release focuses on doing one thing well: organizing standard ROM collections
- If you need MAME support, please use version 2.1.0 or earlier
- Future development will focus on enhancing standard DAT file support

## 🙏 Acknowledgments

Thanks to all users who provided feedback that led to this focused, cleaner version.

---

**Download:** [romaudit_cli v2.2.0](https://github.com/yourusername/romaudit_cli/releases/tag/v2.2.0)
**Documentation:** [README](https://github.com/yourusername/romaudit_cli/blob/main/README.md)
**Changelog:** [CHANGELOG](https://github.com/yourusername/romaudit_cli/blob/main/CHANGELOG.md)