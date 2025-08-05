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
- Include the version of romaudit_cli you're using
- Provide steps to reproduce the issue
- Include relevant DAT file format examples if applicable
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

## Testing

Currently, the project lacks automated tests. When adding new features:
- Consider adding unit tests
- Test with various DAT file formats
- Test edge cases (empty files, malformed data, etc.)
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

# Before committing
cargo fmt
cargo clippy
```

## Questions?

Feel free to open an issue for any questions about contributing!