# callgrind_differ

`callgrind_differ` is a tool that compares the output of different
[`callgrind_annotate`](https://valgrind.org/docs/manual/cl-manual.html#cl-manual.callgrind_annotate-options) outputs.
This allows for precise and visual indications on the instruction count of an executable at different times during its
development.

**Disclaimer:** Instruction counting is **not** an absolute metric for performance. It is only one amongst many that can
be used to speed up programs. There are many more factors to take into account. There are however use-cases where
instruction count may relate closely to wall-time performance and my use-case was one of these.

## 🚨 IMPORTANT: Using Real Callgrind Data

**This tool is designed to work exclusively with real `callgrind_annotate` output files.** You must:

1. **Run your program through valgrind's callgrind tool**
2. **Process the callgrind.out file with `callgrind_annotate`** 
3. **Feed the annotate output to callgrind_differ**

**Do not attempt to generate synthetic or fake callgrind data** - the tool expects the specific format produced by the official `callgrind_annotate` command.

## Quick Start

```bash
# 1. Run your program through callgrind
valgrind --tool=callgrind --dump-instr=yes --collect-jumps=yes ./your_program

# 2. Generate annotate file
callgrind_annotate --auto=no --threshold=99.99 callgrind.out.12345 > baseline.cg

# 3. Make changes, recompile, and repeat steps 1-2 to create optimized.cg

# 4. Compare the results
callgrind_differ baseline.cg optimized.cg
```

# Screenshot
![](./documents/SampleScreenshot.png)

This screenshot features 8 times the same program run, at different stages of manual optimization (compiler flags were
the same). The first data column is the oldest run and the last one the most recent. The overall decrease in instruction
count from the beginning to the end was 31.531%.

The first column is the symbol name, and following columns are metrics taken on a run of the program. In this instance,
the first column of data is taken as a reference (hence why it has no +/- percentage). All other columns are compared to
it. The instruction count is repeated for all other columns.

# Features
  * **Mixed Input Support**: Can take as input any number of `callgrind_annotate` outputs AND CSV files in any combination
  * **Smart Terminal Color Detection**: Automatically detects terminal capability and enables colors appropriately
  * **Advanced CSV Export**: Export results to CSV with percentage calculations, differences, and comprehensive data options
  * **Custom Column Naming**: Name your runs/columns for better organization and clarity
  * **Flexible File Detection**: Content-based file type detection (not extension-dependent)
  * **Colored Terminal Output**: Green for decreased instruction count, red for increases (when terminal supports it)
  * **Flexible Sorting**: Sort by any column including symbol names
  * **Smart Filtering**: Auto-removes symbols whose instruction count never changes (disable with `--all`)
  * **Reference Column Selection**: Any column can be the reference for comparisons, including adjacent column comparisons
  * **Multiple Display Modes**: Show instruction count, differences, percentages, or ratios (>1000%)
  * **Symbol Name Processing**: String replacement capabilities for cleaner symbol names

# How to use

## Mixed Input Support
`callgrind_differ` can seamlessly handle:
- Pure `callgrind_annotate` files
- Pure CSV files  
- Mixed combinations of both file types

The tool automatically detects file types by content (not extension), so you can use any combination:
```sh
# Pure callgrind files
callgrind_differ run1.cg run2.cg run3.cg

# Pure CSV files
callgrind_differ baseline.csv optimized.csv

# Mixed file types
callgrind_differ baseline.cg optimized.csv final_run.cg
```

## How to Generate Callgrind Annotate Files 

### Step-by-Step Process

#### 1. Compile Your Program with Debug Info
**C/C++**: Use `-O2 -g` flags  
**Rust**: Set `CARGO_PROFILE_RELEASE_DEBUG=true` environment variable

```bash
# C/C++ example
gcc -O2 -g -o my_program my_program.c

# Rust example  
export CARGO_PROFILE_RELEASE_DEBUG=true
cargo build --release
```

#### 2. Run Through Callgrind
Use valgrind's callgrind tool to profile your program:

```bash
valgrind --tool=callgrind \
  --dump-instr=yes \
  --collect-jumps=yes \
  --separate-threads=yes \
  ./your_program [program_arguments]
```

**Flag Explanations**:
- `--dump-instr=yes`: Collect instruction-level information
- `--collect-jumps=yes`: Collect jump/branch information  
- `--separate-threads=yes`: Handle multi-threaded programs properly

This creates a file named `callgrind.out.XXXXX` where XXXXX is the process ID.

#### 3. Generate Annotate File
Process the callgrind output with `callgrind_annotate`:

```bash
# Find the most recent callgrind output file
callgrind_file=$(ls -t callgrind.out.* | head -n1)

# Generate annotate file
callgrind_annotate --auto=no --threshold=99.99 "$callgrind_file" > my_run.cg
```

**Key `callgrind_annotate` Options**:
- `--auto=no`: Disable automatic source annotation (cleaner output)
- `--threshold=99.99`: Include functions down to 0.01% of total (very comprehensive)
- `--threshold=95.0`: Include functions down to 5% of total (less verbose)
- `--threshold=0.1`: Include almost all functions (very detailed)

#### 4. Use with callgrind_differ
```bash
callgrind_differ my_run.cg
```

### Threshold Selection Guide

The `--threshold` parameter in `callgrind_annotate` determines which functions appear in the output:

```bash
# Very comprehensive (recommended for detailed analysis)
callgrind_annotate --threshold=99.99 callgrind.out.12345 > detailed.cg

# Balanced view (good for general use)
callgrind_annotate --threshold=99.0 callgrind.out.12345 > balanced.cg

# High-level overview (major functions only)  
callgrind_annotate --threshold=95.0 callgrind.out.12345 > overview.cg
```

### Complete Example Workflow

```bash
#!/bin/bash
# Complete workflow for profiling with callgrind_differ

# 1. Build with debug info
export CARGO_PROFILE_RELEASE_DEBUG=true
cargo build --release

# 2. Create baseline profile
valgrind --tool=callgrind \
  --dump-instr=yes \
  --collect-jumps=yes \
  --callgrind-out-file=callgrind.out.baseline \
  ./target/release/my_program input.txt

# 3. Generate baseline annotate file
callgrind_annotate --auto=no --threshold=99.99 \
  callgrind.out.baseline > baseline.cg

# 4. Make optimizations to code and rebuild
# ... your code changes ...
cargo build --release

# 5. Create optimized profile  
valgrind --tool=callgrind \
  --dump-instr=yes \
  --collect-jumps=yes \
  --callgrind-out-file=callgrind.out.optimized \
  ./target/release/my_program input.txt

# 6. Generate optimized annotate file
callgrind_annotate --auto=no --threshold=99.99 \
  callgrind.out.optimized > optimized.cg

# 7. Compare the results
callgrind_differ baseline.cg optimized.cg \
  --csv-export results.csv \
  --csv-all-data \
  --csv-names "Baseline,Optimized"

# 8. Clean up raw callgrind files (optional)
rm callgrind.out.baseline callgrind.out.optimized
```

### Understanding the Output Format

`callgrind_annotate` produces output that looks like:

```
Profile data file 'callgrind.out.12345' (creator: callgrind-3.18.1)

...header information...

Ir

14,418,621,168 (100.0%)  PROGRAM TOTALS

Ir
11,790,287,101 (81.78%)  main [my_program.c:45]
 1,126,749,000 ( 7.81%)  bubble_sort [my_program.c:12]
   125,234,567 ( 0.87%)  malloc
   ...more functions...
```

**Key Parts**:
- **Header**: Shows source callgrind file and version
- **PROGRAM TOTALS**: Total instruction count for the entire program
- **Function Entries**: Each line shows instruction count, percentage, and function name
- **Source Location**: File and line number (when available)

This is exactly the format that `callgrind_differ` expects as input.

### Basic CSV Export
Export your results to CSV format for further analysis:
```sh
callgrind_differ run1.cg run2.cg --csv-export results.csv
```

### Advanced CSV Export Options

#### Export with Percentages Only
Include percentage changes without raw differences:
```sh
callgrind_differ run1.cg run2.cg --csv-export results.csv --csv-percentages
```
This creates columns like: `name,baseline_ir,optimized_ir,optimized_pct`

#### Export with Differences Only  
Include raw differences without percentages:
```sh
callgrind_differ run1.cg run2.cg --csv-export results.csv --csv-differences
```
This creates columns like: `name,baseline_ir,optimized_ir,optimized_diff`

#### Export All Data
Include both differences and percentages:
```sh
callgrind_differ run1.cg run2.cg --csv-export results.csv --csv-all-data
```
This creates comprehensive columns like: `name,baseline_ir,optimized_ir,optimized_diff,optimized_pct`

#### Custom Column Names
Name your columns for better organization:
```sh
callgrind_differ baseline.cg v1.cg v2.cg --csv-export results.csv --csv-names "Baseline,Version1,Version2"
```

### CSV Format Details
- **Differences**: Raw numerical changes (e.g., `-111111` for decrease of 111,111 instructions)
- **Percentages**: Percentage changes with 3 decimal precision (e.g., `-9.000` for 9% decrease)
- **Column Naming**: First column is always used as reference for percentage/difference calculations
- **Content Detection**: Files are detected by content analysis, not file extensions
### Example scenario: Trying to optimize
#### Creating a first callgrind file
Find your favourite benchmark code and compile with optimizations but *leave debug info enabled*. For C/C++, this should
be something along the lines of `-O2 -g`, and for Rust setting the `CARGO_PROFILE_RELEASE_DEBUG` environment variable to
`true`.

Run `valgrind` with the `callgrind` tool as you would normally. I personally use some flags which may or may not help:
```sh
valgrind --tool=callgrind --dump-instr=yes --collect-jumps=yes ./your_executable <arguments to your executable...>
```

This will generate a `callgrind.out.<pid>` file. Run `callgrind_annotate` on that file and redirect the output to a file:
```sh
callgrind_annotate --auto=no --threshold=99.99 callgrind.out.<pid> > run_1.callgrind
```
You may freely discard the `callgrind.out.<pid>`. `callgrind_differ` does not use it. The options you pass to
`callgrind_annotate` are yours to choose. You may specify `--auto=yes` (the default) but `callgrind_differ` will not use
the extra text it generates.

#### Creating a second callgrind file and diffing
Make changes into your code, re-compile and re-run both `valgrind` and `callgrind_annotate` (make sure to run it on the
*new* callgrind file) and redirect its output to another file (say `run_2.callgrind`).

You may then use `callgrind_differ` to compare whether your changes improved the instruction count or not:
```sh
callgrind_differ run_1.callgrind run_2.callgrind
# Or, if you use bash expansions
callgrind_differ run_{1,2}.callgrind
```

### Example scenario: Measuring improvements from the past
Say you have a benchmark command and you can run it through multiple commits. You may use something akin to the
following to generate `callgrind_annotate` files throughout your git history.

The following example uses Rust, but you may freely replace your compile command and execution:
```sh
set euo -pipefail

# `$x` is used as a prefix for `callgrind_annotate` output files.
# This allows for files to show in-order when listing directories
x=0

export CARGO_PROFILE_RELEASE_DEBUG=true

for commit in f7bed8e master develop rc-v1.3.0; # You may use git hashes, tags, branches, ...
                                                # Anything you can checkout to will do
do
  # === Checkout to the given commit
  git checkout ${commit}

  # === Build and run through callgrind
  cargo build --release --bin benchmarks # or `gcc -O2 -g`
  valgrind --tool=callgrind --dump-instr=yes --collect-jumps=yes ./target/release/benchmarks input_file

  # === Annotate the output file
  # The line below creates a variable named `$cg_file` with the last modified file matching `callgrind.out.*`
  cg_file=`\ls -1t callgrind.out.* | head -n1`
  callgrind_annotate --auto=no --threshold=99.99 $cg_file > ${x}-${commit}.cg

  # === Cleanup
  rm ${cg_file}; # Don't remove `${x}-${commit}.cg` ;)

  ((x++))
done
```

This will create files `0-f7bed8e.cg`, `1-master.cg`, `2-develop.cg` and `3-rc-v1.3.0.cg`.

You can compare them all (assuming `f7bed8e` is older than `master` which is older than `develop` which is older than
`rc-v1.3.0`) using:
```sh
callgrind_differ 0-f7bed8e.cg 1-master.cg 2-develop.cg 3-rc-v1.3.0.cg
# If you have no other `.cg` file in your current directory, you can do
callgrind_differ {0,1,2,3}-*.cg
# which is pretty handy.
```

# Options

## Complete Flag Reference

### Input and Output Options

#### `[INPUTS]...` (Positional Arguments)
**Purpose**: Specify the files to compare  
**Accepts**: `callgrind_annotate` output files (`.cg` files) or CSV files
**Mixed Support**: You can mix both file types in any order
**Detection**: Files are detected by content, not extension

```bash
# Pure callgrind files
callgrind_differ run1.cg run2.cg run3.cg

# Pure CSV files  
callgrind_differ data.csv optimized.csv

# Mixed file types
callgrind_differ baseline.cg optimization.csv final.cg
```

#### `--csv-export <PATH>`
**Purpose**: Export results to CSV format for analysis  
**Format**: Creates structured CSV with symbol names and instruction counts  
**Enhances**: Works with `--csv-percentages`, `--csv-differences`, `--csv-all-data`

```bash
callgrind_differ run1.cg run2.cg --csv-export results.csv
```

### CSV Export Control Flags

#### `--csv-percentages`
**Purpose**: Include percentage change columns in CSV export  
**Format**: Adds `*_pct` columns showing percentage changes  
**Calculation**: Relative to reference column (see `--relative-to`)

```bash
# Creates: name,baseline_ir,optimized_ir,optimized_pct
callgrind_differ baseline.cg optimized.cg --csv-export results.csv --csv-percentages
```

#### `--csv-differences` 
**Purpose**: Include raw difference columns in CSV export  
**Format**: Adds `*_diff` columns showing instruction count changes  
**Values**: Positive for increases, negative for decreases

```bash
# Creates: name,baseline_ir,optimized_ir,optimized_diff  
callgrind_differ baseline.cg optimized.cg --csv-export results.csv --csv-differences
```

#### `--csv-all-data`
**Purpose**: Include both percentages and differences in CSV export  
**Format**: Combines both `--csv-percentages` and `--csv-differences`  
**Most Comprehensive**: Provides complete data set for analysis

```bash
# Creates: name,baseline_ir,optimized_ir,optimized_diff,optimized_pct
callgrind_differ baseline.cg optimized.cg --csv-export results.csv --csv-all-data
```

#### `--csv-names [NAMES...]`
**Purpose**: Provide custom column names for better organization  
**Requirement**: Must match the number of callgrind files (not CSV files)  
**Format**: Comma-separated list

```bash
callgrind_differ baseline.cg v1.cg v2.cg --csv-names "Baseline,Version_1,Version_2"
```

### Display and Filtering Options

#### `-a, --all`
**Purpose**: Show all symbols, even those without changes  
**Default**: Only symbols with changes are shown  
**Use Case**: When you need to see the complete symbol table

```bash
callgrind_differ run1.cg run2.cg --all
```

#### `--show [OPTIONS...]`
**Purpose**: Control what information to display for each column  
**Options**:
- `ircount`: Show instruction counts
- `percentagediff`: Show percentage changes  
- `ircountdiff`: Show raw instruction count differences
- `all`: Show all three (default)

```bash
# Show only percentages
callgrind_differ run1.cg run2.cg --show percentagediff

# Show counts and differences but not percentages
callgrind_differ run1.cg run2.cg --show ircount,ircountdiff
```

### Sorting Options

#### `--sort-by <CRITERION>`
**Purpose**: Control how results are sorted  
**Options**:
- `symbol`: Alphabetical by symbol name (default)
- `first-ir`: By instruction count of first column
- `last-ir`: By instruction count of last column  
- `columnX`: By instruction count of column X (0-indexed)

**Direction Modifiers**:
- Prefix with `-` for descending order
- Prefix with `+` for ascending order (default)

```bash
# Sort by symbol name (default)
callgrind_differ run1.cg run2.cg --sort-by symbol

# Sort by first column, descending
callgrind_differ run1.cg run2.cg --sort-by=-first-ir

# Sort by third column (index 2), ascending  
callgrind_differ run1.cg run2.cg run3.cg --sort-by column2
```

### Reference Column Options

#### `--relative-to <REFERENCE>`
**Purpose**: Choose which column serves as the reference for comparisons  
**Options**:
- `first`: Use first column as reference (default)
- `last`: Use last column as reference
- `previous`: Each column compares to the previous one
- `columnX`: Use column X as reference (0-indexed)

```bash
# Compare everything to the last column
callgrind_differ old.cg middle.cg new.cg --relative-to last

# Compare each column to the previous one
callgrind_differ v1.cg v2.cg v3.cg --relative-to previous

# Use second column (index 1) as reference
callgrind_differ baseline.cg target.cg optimized.cg --relative-to column1
```

### Text Processing Options

#### `--string-replace [PATTERNS...]`
**Purpose**: Clean up symbol names by replacing patterns  
**Format**: `old_text/new_text` (can be repeated)  
**Use Case**: Simplify complex symbol names for better readability

```bash
# Replace std:: with STD::
callgrind_differ run1.cg run2.cg --string-replace "std::/STD::"

# Multiple replacements
callgrind_differ run1.cg run2.cg \
  --string-replace "std::/STD::" \
  --string-replace "__/private_" \
  --string-replace "malloc/MALLOC"
```

### Output Control Options

#### `-c, --color <MODE>`
**Purpose**: Control colored output  
**Modes**:
- `default`: Auto-detect terminal capabilities (default)
- `always`: Force colors even when piping to files
- `never`: Disable all colors

```bash
# Force colors even when redirecting to file
callgrind_differ run1.cg run2.cg --color always > results.txt

# Disable colors completely
callgrind_differ run1.cg run2.cg --color never
```

### Unsupported/Future Options

#### `--export-graph <PATH>`
**Status**: Currently unsupported  
**Purpose**: Will generate graphs of performance data  
**Note**: Placeholder for future functionality
```
$> callgrind_differ -h
A tool to help keep track of performance changes over time

Usage: callgrind_differ [OPTIONS] [INPUTS]...

Arguments:
  [INPUTS]...  `callgrind_annotate` files or CSV file. Positional arguments

Options:
  -a, --all
          Show all lines, even those without a change
  -c, --color <COLOR>
          Whether the output should be colored or not [default: default] 
          [possible values: always, never, default]
      --sort-by <SORT_BY>
          By which field to sort by [default: symbol]
      --csv-export <CSV_EXPORT>
          Path to an output file in which to write the IR as CSV
      --csv-percentages
          Include percentage columns in CSV export
      --csv-differences  
          Include difference columns in CSV export
      --csv-all-data
          Include both differences and percentages in CSV export
      --csv-names [<CSV_NAMES>...]
          A comma-separated list of column names for the CSV export
      --string-replace [<STRING_REPLACE>...]
          A replacement to perform in the symbol names
      --export-graph <EXPORT_GRAPH>
          Path to an output file in which to write a graph of the IR values. Currently unsupported
      --relative-to <RELATIVE_TO>
          The column which is the reference for IR. Other columns have diffs relative to it [default: first]
      --show [<SHOW>...]
          A comma-separated list of what to show for each column of data
  -h, --help
          Print help (see more with '--help')
```

## Color Output Control
The `--color` option now supports intelligent defaults:
- `--color default` (default): Automatically detects terminal capabilities
- `--color always`: Force color output even to files/pipes  
- `--color never`: Disable colors completely

## New CSV Export Flags
- `--csv-percentages`: Export with percentage calculations only
- `--csv-differences`: Export with raw difference calculations only  
- `--csv-all-data`: Export with both percentages and differences
- `--csv-names`: Provide custom names for your data columns

More details can be obtained with `callgrind_differ --help`.

# Recently Implemented Features ✅

The following features have been recently added to enhance the tool's capabilities:

- ✅ **Terminal Color Detection**: Automatically detects terminal capabilities and enables colors when appropriate
- ✅ **CSV Input Support**: Read existing CSV files as input alongside callgrind files  
- ✅ **Enhanced CSV Export**: Export with percentages, differences, or comprehensive data
- ✅ **Custom Column Naming**: Name your runs/columns with `--csv-names` for better organization
- ✅ **Mixed File Type Support**: Seamlessly process combinations of callgrind and CSV files
- ✅ **Content-Based File Detection**: Smart file type detection by content analysis, not extensions

# Future Enhancements

This list represents potential future improvements. You are welcome to implement these features and submit a pull request, or file an issue if you think the tool would benefit from additional functionality.

- 🔄 **CSV Color Formatting**: Investigate adding color codes to CSV export for enhanced readability
- 🔄 **Graph Export**: Complete the graph export functionality (currently marked as unsupported)
- 🔄 **Additional File Formats**: Support for JSON or other structured data formats
- 🔄 **Advanced Filtering**: More sophisticated symbol filtering and grouping options
- 🔄 **Performance Metrics**: Additional metrics beyond instruction count (cache misses, branch predictions, etc.)
- 🔄 **Interactive Mode**: Real-time file watching and updating of comparisons
