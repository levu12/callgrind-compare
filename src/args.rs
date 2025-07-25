use std::{borrow::Cow, fmt::Display, path::Path, str::FromStr};

use anyhow::{bail, Result};
use clap::Parser;
use is_terminal::IsTerminal;
use itertools::Itertools;

/// The field on which to sort the output by.
#[derive(Debug, Clone, Copy)]
pub enum SortByField {
    /// Sort by the name of the symbol in lexicographic order.
    Symbol,
    /// Sort by the instruction count of the first column.
    FirstIR,
    /// Sort by the instruction count of the last column.
    LastIR,
    /// Sort by the instruction count of the given column (0-indexed).
    ColumnIR(u32),
}

/// The order in which to sort (ascending / descending).
#[derive(Debug, Clone, Copy)]
pub enum SortByOrder {
    /// Ascending order. Lowest value at the top.
    Ascending,
    /// Descending order. Lowest value at the bottom.
    Descending,
}

/// How to sort the output. The default is by ascending symbol.
#[derive(Debug, Clone, Copy)]
pub struct SortBy {
    /// The field on which to sort the output.
    pub field: SortByField,
    /// The order on which to sort the output.
    pub order: SortByOrder,
}

impl Default for SortBy {
    fn default() -> Self {
        SortBy {
            field: SortByField::Symbol,
            order: SortByOrder::Ascending,
        }
    }
}

impl FromStr for SortBy {
    type Err = anyhow::Error;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        // Attempt to parse a leading '+' or '-' for order.
        let order = if s.is_empty() {
            bail!("Empty sort-by value")
        } else if s.as_bytes()[0] == b'+' {
            s = &s[1..];
            SortByOrder::Ascending
        } else if s.as_bytes()[0] == b'-' {
            s = &s[1..];
            SortByOrder::Descending
        } else {
            SortByOrder::Ascending
        };

        // Then check the field.
        let field = match s {
            "symbol" => SortByField::Symbol,
            "last-ir" => SortByField::LastIR,
            "first-ir" => SortByField::FirstIR,
            // We only accept things like "column3" or "column0".
            mut s if s.starts_with("column") => {
                s = &s["column".len()..];
                if s.is_empty() {
                    bail!("sort-by=column needs a 0-index, e.g.: --sort-by=column3 for 4th column");
                }
                if let Ok(x) = s.parse::<u32>() {
                    SortByField::ColumnIR(x)
                } else {
                    bail!("Invalid column number: {s}");
                }
            }
            _ => bail!("Invalid sort-by. Accepted values are: symbol, first-ir, last-ir, columnX"),
        };

        Ok(Self { field, order })
    }
}

impl Display for SortBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// How columns are compared one to another. The default is to compare to the first column.
#[derive(Default, Debug, Clone, Copy)]
pub enum RelativeTo {
    /// Every column is compared to the first column (default).
    #[default]
    First,
    /// Every column is compared to the last column.
    Last,
    /// Every column is compared to column preceding it.
    Previous,
    /// Every column is compared to the n-th column (0-indexed).
    Column(u32),
}

impl FromStr for RelativeTo {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "first" => Ok(Self::First),
            "last" => Ok(Self::Last),
            "previous" => Ok(Self::Previous),
            s if s.starts_with("column") => {
                let number: &str = &s["column".len()..];
                if let Ok(x) = number.parse::<u32>() {
                    Ok(Self::Column(x))
                } else {
                    bail!("Invalid column number: {number}");
                }
            }
            _ => bail!("Invalid relative-to. Accepted values are: first, last, previous, columnX"),
        }
    }
}

impl Display for RelativeTo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// What to show for each data column.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Show {
    /// All columns.
    #[default]
    All,
    /// The IR count.
    IRCount,
    /// The percentage (or ratio) of increase/decrease (with respect to [`RelativeTo`]).
    PercentageDiff,
    /// The difference in IR count with respect to [`RelativeTo`].
    IRCountDiff,
}

