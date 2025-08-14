# romaudit_cli

[![Version](https://img.shields.io/badge/version-2.2.0-blue.svg)](https://github.com/yourusername/romaudit_cli)
[![License](https://img.shields.io/badge/license-Personal%20Use%20Only-red.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org/)

A powerful command-line ROM collection management tool written in Rust. romaudit_cli helps you organize, verify, and maintain your ROM collections using DAT files, with intelligent folder organization and comprehensive tracking.

**📋 License: Personal Use Only** - Free for personal use. Commercial use prohibited. See [LICENSE](LICENSE) for details.

**🚀 New in 2.2.0**: Streamlined for standard DAT files - focused on No-Intro and similar formats!

## Quick Start

```bash
# Ensure you have Rust with edition 2024 support
rustup update

# Clone and build
git clone https://github.com/yourusername/romaudit_cli.git
cd romaudit_cli
cargo build --release

# No configuration needed! Just run:
./target/release/romaudit_cli

# The tool will:
# - Look for a .dat file in current directory
# - Scan and organize ROMs
# - Create roms/ and logs/ directories
# - Save progress to rom_db.json
```

**Note**: No configuration file needed! The tool works with sensible defaults. See [Configuration](#configuration) for optional customization.

## Features

- **Standard DAT Support**: Optimized for No-Intro and similar DAT formats
- **Multi-threaded Performance**: Utilizes multiple CPU cores for fast file scanning
- **Automatic ROM Organization**: Intelligently organizes ROMs based on configurable rules
- **Smart Folder Management**: 
  - Multi-part games (disks, tracks) automatically placed in folders
  - Single ROMs with mismatched names get their own folders
  - Preserves internal folder structures from DAT files
- **Hash Verification**: Supports CRC32, MD5, and SHA1 verification
- **Duplicate Detection**: Identifies and manages duplicate ROMs
- **Unknown ROM Handling**: Separates unrecognized files for easy review
- **Shared ROM Tracking**: Identifies ROMs used by multiple games
- **Progress Tracking**: Visual progress bar during scanning
- **Graceful Shutdown**: Clean interruption handling with Ctrl+C
- **Detailed Logging**: Comprehensive logs for all operations
- **Persistent Database**: Maintains ROM database across scans
- **Fully Configurable**: No hardcoded values - everything is customizable
- **Modern Rust**: Uses Rust edition 2024 for latest language features
- **Optimized Performance**: Small binary size with aggressive optimizations

## What's New in 2.2.0

**Focused on Standard DAT Files**:
- Removed MAME-specific handling for cleaner, more maintainable code
- Optimized for No-Intro, Redump, and similar DAT formats
- Simplified organization logic
- Improved performance for standard collections

## Limitations

- **No Compressed File Support**: romaudit_cli works only with uncompressed ROM files. ZIP, 7Z, RAR, and other compressed formats are not supported. Extract your ROMs before scanning.

## Installation

### Prerequisites for Building

- Rust 1.75 or later (with edition 2024 support)
- Cargo (comes with Rust)

### Building from Source

```bash
git clone https://github.com/yourusername/romaudit_cli.git
cd romaudit_cli
cargo build --release
```

The compiled binary will be in `target/release/romaudit_cli` (or `romaudit_cli.exe` on Windows).

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

3. Run the program (no configuration needed):
   ```bash
   ./romaudit_cli
   ```

4. The program will:
   - Automatically detect and use the first `.dat` file found
   - Scan all uncompressed files in the current directory and subdirectories
   - Match them against the DAT file
   - Organize them according to the rules
   - Generate detailed logs
   - Handle interruptions gracefully (Ctrl+C saves progress)

**Note**: No `config.toml` required! The tool uses sensible defaults. See [Configuration](#configuration) if you want to customize settings.

### Directory Structure

After running, your directory will be organized as:

```
.
├── roms/                    # Organized ROM files
│   ├── Game Name/          # Multi-part games in folders
│   │   ├── disk1.bin
│   │   └── disk2.bin
│   ├── Single Game.rom     # Single ROMs directly in roms/
│   └── Special Game/       # Games with mismatched names
│       └── SPECIAL.BIN
├── logs/                   # Detailed audit logs
│   ├── have.txt           # List of found ROMs
│   ├── missing.txt        # List of missing ROMs
│   ├── shared.txt         # ROMs shared between games
│   └── folders.txt        # Games stored in subfolders
├── duplicates1/           # Duplicate files (if any)
├── unknown1/              # Unrecognized files (if any)
├── rom_db.json           # Persistent ROM database
└── your_file.dat         # Original DAT file
```

## Organization Rules

romaudit_cli follows these intelligent organization rules:

**Key Principle**: Folders are used ONLY when necessary - either for multiple files or to prevent naming conflicts.

1. **Multiple ROM Files** → Always use folders
   - Example: `roms/Game Name/disk1.bin`, `roms/Game Name/disk2.bin`

2. **Single ROM (matching name)** → Direct in roms/
   - Example: `roms/Sonic the Hedgehog.md`

3. **Single ROM (different name)** → Use folder
   - Example: `roms/Memory (Japan)/MEMORY.ASF`

4. **ROMs with paths** → Preserve folder structure
   - Example: `roms/Game/folder/file.bin`

## Configuration

romaudit_cli uses sensible defaults but is fully configurable. 

### Default Configuration (No File Needed)
The tool works out-of-the-box without any configuration file, using these defaults:
- ROM directory: `roms/`
- Logs directory: `logs/`
- Database file: `rom_db.json`
- Buffer size: 1MB
- Duplicate prefix: `duplicates`
- Unknown prefix: `unknown`

### Custom Configuration (Optional)
You can optionally create a `config.toml` file to override defaults:

```toml
# config.toml (optional)
rom_dir = "my_roms"
logs_dir = "audit_logs"
db_file = "my_database.json"
```

**Note**: The config file is completely optional. The tool runs perfectly with default settings.

## DAT File Support

romaudit_cli supports standard DAT file format (XML-based):

### Standard DAT Format (No-Intro Style)
```xml
<game name="Game Name">
    <rom name="game.rom" size="524288" crc="12345678" md5="..." sha1="..."/>
</game>
```

The tool automatically detects and parses DAT files with multiple hash types (CRC32, MD5, SHA1).

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

### Graceful Interruption

If you need to stop the tool:
- Press Ctrl+C for clean shutdown
- Progress is automatically saved to `rom_db.json`
- Run the tool again to continue from where you left off

## Performance

- **Multi-threaded Scanning**: Leverages multiple CPU cores for faster hashing
- **Efficient Hashing**: Uses 1MB buffer for optimal performance  
- **Single-pass scanning**: Calculates hashes only once per file
- **Progress Tracking**: Visual feedback with ETA for long operations
- **Direct File Access**: Working with uncompressed files provides faster processing
- **Persistent Database**: Speeds up subsequent scans
- **Graceful Interruption**: Clean shutdown preserves progress

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

## Troubleshooting

### No DAT file found
Ensure you have a `.dat` file in the current directory. The tool automatically detects and uses the first one it finds.

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
For very large collections, the initial scan may take time. The tool shows progress with ETA. Subsequent scans will be faster due to the persistent database.

### Process interruption
If you need to stop the tool, press Ctrl+C. The tool will save its progress and you can continue later by running it again.

## FAQ

### Does romaudit_cli require a configuration file?
**No.** The tool works perfectly with built-in defaults. You only need to create a `config.toml` if you want to customize settings like directory names or buffer sizes.

### Does romaudit_cli support compressed ROM files?
**No.** romaudit_cli only works with uncompressed ROM files. You must extract all ROMs from ZIP, 7Z, RAR, or other archive formats before scanning. This is by design to ensure accurate hash verification and file organization.

### What DAT formats are supported?
Standard XML-based DAT files, commonly used by No-Intro, Redump, and similar preservation projects.

### Why doesn't it support compressed files?
Working with uncompressed files ensures:
- Accurate hash verification
- Proper file organization
- Better performance
- Simpler codebase

### What file formats are supported?
Any uncompressed ROM file format (.nes, .snes, .md, .gb, .gba, .n64, .iso, .bin, etc.) that matches entries in your DAT file.