use std::{fs::File, io::BufReader, path::Path};

use anyhow::{bail, Result};

use crate::args::{SortBy, SortByField, SortByOrder, StringReplacement};

/// Annotations of a run of a binary.
#[derive(Default)]
pub struct Run {
    // The name of the run, if any. This is purely for human readability purposes.
    pub name: String,
    /// The symbols that were hit and their instruction count.
    pub symbols: Vec<AnnotatedSymbol>,
    /// The total number of IR for this run.
    pub total_ir: u64,
}

impl Run {
    /// Create a new run.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new run with a name.
    #[allow(unused)]
    pub fn new_named(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }

    /// Add an IR count for the given symbol in the run.
    ///
    /// This may be called multiple times with the same symbol. Due to inlining, a symbol may end
    /// up in different files at different lines. This function _adds_ the IR count each time.
    ///
    /// ```
    /// # use callgrind_differ::runs::Run;
    /// let mut run = Run::new();
    /// run.add_ir("foo", 12);
    /// run.add_ir("foo", 24);
    /// assert_eq!(run.symbols.iter().find(|sym| sym.name == "foo").unwrap().ir, 36);
    /// ```
    pub fn add_ir(&mut self, symbol: &str, ir: u64) {
        if let Some(ref mut symbol) = self.symbols.iter_mut().find(|sym| sym.name == symbol) {
            symbol.ir += ir;
        } else {
            self.symbols.push(AnnotatedSymbol {
                name: symbol.to_string(),
                ir,
            });
        }
    }

    /// Load a run from a `callgrind_annotate` output file.
    pub fn from_callgrind_annotate_file<P: AsRef<Path>>(
        path: P,
        replacements: &[StringReplacement],
    ) -> Result<Self> {
        Ok(crate::callgrind::parse(
            BufReader::new(File::open(path)?),
            replacements,
        ))
    }
}

/// The annotation records of multiple runs.
///
/// The annotations do make sense only if they all refer to the same binary (though it may be at
/// different stages of development).
#[derive(Default)]
pub struct Records {
    /// The names of the runs, if any. This is purely for human readability purposes.
    ///
    /// In case a name is unknown or unset, a blank string is inserted. The length of `run_names`
    /// must match that of any [`RecordsSymbol`] in [`Self::symbols`].
    pub run_names: Vec<String>,
    /// The total IR of each run.
    pub runs_total_irs: Vec<u64>,
    /// The symbols and their IR count for each run.
    pub symbols: Vec<RecordsSymbol>,
}

impl Records {
    /// Create a new records, ready to insert annotated runs.
    pub fn new() -> Self {
        Records::default()
    }

    /// Add annotations about a run to the records.
    pub fn add_run(&mut self, run: Run) {
        self.assert_invariants();

        for run_symbol in run.symbols {
            // Add an `irs` entry for each symbol.
            if let Some(ref mut symbol) = self
                .symbols
                .iter_mut()
                .find(|symbol| symbol.name == run_symbol.name)
            {
                symbol.irs.push(run_symbol.ir);
            } else {
                // If we can't find the symbol, we have to create it. However, we must already push
                // `self.n_runs()` zeroes into it to account for previous runs.
                let mut new_symbol = RecordsSymbol {
                    name: run_symbol.name,
                    irs: vec![0; self.n_runs()],
                };
                new_symbol.irs.push(run_symbol.ir);
                self.symbols.push(new_symbol);
            }
        }

        // Push the name of the run, this will update [`Self::n_runs`].
        self.run_names.push(run.name);
        self.runs_total_irs.push(run.total_ir);

        let n_runs = self.n_runs();
        // Add a 0 to each symbol that was not hit by the run.
        for ref mut symbol in &mut self.symbols {
            if symbol.irs.len() != n_runs {
                symbol.irs.push(0);
            }
        }

        // As long as the invariants were held before, they should hold now.
        self.assert_invariants();
    }

