# callgrind-compare

A modern tool to compare callgrind_annotate outputs and track performance changes over time.

`callgrind-compare` allows for precise analysis of instruction count differences between different versions of your programs by comparing the output of valgrind's `callgrind_annotate` tool.

**Note:** Instruction counting is one metric among many for performance analysis. While not an absolute measure of performance, it can be valuable for understanding algorithmic changes and their impact on program execution.

## Installation

```bash
cargo install callgrind-compare
```

## Quick Start

```bash
# 1. Run your program through callgrind
valgrind --tool=callgrind --dump-instr=yes --collect-jumps=yes ./your_program

# 2. Generate annotate file
callgrind_annotate --auto=no --threshold=99.99 callgrind.out.12345 > baseline.cg

# 3. Make changes, recompile, and repeat steps 1-2 to create optimized.cg

# 4. Compare the results
callgrind-compare baseline.cg optimized.cg
```

## Features

- **Mixed Input Support**: Handles both `callgrind_annotate` files and CSV files in any combination
- **Smart Terminal Detection**: Automatically detects terminal capabilities and enables colors when appropriate  
- **Advanced CSV Export**: Export results with percentages, differences, or comprehensive data
- **Custom Column Naming**: Name your runs/columns with `--csv-names` for better organization
- **Flexible Sorting**: Sort by any column including symbol names
- **Multiple Display Modes**: Show instruction count, differences, percentages, or combinations
- **Symbol Name Processing**: String replacement capabilities for cleaner symbol names
- **Reference Column Selection**: Any column can be the reference for comparisons

## Screenshot

![Sample Output](./documents/SampleScreenshot.png)

This screenshot shows comparison of the same program across 8 different optimization stages. The first column serves as the reference, with subsequent columns showing both absolute instruction counts and percentage changes.

## How to Use

### Basic Comparison

Compare two callgrind annotate outputs:
```bash
callgrind-compare baseline.cg optimized.cg
```

### Multiple File Comparison

Compare multiple versions:
```bash
callgrind-compare v1.cg v2.cg v3.cg v4.cg
```

### CSV Export

Export results for further analysis:
```bash
callgrind-compare baseline.cg optimized.cg --csv-export results.csv
```

### Advanced CSV Export

```bash
# Export with percentages and differences
callgrind-compare baseline.cg v1.cg v2.cg \
  --csv-export results.csv \
  --csv-all-data \
  --csv-names "Baseline,Version1,Version2"
```

### Mixed Input Types

You can mix callgrind files and CSV files:
```bash
callgrind-compare baseline.cg intermediate.csv final.cg
```

## Generating Callgrind Data

### Step 1: Compile with Debug Info
```bash
# C/C++
gcc -O2 -g -o my_program my_program.c

# Rust
export CARGO_PROFILE_RELEASE_DEBUG=true
cargo build --release
```

### Step 2: Run Through Callgrind
```bash
valgrind --tool=callgrind \
  --dump-instr=yes \
  --collect-jumps=yes \
  --separate-threads=yes \
  ./your_program [arguments]
```

**Flag explanations:**
- `--dump-instr=yes`: Collect instruction-level information
- `--collect-jumps=yes`: Collect jump/branch information  
- `--separate-threads=yes`: Handle multi-threaded programs properly

### Step 3: Generate Annotate File
```bash
# Find the callgrind output
callgrind_file=$(ls -t callgrind.out.* | head -n1)

# Generate annotate file  
callgrind_annotate --auto=no --threshold=99.99 "$callgrind_file" > my_run.cg
```

**Threshold recommendations:**
- `--threshold=99.99`: Very comprehensive (includes functions down to 0.01%)
- `--threshold=95.0`: Balanced view (functions down to 5%)
- `--threshold=90.0`: High-level overview

## Command Line Options

### Display Options

- `-a, --all`: Show all symbols, even those without changes
- `--show [OPTIONS]`: Control what information to display
  - `ircount`: Show instruction counts
  - `percentagediff`: Show percentage changes
  - `ircountdiff`: Show raw differences
  - `all`: Show all three (default)

### Sorting Options

- `--sort-by <CRITERION>`: Control result sorting
  - `symbol`: Alphabetical by symbol name (default)
  - `first-ir`: By first column instruction count
  - `last-ir`: By last column instruction count
  - `columnX`: By column X instruction count (0-indexed)
  - Prefix with `-` for descending order

### Reference Column Options

- `--relative-to <REFERENCE>`: Choose reference for comparisons
  - `first`: Use first column as reference (default)
  - `last`: Use last column as reference
  - `previous`: Each column compares to the previous one
  - `columnX`: Use column X as reference (0-indexed)

### CSV Export Options

- `--csv-export <PATH>`: Export results to CSV
- `--csv-percentages`: Include percentage columns
- `--csv-differences`: Include difference columns  
- `--csv-all-data`: Include both percentages and differences
- `--csv-names [NAMES]`: Custom column names

### Output Control

- `-c, --color <MODE>`: Control colored output
  - `default`: Auto-detect terminal (default)
  - `always`: Force colors
  - `never`: Disable colors

### Symbol Processing

- `--string-replace [REPLACEMENTS]`: Replace strings in symbol names
  - Format: `old/new` (e.g., `__ZN/simplified`)

## Examples

### Basic Performance Tracking
```bash
# Compile and profile baseline
gcc -O2 -g -o program program.c
valgrind --tool=callgrind ./program input.txt
callgrind_annotate --threshold=99.99 callgrind.out.* > baseline.cg

# Make optimizations and profile again  
# ... make changes ...
gcc -O2 -g -o program program.c
valgrind --tool=callgrind ./program input.txt
callgrind_annotate --threshold=99.99 callgrind.out.* > optimized.cg

# Compare results
callgrind-compare baseline.cg optimized.cg
```

### Multi-Version Analysis
```bash
# Compare across git branches
for branch in v1.0 v1.1 v1.2; do
    git checkout $branch
    cargo build --release
    valgrind --tool=callgrind ./target/release/program
    callgrind_annotate --threshold=99.99 callgrind.out.* > ${branch}.cg
done

callgrind-compare v1.0.cg v1.1.cg v1.2.cg \
  --csv-export evolution.csv \
  --csv-all-data \
  --csv-names "v1.0,v1.1,v1.2"
```

### Advanced Sorting and Filtering
```bash
# Show only functions that changed, sorted by impact
callgrind-compare baseline.cg optimized.cg \
  --sort-by=-last-ir \
  --show percentagediff,ircountdiff
```

## Building from Source

```bash
git clone https://github.com/levu12/callgrind-compare
cd callgrind-compare
cargo build --release
```

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

This project builds upon ideas from performance analysis tools in the valgrind ecosystem, providing enhanced functionality for modern development workflows.
