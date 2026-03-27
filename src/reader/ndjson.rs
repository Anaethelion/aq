use std::io::{BufReader, Cursor, Seek, SeekFrom};
use std::path::Path;

use arrow::json::reader::infer_json_schema_from_seekable;
use arrow::json::ReaderBuilder;
use arrow::record_batch::RecordBatch;

use crate::error::Result;

pub fn read_file(path: &Path) -> Result<Vec<RecordBatch>> {
    let file = std::fs::File::open(path)?;
    let mut buf = BufReader::new(file);
    read_seekable(&mut buf)
}

pub fn read_bytes(bytes: &[u8]) -> Result<Vec<RecordBatch>> {
    let mut cursor = BufReader::new(Cursor::new(bytes));
    read_seekable(&mut cursor)
}

fn read_seekable<R: std::io::Read + std::io::BufRead + Seek>(
    reader: &mut R,
) -> Result<Vec<RecordBatch>> {
    // Peek to check if the input is empty before letting Arrow try to infer schema,
    // which would produce a cryptic error on empty input.
    let start = reader.stream_position()?;
    let mut probe = [0u8; 1];
    let has_content = reader.read(&mut probe)? > 0 && !probe[0].is_ascii_whitespace() || {
        // drain remaining whitespace to check for non-empty content
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        buf.iter().any(|b| !b.is_ascii_whitespace())
    };
    reader.seek(SeekFrom::Start(start))?;

    if !has_content {
        return Ok(vec![]);
    }

    let (schema, _) = infer_json_schema_from_seekable(&mut *reader, None)?;
    let schema = std::sync::Arc::new(schema);
    reader.seek(SeekFrom::Start(start))?;
    let r = ReaderBuilder::new(schema).build(reader)?;
    let batches: std::result::Result<Vec<_>, _> = r.collect();
    Ok(batches?)
}
