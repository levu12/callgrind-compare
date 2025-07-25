# Changelog

All notable changes to callgrind-compare will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-07-25

### Added
- Initial release of callgrind-compare as an independent project
- Mixed input support for callgrind_annotate files and CSV files
- Smart terminal color detection with configurable color modes
- Advanced CSV export with percentages, differences, and comprehensive data options
- Custom column naming for better organization
- Flexible sorting by any column or symbol name
- Multiple display modes (instruction counts, differences, percentages)
- Symbol name string replacement for cleaner output
- Reference column selection with support for relative comparisons
- Comprehensive command-line interface with extensive options
- Real callgrind data processing with proper validation
- Professional documentation and examples

### Technical Details
- Built with Rust 2021 edition
- Uses clap for command-line parsing
- CSV processing with serde
- Terminal detection with is-terminal crate
- Comprehensive error handling with anyhow