impl FromStr for Show {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "all" => Ok(Self::All),
            "ircount" => Ok(Self::IRCount),
            "percentagediff" => Ok(Self::PercentageDiff),
            "ircountdiff" => Ok(Self::IRCountDiff),
            _ => bail!(
                "Invalid show. Accepted values are: all, ircount, percentagediff, ircountdiff"
            ),
        }
    }
}

impl Display for Show {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Whether to color the output.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    /// Always color the output.
    Always,
    /// Only color if the output is a terminal.
    #[default]
    Default,
    /// Never color.
    Never,
}

impl Color {
    /// Determine if colors should be used based on the setting and terminal detection.
    pub fn should_color(self) -> bool {
        match self {
            Color::Always => true,
            Color::Default => std::io::stdout().is_terminal(),
            Color::Never => false,
        }
    }
}

impl FromStr for Color {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "always" => Ok(Self::Always),
            "default" => Ok(Self::Default),
            "never" => Ok(Self::Never),
            _ => bail!("Invalid color. Accepted values are: always, default, never"),
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// A string replacement to perform on a symbol name.
#[derive(Default, Debug, Clone)]
pub struct StringReplacement {
    /// The string to look for.
    from: String,
    /// What it will be replaced with.
    to: String,
}

impl StringReplacement {
    /// Perform the replacement on the given string.
    pub fn perform<'a>(&self, s: Cow<'a, str>) -> Cow<'a, str> {
        if s.find(&self.from).is_some() {
            Cow::Owned(s.replace(&self.from, &self.to))
        } else {
            s
        }
    }
}

impl FromStr for StringReplacement {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once('/') {
            Some((from, to)) => Ok(Self {
                from: from.to_string(),
                to: to.to_string(),
            }),
            None => bail!("No '/' in string replacement"),
        }
    }
}

