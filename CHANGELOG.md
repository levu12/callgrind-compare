# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-07-25

### Added
- **Initial release** - A tool to compare callgrind_annotate outputs and track performance changes over time
- **Terminal Color Detection**: Smart terminal detection with `--color default` mode for automatic color handling
- **CSV Input Support**: Read existing CSV files as input alongside callgrind files
- **Enhanced CSV Export Options**:
  - `--csv-percentages`: Export with percentage calculations 
  - `--csv-differences`: Export with raw difference calculations
  - `--csv-all-data`: Export comprehensive data with both percentages and differences
- **Custom Column Naming**: Use `--csv-names` to provide meaningful names for your data columns
- **Mixed File Type Processing**: Seamlessly process combinations of callgrind and CSV files in any order
- **Content-Based File Detection**: Smart file type detection by analyzing content, not relying on file extensions
- **Flexible Sorting**: Sort by symbol name, instruction counts, or any column
- **Reference Column Selection**: Choose which column to use as reference for comparisons
- **Symbol Name Processing**: String replacement capabilities for cleaner symbol names
- **Comprehensive CLI**: Full command-line interface with help and version information

### Technical Details
- Uses modern `is-terminal` crate for terminal detection (replacing deprecated `atty`)
- Built with Rust 2021 edition
- Comprehensive test suite including integration tests
- Well-documented codebase with extensive README and examples
