use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use arrow::csv::{reader::Format, ReaderBuilder};
use arrow::record_batch::RecordBatch;

use crate::error::Result;

pub fn read_file(path: &Path, delimiter: Option<u8>) -> Result<Vec<RecordBatch>> {
    let mut file = std::fs::File::open(path)?;
    read(&mut file, delimiter)
}

pub fn read<R: Read + Seek>(reader: &mut R, delimiter: Option<u8>) -> Result<Vec<RecordBatch>> {
    let fmt = Format::default()
        .with_header(true)
        .with_delimiter(delimiter.unwrap_or(b','));
    let (schema, _) = fmt.infer_schema(&mut *reader, None)?;
    let schema = std::sync::Arc::new(schema);
    reader.seek(SeekFrom::Start(0))?;
    let csv_reader = ReaderBuilder::new(schema)
        .with_header(true)
        .with_delimiter(delimiter.unwrap_or(b','))
        .build(reader)?;
    let batches: std::result::Result<Vec<_>, _> = csv_reader.collect();
    Ok(batches?)
}
