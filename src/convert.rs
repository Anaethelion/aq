use arrow::json::writer::LineDelimitedWriter;
use arrow::record_batch::RecordBatch;
use serde_json::Value;

use crate::error::{ArrowCliError, Result};

/// Convert Arrow record batches to a Vec of JSON objects (one per row).
pub fn batches_to_json_rows(batches: &[RecordBatch]) -> Result<Vec<Value>> {
    if batches.is_empty() {
        return Ok(vec![]);
    }

    // Write all batches as NDJSON into a buffer
    let mut buf: Vec<u8> = Vec::new();
    {
        let mut writer = LineDelimitedWriter::new(&mut buf);
        for batch in batches {
            writer.write(batch)?;
        }
        writer.finish()?;
    }

    // Parse each NDJSON line as a serde_json::Value
    let text = String::from_utf8(buf)
        .map_err(|e| ArrowCliError::JqRuntime(format!("UTF-8 error: {e}")))?;

    let rows = text
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| serde_json::from_str::<Value>(l))
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(rows)
}
