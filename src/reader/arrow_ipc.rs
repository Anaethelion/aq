use std::io::{Cursor, Read};
use std::path::Path;

use arrow::ipc::reader::{FileReader, StreamReader};
use arrow::record_batch::RecordBatch;

use crate::error::{ArrowCliError, Result};

/// Arrow IPC file format magic bytes: "ARROW1\0\0"
const FILE_MAGIC: &[u8] = b"ARROW1\0\0";
/// Arrow IPC stream continuation marker: 0xFFFFFFFF as little-endian i32 = -1
const CONTINUATION: &[u8] = &[0xFF, 0xFF, 0xFF, 0xFF];
/// Sanity cap on schema metadata size (16 MiB)
const MAX_META_LEN: i32 = 16 * 1024 * 1024;

pub fn read_file(path: &Path) -> Result<Vec<RecordBatch>> {
    let mut file = std::fs::File::open(path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    read_bytes(&bytes)
}

pub fn read_stream<R: Read>(mut reader: R) -> Result<Vec<RecordBatch>> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    read_bytes(&bytes)
}

/// Detect Arrow IPC format from raw bytes and dispatch to the right reader.
pub fn read_bytes(bytes: &[u8]) -> Result<Vec<RecordBatch>> {
    if bytes.starts_with(FILE_MAGIC) {
        let reader = FileReader::try_new(Cursor::new(bytes), None)?;
        let batches: std::result::Result<Vec<_>, _> = reader.collect();
        return Ok(batches?);
    }

    // Stream format: validate the header before handing to StreamReader,
    // which would panic on an invalid metadata length allocation.
    if bytes.len() >= 8 {
        let (marker, rest) = bytes.split_at(4);
        let meta_bytes = if marker == CONTINUATION {
            // Standard: 0xFFFFFFFF continuation marker, then 4-byte length
            &rest[..4]
        } else {
            // Legacy: first 4 bytes ARE the length
            marker
        };
        let meta_len = i32::from_le_bytes(meta_bytes.try_into().unwrap());
        if meta_len <= 0 || meta_len > MAX_META_LEN {
            return Err(ArrowCliError::Arrow(arrow::error::ArrowError::ParseError(
                format!(
                    "Arrow IPC stream has invalid metadata length: {meta_len}. \
                     First 8 bytes: {:02X?}. \
                     Input may not be Arrow IPC format — try --format ndjson or --format csv.",
                    &bytes[..bytes.len().min(8)]
                ),
            )));
        }
    }

    let reader = StreamReader::try_new(Cursor::new(bytes), None)?;
    let batches: std::result::Result<Vec<_>, _> = reader.collect();
    Ok(batches?)
}