impl Display for StringReplacement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// A tool to help keep track of performance changes over time.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[allow(clippy::struct_excessive_bools)]
pub struct Args {
    /// Show all lines, even those without a change.
    #[arg(short, long, default_value_t = false)]
    pub all: bool,
    /// Whether the output should be colored or not.
    ///
    /// Accepted values are:
    ///  * `always`: The output will always be colored
    ///  * `default`: The output is colored only if the output is a tty (default)
    ///  * `never`: The output is never colored
    #[arg(short, long, default_value = "default")]
    pub color: Color,
    /// By which field to sort by.
    ///
    /// Accepted values are:
    ///   * `symbol`: Sort lexicographically by the symbol name.
    ///   * `first-ir`: Sort by the instruction count of the first column.
    ///   * `last-ir`: Sort by the instruction count of the last column.
    ///   * `columnX`: With `X` a number, sort by the X-th column (0-indexed).
    ///
    /// Additionally, a `-` can be prepended to sort in descending order (a `+` can be prepended
    /// for ascending order, but that is already the default.
    ///
    /// ```no_compile
    /// symbol        // Sort by ascending symbol (default)
    /// +symbol       // Sort by ascending symbol
    /// -symbol       // Sort by descending symbol
    /// -first-ir     // Sort by descending ir for the first column
    /// column0       // Sort by ascending ir for the first column
    /// -column3      // Sort by descending ir for the 4th column
    /// ```
    #[arg(long, default_value = "symbol")]
    pub sort_by: SortBy,
    /// Path to an output file in which to write the IR as CSV.
    #[arg(long, default_value_t)]
    pub csv_export: String,
    /// Include percentage differences in CSV export.
    #[arg(long, default_value_t = false)]
    pub csv_percentages: bool,
    /// Include IR count differences in CSV export.
    #[arg(long, default_value_t = false)]
    pub csv_differences: bool,
    /// Include all data (IR counts, differences, and percentages) in CSV export.
    #[arg(long, default_value_t = false)]
    pub csv_all_data: bool,
    /// Column names for the CSV export. Use multiple times to specify multiple names.
    ///
    /// There must be as many names as there are `callgrind_annotate` files given as argument
    /// (i.e. this does not account for columns from CSV files, which may already have their own
    /// names.). Use --csv-names "Name1" --csv-names "Name2" for names with spaces or commas.
    #[arg(long, action = clap::ArgAction::Append)]
    pub csv_names: Vec<String>,
    /// A replacement to perform in the symbol names.
    ///
    /// The replacement has the form `foo/bar` and will replace any occurence of `foo` within the
    /// symbol name by `bar`. This option can be repeated any number of times.
    #[arg(long, num_args=0..)]
    pub string_replace: Vec<StringReplacement>,
    /// Path to an output file in which to write a graph of the IR values. Currently unsupported.
    #[arg(long, default_value_t)]
    pub export_graph: String,
    /// The column which is the reference for IR. Other columns have diffs relative to it.
    ///
    /// Accepted values are:
    ///   * `first`: Differences are shown relative to the first column (default).
    ///   * `last`: Differences are shown relative to the last column.
    ///   * `previous`: Differences are shown relative to the column preceding it.
    ///   * `columnX`: With `X` a number, relative to the X-th column (0-indexed).
    #[arg(long, default_value = "first")]
    pub relative_to: RelativeTo,
    /// A comma-separated list of what to show for each column of data.
    ///
    /// Accepted values are:
    ///   * `ircount`: The IR count.
    ///   * `percentagediff`: The percentage/ratio of ir count with respect to [`relative_to`].
    ///   * `ircountdiff`: The IR count difference with respect to [`relative_to`].
    ///   * `all`: `ircountdiff` + `percentagediff` + `ircount`
    ///
    /// Any value re-specified will be ignored. `all` has precedence. To show all columns in a
    /// different order than `all`, specify each column individually but not `all`.
    #[arg(long, num_args=0.., value_delimiter=',')]
    pub show: Vec<Show>,
    /// `callgrind_annotate` files or CSV file. Positional arguments.
    ///
    /// If the file name ends with `.csv` (case-insensitive), then the argument will be interpreted
    /// as a csv file where each row is a symbol, each column a run and each cell an IR count.
    /// The first row will be interepreted as a header if and only if the first cell contains
    /// `"name"` and the second cell cannot be parsed as an integer.
    ///
    /// Otherwise, interpret the file as an output from `callgrind_annotate`.
    ///
    /// Columns are loaded in the order they are positioned. One can have columns from a run
    /// (`callgrind_annotate`), then a CSV and then another run. The columns of the CSV file will
    /// be surrounded by the columns of the runs.
    pub inputs: Vec<String>,
}

impl Args {
    /// Perform final check for values in the arguments.
    ///
    /// # Returns
    /// If all arguments are well-formed, returns an `Ok`. Otherwise, returns an `Err`.
    pub fn validated(mut self) -> Result<Self> {
        self.check_csv_names_count()?;
        self.check_input_length()?;
        self.sanitize_show();
        Ok(self)
    }

    /// Check that the number of names in `csv_names` matches the number of runs in `inputs`.
    fn check_csv_names_count(&self) -> Result<()> {
        if !self.csv_names.is_empty() {
            let runs_count = self
                .inputs
                .iter()
                .filter(|file| {
                    !Path::new(file)
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("csv"))
                })
                .count();
            if runs_count != self.csv_names.len() {
                bail!("Mismatch between `csv-names` count {} and number of callgrind files {runs_count}", self.csv_names.len());
            }
        }
        Ok(())
    }

    /// Sanitize `show`.
    ///
    /// If `All` is specified, replace with individual columns.
    /// Otherwise, remove duplicates but keep ordering of first occurence.
    fn sanitize_show(&mut self) {
        if self.show.is_empty() || self.show.iter().contains(&Show::All) {
            self.show = vec![Show::IRCountDiff, Show::PercentageDiff, Show::IRCount];
        } else {
            let mut new_show = vec![];
            for show in &self.show {
                if !new_show.contains(show) {
                    new_show.push(*show);
                }
            }
            self.show = new_show;
        }
    }

    /// Make sure we are provided with 1 positional argument at least.
    fn check_input_length(&self) -> Result<()> {
        if self.inputs.is_empty() {
            bail!("No input file")
        }
        Ok(())
    }
}
