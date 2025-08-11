# Building romaudit_cli Version 2.0.0

## Overview

Version 2.0.0 is a complete modular rewrite that maintains 100% compatibility with v1.x while providing a much better code structure.

### Build Instructions

```bash
# Clean build directory (optional)
cargo clean

# Build the modular version
cargo build --release

# The binary will be at:
# target/release/romaudit_cli (Linux/Mac)
# target/release/romaudit_cli.exe (Windows)

# Run without any configuration (uses defaults)
./target/release/romaudit_cli
```

### Configuration

The tool works perfectly without any configuration file. If you want to customize settings:

1. **Option 1**: Use defaults (recommended)
   - No configuration needed
   - Tool creates `roms/` and `logs/` directories
   - Database saved as `rom_db.json`

2. **Option 2**: Custom configuration (optional)
   - Uncomment `toml = "0.9.5"` in Cargo.toml
   - Rebuild with `cargo build --release`
   - Create `config.toml` with your settings

### File Structure

Create the following file structure in your `src/` directory:

```
src/
├── main.rs
├── config.rs
├── error.rs
├── types.rs
├── database/
│   └── mod.rs
├── logger/
│   └── mod.rs
├── organizer/
│   ├── mod.rs
│   ├── folders.rs
│   ├── processor.rs
│   └── rules.rs
├── parser/
│   ├── mod.rs
│   ├── detector.rs
│   └── xml.rs
└── scanner/
    ├── mod.rs
    ├── collector.rs
    └── hasher.rs
```

### Creating the Files

1. Copy each module code from the artifacts above into the corresponding file
2. Ensure all files are saved with UTF-8 encoding
3. Build the project

### Advantages of Modular Version

- **No functional changes** - Works exactly like v1.6.4
- **Easier debugging** - Issues isolated to specific modules
- **Better maintainability** - Each module has a single purpose
- **Simpler testing** - Can test individual components
- **Easier contributions** - New features don't affect existing code

### Verifying the Build

After building, test with:

```bash
# Place a DAT/XML file in the directory
cp /path/to/your/dat/file.xml .

# Run the tool
./target/release/romaudit_cli

# Should see:
# Found DAT/XML file: ./file.xml
# DAT type: [type detection]
# Scanning files...
```

### Troubleshooting

If you get compilation errors:

1. **Missing module**: Ensure all files are created in the correct directories
2. **Import errors**: Check that module names match the file structure
3. **Type errors**: Ensure you're using Rust edition 2024

### Migrating from Monolithic

No migration needed! The modular version:
- Reads the same `rom_db.json`
- Produces the same output
- Uses the same configuration
- Has identical behavior

Just replace the binary and continue using as before.

### Next Steps

See [MODULE-GUIDE.md](MODULE-GUIDE.md) for:
- Module documentation
- Adding new features
- Testing guidelines
- Architecture details