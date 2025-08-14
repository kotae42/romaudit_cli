# Migration Guide

This guide helps you upgrade romaudit_cli between versions.

## Upgrading to 2.2.0

### From 2.1.0 or earlier
**BREAKING CHANGES**: This version removes all MAME-specific functionality to focus on standard DAT files.

1. **What Changed**:
   - **Removed**: All MAME XML support
   - **Removed**: Support for `.xml` files (only `.dat` files now)
   - **Removed**: Parent/clone relationship handling
   - **Removed**: DAT type detection (merged/split/non-merged)
   - **Simplified**: All games treated independently
   - **Simplified**: Shared ROMs always copied to each game

2. **Before Upgrading**:
   - Convert any `.xml` files to `.dat` format if needed
   - Backup your current organization:
     ```bash
     cp -r roms roms_backup
     cp rom_db.json rom_db.json.backup
     ```

3. **Build the New Version**:
   ```bash
     cargo clean
     cargo build --release
     ```

4. **What to Expect**:
   - The tool will no longer recognize `.xml` files
   - No special handling for MAME collections
   - Simpler, more predictable behavior
   - Better performance without MAME detection overhead

5. **Re-organization May Be Needed**:
   - If you previously organized MAME collections, you may want to re-run
   - Shared ROMs will now be copied to each game (no parent/clone optimization)

### Why These Changes?

- **Focus**: Better support for standard ROM collections (No-Intro, Redump, etc.)
- **Simplicity**: ~30% less code to maintain
- **Performance**: Faster processing without MAME detection
- **Reliability**: More predictable behavior for all DAT files

## Upgrading to 2.1.0

### From Any 2.0.x Version
**Performance Enhancement**: Multi-threaded scanning for faster processing.

1. **What's New**:
   - Multi-threaded file scanning and hashing
   - Significant performance improvement on multi-core systems
   - Bug fixes for MAME handling (now removed in 2.2.0)

2. **No Migration Required**:
   - Just replace the binary
   - Your existing `rom_db.json` works as-is

## Upgrading to 2.0.0

### From Any 1.x Version
**Major architectural change with 100% compatibility!**

1. **What Changed**:
   - Complete refactoring from monolithic to modular architecture
   - Code split into 14 specialized modules
   - Better error handling and type safety

2. **What Stayed the Same**:
   - All functionality identical to v1.6.4
   - Same configuration format
   - Same database format (`rom_db.json`)
   - Same command-line usage
   - Same output format

3. **No Migration Required**:
   - Just replace the binary
   - Your existing `rom_db.json` works as-is
   - Your existing configuration works as-is
   - Continue using exactly as before

4. **Benefits You Get**:
   - Faster bug fixes (isolated modules)
   - Easier to add custom features
   - Better error messages
   - More stable codebase

## General Upgrade Process

### For Any Version

1. **Backup Your Data**:
   ```bash
   cp -r roms roms_backup
   cp rom_db.json rom_db.json.backup
   ```

2. **Get the New Version**:
   ```bash
   git pull
   cargo build --release
   ```

3. **Test First**:
   - Run in a test directory with a small subset of ROMs
   - Verify the organization meets your needs

4. **Run the Upgrade**:
   ```bash
   ./target/release/romaudit_cli
   ```

5. **Check Results**:
   - Review `logs/` directory for changes
   - Verify ROM organization in `roms/`

## Version Compatibility

### Database Compatibility
- `rom_db.json` is forward-compatible (older databases work with newer versions)
- Always backup before major version upgrades

### DAT File Compatibility
- v2.2.0+: Only supports `.dat` files (XML support removed)
- v1.6.2-v2.1.0: Supports both `.dat` and `.xml` files
- All versions: Support standard No-Intro DAT format

### Configuration Compatibility
- Configuration structure unchanged across all versions
- Custom configurations remain compatible

## Troubleshooting Upgrades

### Issue: XML files not recognized (v2.2.0+)
**Solution**: This is expected. Convert XML files to DAT format or use an earlier version if MAME support is needed.

### Issue: ROMs organized differently
**Solution**: v2.2.0 treats all games independently. Shared ROMs are now copied to each game. This is the expected behavior.

### Issue: Build fails with dependency errors
**Solution**: Run `cargo update` to refresh dependencies.

### Issue: Process won't stop cleanly
**Solution**: All versions since v1.6.2 support proper Ctrl+C handling.

## Downgrading

If you need to downgrade:

1. **Restore Backups**:
   ```bash
   mv roms_backup roms
   mv rom_db.json.backup rom_db.json
   ```

2. **Checkout Previous Version**:
   ```bash
   git checkout v2.1.0  # or desired version
   cargo build --release
   ```

3. **Re-run Organization**:
   ```bash
   ./target/release/romaudit_cli
   ```

## Support

For issues with migration:
1. Check the [CHANGELOG](CHANGELOG.md) for detailed changes
2. Review the [README](README.md) for current features
3. Open an issue on GitHub if you encounter problems