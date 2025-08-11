# Version 2.0.1 Release Notes

## Release Date: January 2025

## Summary
Version 2.0.1 introduces critical improvements for MAME ROM collection handling while maintaining backward compatibility with existing functionality.

## Key Changes

### MAME XML Support Improvements
The most significant change in this release is the separation of handling logic for MAME XML files versus standard DAT files:

#### For MAME XML Files:
- **Literal Parsing**: Each game entry is parsed exactly as specified in the XML
- **No Parent/Clone Sharing**: Files are not shared between games
- **No Folder Creation**: Only existing folders are organized, no new folders created
- **User Choice**: Users select the appropriate XML type for their collection:
  - `merged.xml` for merged sets
  - `split.xml` for split sets
  - `non-merged.xml` for non-merged sets

#### For Standard DAT Files:
- **Intelligent Handling Maintained**: Parent/clone relationships still recognized
- **Shared File Support**: Automatic handling of files shared between games
- **Missing File Resolution**: Can copy missing files from parent sets

## Problem Solved
This update specifically addresses issues with MAME non-merged collections where:
- Spurious folders were being created for clone games
- Files were incorrectly copied between folders
- Individual ROM files were treated as complete sets

## Technical Details

### New Detection Logic
```rust
pub fn is_mame_xml(content: &str) -> bool {
    content.contains("<mame build=") || 
    content.contains("<!DOCTYPE mame [") ||
    content.contains("<machine name=") && content.contains("romof=")
}
```

### Separate Processing Paths
- MAME collections use `organize_mame_collection()`
- Standard collections use `organize_standard_collection()`

## Migration Guide

### For Users:
1. Update to version 2.0.1
2. Ensure you're using the correct MAME XML for your collection type
3. Re-run organization if you experienced issues with 2.0.0

### For Developers:
- Check `DatFile.is_mame_dat` field to determine processing logic
- Use appropriate organization method based on DAT type

## Compatibility
- Fully backward compatible with v2.0.0
- No configuration changes required
- Existing organized collections remain valid

## Performance
- Slightly faster processing for MAME collections (no parent/clone resolution)
- Memory usage unchanged

## Testing
- Tested with MAME 0.279 XML files (merged, split, non-merged)
- Verified with various standard DAT formats
- No regression in existing functionality

## Future Improvements
- Consider adding explicit user option to force DAT type detection
- Potential for parallel processing of independent MAME games
- Enhanced reporting for MAME vs standard organization

## Bug Fixes
- Fixed: Spurious folder creation in MAME non-merged collections
- Fixed: Incorrect file copying between MAME game folders
- Fixed: Individual ROM files treated as complete game sets

## Known Issues
None at this time.

## Acknowledgments
Thanks to users who reported issues with MAME non-merged collection handling.