    /// Sort the symbols according to the given order.
    ///
    /// See [`SortBy`] for more details.
    pub fn sort(&mut self, by: SortBy) -> Result<()> {
        let n = self.n_runs();
        match by.field {
            SortByField::Symbol => self.symbols.sort_by(|a, b| a.name.cmp(&b.name)),
            SortByField::FirstIR => self.symbols.sort_by(|a, b| a.irs[0].cmp(&b.irs[0])),
            SortByField::LastIR => self.symbols.sort_by(|a, b| a.irs[n - 1].cmp(&b.irs[n - 1])),
            SortByField::ColumnIR(x) if (x as usize) < n => self
                .symbols
                .sort_by(|a, b| a.irs[x as usize].cmp(&b.irs[x as usize])),
            SortByField::ColumnIR(x) => bail!("Invalid column {x} (got {n} columns)"),
        }

        if matches!(by.order, SortByOrder::Descending) {
            self.symbols.reverse();
        }

        Ok(())
    }

    /// Return the number of runs that have been stored in `Self`.
    pub fn n_runs(&self) -> usize {
        self.run_names.len()
    }

    /// Make sure that the invariants of the structure are held.
    ///
    /// This function functionally does nothing, but checking integrity is cheap and may save time
    /// in debugging.
    ///
    /// # Panics
    /// This function panics if an invariant is broken.
    pub fn assert_invariants(&self) {
        let n_runs = self.n_runs();

        // The number of runs contained in `self.run_names` must match that of
        // `self.runs_total_irs`.
        assert!(
            n_runs == self.runs_total_irs.len(),
            "Invalid # of total irs (got {}, expected{n_runs})",
            self.runs_total_irs.len()
        );

        // The number of runs contained in `self.run_names` must match that of each symbol in
        // `self.symbols`.
        for symbol in &self.symbols {
            assert!(
                symbol.irs.len() == n_runs,
                "Invalid # of runs for symbol {} (got {}, expected {n_runs})",
                symbol.name,
                symbol.irs.len()
            );
        }
    }

    /// Load records from a CSV file.
    ///
    /// The CSV file should have the following format:
    /// - First column: symbol names
    /// - Subsequent columns: IR counts for each run
    /// - Optional header row (detected automatically)
    pub fn from_csv_file<P: AsRef<Path>>(
        path: P,
        replacements: &[StringReplacement],
    ) -> Result<Self> {
        let file = File::open(path)?;
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(file);

        let mut records = Self::new();
        let mut first_row = true;
        let mut column_names: Vec<String> = Vec::new();

        for result in reader.records() {
            let record = result?;
            
            if record.len() < 2 {
                continue; // Skip rows that don't have at least symbol name and one IR count
            }

            let symbol_name = record.get(0).unwrap_or("").to_string();
            
            // Check if this is a header row
            if first_row && symbol_name.eq_ignore_ascii_case("name") {
                // This is a header row, extract column names
                for i in 1..record.len() {
                    column_names.push(record.get(i).unwrap_or(&format!("Run {}", i)).to_string());
                }
                first_row = false;
                continue;
            }

            if first_row {
                // No header row, generate default column names
                for i in 1..record.len() {
                    column_names.push(format!("Run {}", i));
                }
            }
            first_row = false;

            // Initialize runs if this is the first data row
            if records.n_runs() == 0 {
                records.run_names = column_names.clone();
                records.runs_total_irs = vec![0; column_names.len()];
            }

            // Apply string replacements to symbol name
            let processed_symbol_name = replacements.iter().fold(
                std::borrow::Cow::Borrowed(symbol_name.as_str()),
                |name, replacement| replacement.perform(name)
            );

            let mut symbol = RecordsSymbol {
                name: processed_symbol_name.to_string(),
                irs: Vec::new(),
            };

            // Parse IR counts for each run
            for i in 1..record.len().min(column_names.len() + 1) {
                if let Some(ir_str) = record.get(i) {
                    let ir = ir_str.trim().parse::<u64>().unwrap_or(0);
                    symbol.irs.push(ir);
                    records.runs_total_irs[i - 1] += ir;
                } else {
                    symbol.irs.push(0);
                }
            }

            // Pad with zeros if needed
            while symbol.irs.len() < records.n_runs() {
                symbol.irs.push(0);
            }

            records.symbols.push(symbol);
        }

        records.assert_invariants();
        Ok(records)
    }

