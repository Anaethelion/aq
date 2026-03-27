use std::io::{self, Read};
use std::path::Path;

use arrow::record_batch::RecordBatch;

use crate::cli::InputFormat;
use crate::detect;
use crate::error::{ArrowCliError, Result};

pub mod arrow_ipc;
pub mod csv;
pub mod ndjson;
pub mod parquet;

/// Read all record batches from a file, auto-detecting format if not specified.
pub fn read_file(
    path: &Path,
    format: Option<&InputFormat>,
    delimiter: Option<u8>,
) -> Result<Vec<RecordBatch>> {
    let fmt = match format {
        Some(f) => f.clone(),
        None => detect::detect_from_path(path)?,
    };

    match fmt {
        InputFormat::Parquet => parquet::read(path),
        InputFormat::Arrow => arrow_ipc::read_file(path),
        InputFormat::Csv => csv::read_file(path, delimiter),
        InputFormat::Json => ndjson::read_file(path),
    }
}

/// Read all record batches from stdin, using magic bytes to detect format when not specified.
pub fn read_stdin(format: Option<&InputFormat>, delimiter: Option<u8>) -> Result<Vec<RecordBatch>> {
    let mut bytes = Vec::new();
    io::stdin().read_to_end(&mut bytes)?;

    let fmt = match format {
        Some(f) => f.clone(),
        None => detect::detect_from_bytes(&bytes).ok_or_else(|| {
            ArrowCliError::FormatDetectionFailed(
                "stdin (use --format to specify)".to_string(),
            )
        })?,
    };

    match fmt {
        InputFormat::Parquet => parquet::read_bytes(bytes),
        InputFormat::Arrow => {
            use std::io::Cursor;
            arrow_ipc::read_stream(Cursor::new(bytes))
        }
        InputFormat::Csv => {
            use std::io::Cursor;
            csv::read(&mut Cursor::new(bytes), delimiter)
        }
        InputFormat::Json => ndjson::read_bytes(&bytes),
    }
}

/// Concatenate multiple record batch vectors.
pub fn concat_batches(all: Vec<Vec<RecordBatch>>) -> Result<Vec<RecordBatch>> {
    Ok(all.into_iter().flatten().collect())
}
