#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_lossless
)]

use std::path::Path;

use anyhow::{bail, Result};
use clap::Parser;

use crate::{
    args::{Args, RelativeTo, SortByField},
    display::display,
    runs::{Records, Run},
};

mod args;
mod callgrind;
mod display;
mod runs;

/// Detect if a file is CSV by examining its content rather than extension.
fn is_csv_file(path: &str) -> Result<bool> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut first_line = String::new();
    
    if reader.read_line(&mut first_line)? == 0 {
        return Ok(false); // Empty file
    }
    
    // Check if first line looks like CSV (contains commas and no typical callgrind markers)
    let line = first_line.trim();
    if line.contains(',') && 
       !line.contains("Profile data file") && 
       !line.contains("Profiled target") && 
       !line.contains("Events recorded") &&
       !line.starts_with("Ir") &&
       !line.starts_with("---") {
        return Ok(true);
    }
    
    Ok(false)
}

/// Parse inputs from the configuration into a [`Records`].
///
/// Files are detected as CSV or `callgrind_annotate` based on content, not extension.
/// CSV files are loaded as multiple runs, `callgrind_annotate` files as single runs.
fn parse_records(config: &Args) -> Result<Records> {
    let mut records = Records::new();
    let mut callgrind_file_count = 0;
    
    for input in &config.inputs {
        if is_csv_file(input)? {
            // Load CSV file and merge its records
            let csv_records = Records::from_csv_file(input, &config.string_replace)?;
            for (i, run_name) in csv_records.run_names.iter().enumerate() {
                let mut run = Run::new_named(run_name.clone());
                run.total_ir = csv_records.runs_total_irs[i];
                
                for symbol in &csv_records.symbols {
                    if symbol.irs[i] > 0 {
                        run.add_ir(&symbol.name, symbol.irs[i]);
                    }
                }
                
                records.add_run(run);
            }
        } else {
            // Load callgrind annotate file
            let mut run = Run::from_callgrind_annotate_file(input, &config.string_replace)?;
            
            // Apply custom name if available
            if callgrind_file_count < config.csv_names.len() {
                run.name.clone_from(&config.csv_names[callgrind_file_count]);
            } else if run.name.is_empty() {
                // If no name provided and run doesn't have a name, use filename
                run.name = Path::new(input)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(input)
                    .to_string();
            }
            
            records.add_run(run);
            callgrind_file_count += 1;
        }
    }
    Ok(records)
}

fn main() -> Result<()> {
    let config = Args::parse().validated()?;
    let mut records = parse_records(&config)?;
    if records.n_runs() == 0 {
        bail!("No input run");
    }
    if let RelativeTo::Column(x) = &config.relative_to {
        if (*x as usize) >= records.n_runs() {
            bail!("--relative-to column index out of range");
        }
    }
    if let SortByField::ColumnIR(x) = &config.sort_by.field {
        if (*x as usize) >= records.n_runs() {
            bail!("--sort-by column index out of range");
        }
    }

    records.sort(config.sort_by)?;
    display(&config, &records);

    // Export to CSV if requested
    if !config.csv_export.is_empty() {
        // Determine reference column for calculations
        let reference_column = match &config.relative_to {
            RelativeTo::Last => records.n_runs().saturating_sub(1),
            RelativeTo::Previous | RelativeTo::First => 0, // For previous, we'll use first as reference in CSV
            RelativeTo::Column(x) => (*x as usize).min(records.n_runs().saturating_sub(1)),
        };

        if config.csv_all_data || config.csv_percentages || config.csv_differences {
            records.to_csv_file_enhanced(
                &config.csv_export,
                config.csv_percentages,
                config.csv_differences,
                config.csv_all_data,
                reference_column,
            )?;
        } else {
            records.to_csv_file(&config.csv_export)?;
        }
    }

    Ok(())
}
