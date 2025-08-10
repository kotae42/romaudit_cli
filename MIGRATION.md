# Migration Guide

This guide helps you upgrade romaudit_cli between versions.

## Upgrading to 1.6.2

### From 1.6.1
This is a smooth upgrade with no breaking changes:

1. **New Features**:
   - XML file support added - no changes needed, automatically detects .xml files
   - Graceful shutdown with Ctrl+C - no configuration required
   - Better handling of large files - automatic optimization

2. **No Action Required**:
   - Your existing `rom_db.json` remains compatible
   - Folder organization rules unchanged
   - Configuration remains the same

3. **Build Changes**:
   - New dependency: `ctrlc = "3.4.7"`
   - Run `cargo update` before building

### From 1.6.0
Follow the 1.6.1 migration steps first, then upgrade to 1.6.2.

## Upgrading to 1.6.1

### From 1.6.0
**Important**: This version changes folder organization behavior.

1. **Folder Organization Changes**:
   - BIOS and system files no longer automatically get folders
   - Only multi-file games or mismatched names trigger folder creation
   - Some single-file games may move from folders to direct placement

2. **Before Upgrading**:
   - Back up your current `roms/` directory
   - Note which games are currently in folders

3. **After Upgrading**:
   - Run the tool - it will reorganize based on new rules
   - Check `logs/folders.txt` to see which games use folders
   - Verify your collection is organized as expected

4. **Reverting** (if needed):
   - Restore your backup
   - Use version 1.6.0 if you prefer the old behavior

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

### DAT/XML File Compatibility
- All versions support standard No-Intro DAT format
- All versions support MAME format (DAT or XML)
- v1.6.2+ automatically detects both .dat and .xml files

### Configuration Compatibility
- Configuration structure unchanged across all 1.6.x versions
- Custom configurations remain compatible

## Troubleshooting Upgrades

### Issue: ROMs moved unexpectedly
**Solution**: This is expected when upgrading to 1.6.1. The new logic is more intelligent about folder usage. Check `logs/folders.txt` for the current organization.

### Issue: Build fails with dependency errors
**Solution**: Run `cargo update` to refresh dependencies, especially when upgrading to 1.6.2 (adds ctrlc).

### Issue: Tool doesn't find XML files
**Solution**: Ensure you're using v1.6.2 or later. Earlier versions only support .dat files.

### Issue: Process won't stop cleanly
**Solution**: v1.6.2 adds proper Ctrl+C handling. Earlier versions may need to be force-killed.

## Downgrading

If you need to downgrade:

1. **Restore Backups**:
   ```bash
   mv roms_backup roms
   mv rom_db.json.backup rom_db.json
   ```

2. **Checkout Previous Version**:
   ```bash
   git checkout v1.6.1  # or v1.6.0
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