use std::path::Path;

use arrow::record_batch::RecordBatch;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

use crate::error::Result;

pub fn read(path: &Path) -> Result<Vec<RecordBatch>> {
    let file = std::fs::File::open(path)?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;
    let reader = builder.build()?;
    let batches: std::result::Result<Vec<_>, _> = reader.collect();
    Ok(batches?)
}

pub fn read_bytes(bytes: Vec<u8>) -> Result<Vec<RecordBatch>> {
    // parquet::ChunkReader is implemented for bytes::Bytes but not Cursor<Vec<u8>>
    let bytes = bytes::Bytes::from(bytes);
    let builder = ParquetRecordBatchReaderBuilder::try_new(bytes)?;
    let reader = builder.build()?;
    let batches: std::result::Result<Vec<_>, _> = reader.collect();
    Ok(batches?)
}
