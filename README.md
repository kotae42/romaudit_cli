# romaudit_cli

[![Version](https://img.shields.io/badge/version-1.6.0-blue.svg)](https://github.com/yourusername/romaudit_cli)
[![License](https://img.shields.io/badge/license-Personal%20Use%20Only-red.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org/)

A powerful command-line ROM collection management tool written in Rust. romaudit_cli helps you organize, verify, and maintain your ROM collections using DAT files, with intelligent folder organization and comprehensive tracking.

**📋 License: Personal Use Only** - Free for personal use. Commercial use prohibited. See [LICENSE](LICENSE) for details.

## Quick Start

```bash
# Ensure you have Rust with edition 2024 support
rustup update

# Clone and build
git clone https://github.com/yourusername/romaudit_cli.git
cd romaudit_cli
cargo build --release

# Extract any compressed ROMs (ZIP, 7Z, etc) first!
# romaudit_cli only works with uncompressed files

# Run in a directory with a .dat file and ROM files
./target/release/romaudit_cli
```

## Features

- **Automatic ROM Organization**: Intelligently organizes ROMs based on configurable rules
- **Multi-format DAT Support**: Works with both No-Intro and MAME style DAT files
- **Smart Folder Management**: 
  - Multi-part games (disks, tracks) automatically placed in folders
  - System/BIOS files always organized in folders
  - Single ROMs with mismatched names get their own folders
- **Hash Verification**: Supports CRC32, MD5, and SHA1 verification
- **Duplicate Detection**: Identifies and manages duplicate ROMs
- **Unknown ROM Handling**: Separates unrecognized files for easy review
- **Shared ROM Tracking**: Identifies ROMs used by multiple games
- **Progress Tracking**: Visual progress bar during scanning
- **Detailed Logging**: Comprehensive logs for all operations
- **Persistent Database**: Maintains ROM database across scans
- **Fully Configurable**: No hardcoded values - everything is customizable
- **Modern Rust**: Uses Rust edition 2024 for latest language features
- **Optimized Performance**: Small binary size with aggressive optimizations

## Limitations

- **No Compressed File Support**: romaudit_cli works only with uncompressed ROM files. ZIP, 7Z, RAR, and other compressed formats are not supported. Extract your ROMs before scanning.

## Installation

### Download Pre-built Binary (Recommended)

Download the latest release for your platform from the [Releases page](https://github.com/yourusername/romaudit_cli/releases).

### Prerequisites for Building

- Rust 1.75 or later (with edition 2024 support)
- Cargo (comes with Rust)

### Building from Source

```bash
git clone https://github.com/yourusername/romaudit_cli.git
cd romaudit_cli
cargo build --release
```

The compiled binary will be in `target/release/romaudit_cli`.

## Usage

### Requirements

- A `.dat` file (ROM database) in the current directory
- ROM files to be organized (can be in subdirectories)
- **Important**: ROM files must be uncompressed. The tool does not support ZIP, 7Z, RAR, or other compressed formats.

### Basic Usage

1. **Prepare your ROMs**:
   - Extract all compressed files (ZIP, 7Z, RAR, etc.)
   - romaudit_cli only processes uncompressed ROM files

2. Place the romaudit_cli executable in a directory containing:
   - A `.dat` file (ROM database)
   - Uncompressed ROM files to be organized

3. Run the program:
   ```bash
   ./romaudit_cli
   ```

3. The program will:
   - Scan all uncompressed files in the current directory and subdirectories
   - Match them against the DAT file
   - Organize them according to the rules
   - Generate detailed logs

### Directory Structure

After running, your directory will be organized as:

```
.
├── roms/                    # Organized ROM files
│   ├── Game Name/          # Multi-part games in folders
│   │   ├── disk1.bin
│   │   └── disk2.bin
│   ├── Single Game.rom     # Single ROMs directly in roms/
│   └── [BIOS] System/      # System files always in folders
│       └── bios.bin
├── logs/                   # Detailed audit logs
│   ├── have.txt           # List of found ROMs
│   ├── missing.txt        # List of missing ROMs
│   ├── shared.txt         # ROMs shared between games
│   └── folders.txt        # Games stored in subfolders
├── duplicates1/           # Duplicate files (if any)
├── unknown1/              # Unrecognized files (if any)
├── rom_db.json           # Persistent ROM database
└── your_dat_file.dat     # Original DAT file
```

## Organization Rules

romaudit_cli follows these intelligent organization rules:

1. **Multiple ROM Files** → Always use folders
   - Example: `roms/Game Name/disk1.bin`, `roms/Game Name/disk2.bin`

2. **System/BIOS Files** → Always use folders (even if single file)
   - Example: `roms/[BIOS] Nintendo DSi/bios7i.bin`

3. **Single ROM (matching name)** → Direct in roms/
   - Example: `roms/Sonic the Hedgehog.md`

4. **Single ROM (different name)** → Use folder
   - Example: `roms/Memory (Japan)/MEMORY.ASF`

5. **ROMs with paths** → Preserve folder structure
   - Example: `roms/Game/folder/file.bin`

## Configuration

romaudit_cli uses sensible defaults but is fully configurable. You can modify:

### Directory Names
```rust
config.rom_dir = "my_roms".to_string();
config.logs_dir = "audit_logs".to_string();
```

### System File Detection Patterns
```rust
// Add custom patterns for system file detection
config.system_patterns.bracket_patterns.push("[custom]".to_string());
config.system_patterns.prefix_patterns.push("myboot ".to_string());
config.system_patterns.contains_patterns.push("special_firmware".to_string());
```

### Other Settings
- Database filename
- Buffer size for hashing
- Duplicate/unknown folder prefixes
- Stop words for name comparison

## DAT File Support

romaudit_cli supports various DAT file formats:

### No-Intro Style
```xml
<game name="Game Name">
    <rom name="game.rom" size="524288" crc="12345678" md5="..." sha1="..."/>
</game>
```

### MAME Style
```xml
<machine name="Game Name">
    <rom name="game.rom" size="524288" crc="12345678" md5="..." sha1="...">
    </rom>
</machine>
```

## Advanced Features

### Persistent ROM Database

romaudit_cli maintains a `rom_db.json` file that tracks:
- Which games you have
- ROM locations and filenames
- Hash verification data

This allows for fast incremental scans and historical tracking.

### Shared ROM Detection

Some ROMs are identical across multiple games. romaudit_cli:
- Detects these shared ROMs
- Creates separate copies for each game
- Documents sharing relationships in logs

### Smart Name Matching

The name matching algorithm:
- Handles case differences
- Ignores common punctuation variations
- Recognizes region/version suffixes
- Detects significant name mismatches requiring folders

## Example Scenarios

### Scenario 1: Multi-disk Game
```
Input: diskA.bin, diskB.bin (for "Final Fantasy VII")
Output: roms/Final Fantasy VII/diskA.bin
         roms/Final Fantasy VII/diskB.bin
```

### Scenario 2: BIOS Files
```
Input: [BIOS] Nintendo DSi.bin
Output: roms/[BIOS] Nintendo DSi/[BIOS] Nintendo DSi.bin
```

### Scenario 3: Mismatched Names
```
Input: MEMORY.ASF (for game "Memory (Japan)")
Output: roms/Memory (Japan)/MEMORY.ASF
```

## Performance

- **Efficient Hashing**: Uses 1MB buffer for optimal performance
- **Progress Tracking**: Visual feedback during long operations
- **Direct File Access**: Working with uncompressed files provides faster processing
- **Persistent Database**: Speeds up subsequent scans

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## License

**This software is licensed for personal, non-commercial use only.**

romaudit_cli is free to use for personal ROM collection management. Commercial use is strictly prohibited. This includes:
- Using the software in a business environment
- Selling or redistributing the software
- Using the software to provide commercial services
- Including the software in commercial products

See the [LICENSE](LICENSE) file for full details and [LICENSE-FAQ.md](LICENSE-FAQ.md) for common questions.

For commercial licensing, please contact: [your-email@example.com]

## Acknowledgments

- Inspired by ROM management needs in the retro gaming community
- Built with Rust for performance and reliability
- Thanks to all contributors and testers

## Troubleshooting

### No DAT file found
Ensure you have a `.dat` file in the current directory.

### Files not being matched
- Check that your DAT file uses supported hash types (CRC32, MD5, SHA1)
- **Ensure ROM files are uncompressed** - ZIP, 7Z, RAR files are not supported
- Verify file integrity if ROMs are not being recognized

### Compressed ROM files
romaudit_cli does not support compressed files. Extract all ROMs from their archives before running the tool. Common compressed formats that need extraction:
- ZIP files
- 7Z files  
- RAR files
- GZ/GZIP files
- Any other archive format

### Permission errors
Ensure you have write permissions in the directory where romaudit_cli is running.

### Large collections
For very large collections, the initial scan may take time. Subsequent scans will be faster due to the persistent database.

## FAQ

### Does romaudit_cli support compressed ROM files?
**No.** romaudit_cli only works with uncompressed ROM files. You must extract all ROMs from ZIP, 7Z, RAR, or other archive formats before scanning. This is by design to ensure accurate hash verification and file organization.

### Why doesn't it support compressed files?
Working with uncompressed files ensures:
- Accurate hash verification
- Proper file organization
- Better performance
- Simpler codebase

### What file formats are supported?
Any uncompressed ROM file format (.nes, .snes, .md, .gb, .gba, .n64, .iso, .bin, etc.) that matches entries in your DAT file.