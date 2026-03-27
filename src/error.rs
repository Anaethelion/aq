use thiserror::Error;

#[derive(Error, Debug)]
pub enum ArrowCliError {
    #[error("Cannot auto-detect format for '{0}': use --format to specify")]
    FormatDetectionFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Arrow error: {0}")]
    Arrow(#[from] arrow::error::ArrowError),

    #[error("Parquet error: {0}")]
    Parquet(#[from] parquet::errors::ParquetError),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("jq parse error: {0}")]
    JqParse(String),

    #[error("jq runtime error: {0}")]
    JqRuntime(String),
}

pub type Result<T> = std::result::Result<T, ArrowCliError>;