    /// Export records to a CSV file.
    pub fn to_csv_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let file = File::create(path)?;
        let mut writer = csv::Writer::from_writer(file);

        // Write header
        let mut header = vec!["name".to_string()];
        header.extend(self.run_names.iter().cloned());
        writer.write_record(&header)?;

        // Write symbol data
        for symbol in &self.symbols {
            let mut record = vec![symbol.name.clone()];
            for ir in &symbol.irs {
                record.push(ir.to_string());
            }
            writer.write_record(&record)?;
        }

        writer.flush()?;
        Ok(())
    }
    
    /// Export records to a CSV file with enhanced options including percentages and differences.
    pub fn to_csv_file_enhanced<P: AsRef<Path>>(
        &self,
        path: P,
        include_percentages: bool,
        include_differences: bool,
        include_all_data: bool,
        reference_column: usize,
    ) -> Result<()> {
        let file = File::create(path)?;
        let mut writer = csv::Writer::from_writer(file);

        // Build header based on options
        let mut header = vec!["name".to_string()];
        
        if include_all_data {
            // Include everything: IR, differences, and percentages
            for (i, run_name) in self.run_names.iter().enumerate() {
                if i == reference_column {
                    header.push(format!("{}_ir", run_name));
                } else {
                    header.push(format!("{}_ir", run_name));
                    header.push(format!("{}_diff", run_name));
                    header.push(format!("{}_pct", run_name));
                }
            }
        } else {
            // Selective inclusion
            for (i, run_name) in self.run_names.iter().enumerate() {
                header.push(format!("{}_ir", run_name));
                if i != reference_column && include_differences {
                    header.push(format!("{}_diff", run_name));
                }
                if i != reference_column && include_percentages {
                    header.push(format!("{}_pct", run_name));
                }
            }
        }
        
        writer.write_record(&header)?;

        // Write symbol data with calculations
        for symbol in &self.symbols {
            let mut record = vec![symbol.name.clone()];
            
            let reference_ir = if reference_column < symbol.irs.len() {
                symbol.irs[reference_column]
            } else {
                0
            };
            
            if include_all_data {
                for (i, &ir) in symbol.irs.iter().enumerate() {
                    record.push(ir.to_string());
                    
                    if i != reference_column {
                        // Calculate difference
                        let diff = (ir as i64) - (reference_ir as i64);
                        record.push(diff.to_string());
                        
                        // Calculate percentage
                        let percentage = if reference_ir == 0 {
                            if ir == 0 { 0.0 } else { 100.0 }
                        } else {
                            ((ir as f64 - reference_ir as f64) / reference_ir as f64) * 100.0
                        };
                        record.push(format!("{:.3}", percentage));
                    }
                }
            } else {
                // Selective data inclusion
                for (i, &ir) in symbol.irs.iter().enumerate() {
                    record.push(ir.to_string());
                    
                    if i != reference_column {
                        if include_differences {
                            let diff = (ir as i64) - (reference_ir as i64);
                            record.push(diff.to_string());
                        }
                        
                        if include_percentages {
                            let percentage = if reference_ir == 0 {
                                if ir == 0 { 0.0 } else { 100.0 }
                            } else {
                                ((ir as f64 - reference_ir as f64) / reference_ir as f64) * 100.0
                            };
                            record.push(format!("{:.3}", percentage));
                        }
                    }
                }
            }
            
            writer.write_record(&record)?;
        }

        writer.flush()?;
        Ok(())
    }
}

/// A symbol in the file and its IR count for a single run.
#[derive(Default)]
pub struct AnnotatedSymbol {
    /// The name of the symbol.
    pub name: String,
    /// The instruction count for that run.
    pub ir: u64,
}

/// A symbol in the file and its IR counts for multiple runs.
#[derive(Default)]
pub struct RecordsSymbol {
    /// The name of the symbol.
    pub name: String,
    /// The instruction counts for different runs.
    ///
    /// When storing a collection of [`RecordsSymbol`]s, care must be taken in order to not assign
    /// an IR count of one run to another (i.e. before inserting, the length of `irs` for each
    /// [`RecordsSymbol`] in the collection must be the same).
    pub irs: Vec<u64>,
}
