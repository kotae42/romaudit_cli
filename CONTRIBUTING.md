# Contributing to romaudit_cli

Thank you for your interest in contributing to romaudit_cli! This document provides guidelines for contributing to the project.

## License Agreement

By contributing to romaudit_cli, you agree that your contributions will be licensed under the same Personal Use Only license as the project. This means:
- Your contributions can only be used for personal, non-commercial purposes
- You retain copyright of your contributions
- You grant permission for your code to be distributed under the project's license

## How to Contribute

### Reporting Issues

- Check if the issue already exists in the [Issues](https://github.com/yourusername/romaudit_cli/issues) section
- Include the version of romaudit_cli you're using (current: v1.6.2)
- Provide steps to reproduce the issue
- Include relevant DAT/XML file format examples if applicable
- Mention your operating system and Rust version

### Suggesting Features

- Open an issue with the "enhancement" label
- Describe the feature and its use case
- Explain how it would benefit users
- Consider how it fits with existing functionality

### Code Contributions

1. Ensure you have Rust 1.75+ with edition 2024 support
2. Fork the repository
3. Create a feature branch (`git checkout -b feature/amazing-feature`)
4. Make your changes
5. Run `cargo fmt` to format code
6. Run `cargo clippy` and fix warnings
7. Test your changes thoroughly
8. Commit with descriptive messages
9. Push to your fork
10. Open a Pull Request

### Pull Request Guidelines

- Keep changes focused and atomic
- Update documentation if needed
- Add entries to CHANGELOG.md under "Unreleased"
- Ensure all tests pass (when available)
- Be responsive to review feedback

## Code Style

- Follow Rust standard formatting (use `cargo fmt`)
- Use meaningful variable and function names
- Add comments for complex logic
- Keep functions focused and small
- Prefer clarity over cleverness
- Use proper error handling with the custom `RomAuditError` type

## Testing

Currently, the project lacks automated tests. When adding new features:
- Consider adding unit tests
- Test with various DAT/XML file formats (both .dat and .xml files)
- Test with large MAME XML files (45MB+)
- Test edge cases (empty files, malformed data, etc.)
- Test signal handling (Ctrl+C interruption)
- Test on different operating systems if possible
- Remember: romaudit_cli only supports uncompressed files

## Design Decisions

### No Compressed File Support
romaudit_cli is designed to work only with uncompressed ROM files. This is intentional:
- Ensures accurate hash verification
- Simplifies the codebase
- Improves performance
- Avoids dependencies on compression libraries

If you're considering adding compression support, please open an issue for discussion first.

### Signal Handling (Added in v1.6.2)
The application uses `ctrlc` crate for graceful shutdown:
- Clean interruption with Ctrl+C
- Progress saved to `rom_db.json`
- Can resume from where it left off

## Development Setup

```bash
# Clone your fork
git clone https://github.com/yourusername/romaudit_cli.git
cd romaudit_cli

# Add upstream remote
git remote add upstream https://github.com/originalowner/romaudit_cli.git

# Create a branch
git checkout -b feature/your-feature

# Make changes and test
cargo run

# Test with a large MAME XML file
# Place a .xml file in the directory and run

# Test interruption handling
# Start the tool and press Ctrl+C

# Before committing
cargo fmt
cargo clippy
```

## Current Architecture (v1.6.2)

### Key Components
- **DAT/XML Parser**: Supports both .dat and .xml formats
- **File Scanner**: Recursively scans directories
- **Hash Calculator**: CRC32, MD5, SHA1 support
- **Organization Engine**: Smart folder logic
- **Signal Handler**: Graceful shutdown with ctrlc
- **Progress Tracker**: Visual feedback with indicatif

### Dependencies
- `sha1`, `md-5`, `crc32fast`: Hashing
- `quick-xml`: DAT/XML parsing
- `serde`, `serde_json`: Database persistence
- `indicatif`: Progress bars
- `ctrlc`: Signal handling (v3.4.7)

## Questions?

Feel free to open an issue for any questions about contributing!