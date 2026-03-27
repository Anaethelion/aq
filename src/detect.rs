use std::path::Path;

use crate::cli::InputFormat;
use crate::error::{ArrowCliError, Result};

pub fn detect_from_path(path: &Path) -> Result<InputFormat> {
    match path.extension().and_then(|e| e.to_str()) {
        Some("parquet") => Ok(InputFormat::Parquet),
        Some("arrow") => Ok(InputFormat::Arrow),
        Some("csv") => Ok(InputFormat::Csv),
        Some("json") | Some("ndjson") => Ok(InputFormat::Json),
        other => Err(ArrowCliError::FormatDetectionFailed(
            other
                .map(|e| format!(".{e}"))
                .unwrap_or_else(|| path.display().to_string()),
        )),
    }
}

/// Detect format from the first bytes of stdin (magic bytes).
pub fn detect_from_bytes(header: &[u8]) -> Option<InputFormat> {
    if header.starts_with(b"PAR1") {
        return Some(InputFormat::Parquet);
    }
    // Arrow IPC stream magic: 0xFFFFFFFF followed by "ARROW1"
    if header.starts_with(&[0xFF, 0xFF, 0xFF, 0xFF]) {
        return Some(InputFormat::Arrow);
    }
    // Arrow IPC file magic: "ARROW1\0\0"
    if header.starts_with(b"ARROW1\0\0") {
        return Some(InputFormat::Arrow);
    }
    // JSON/NDJSON: starts with '{' or '[' (possibly with whitespace)
    if let Some(b) = header.iter().find(|&&b| !b.is_ascii_whitespace()) {
        if *b == b'{' || *b == b'[' {
            return Some(InputFormat::Json);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    // detect_from_path

    #[test]
    fn path_parquet() {
        assert_eq!(
            detect_from_path(Path::new("data.parquet")).unwrap(),
            InputFormat::Parquet
        );
    }

    #[test]
    fn path_arrow() {
        assert_eq!(
            detect_from_path(Path::new("data.arrow")).unwrap(),
            InputFormat::Arrow
        );
    }

    #[test]
    fn path_csv() {
        assert_eq!(
            detect_from_path(Path::new("data.csv")).unwrap(),
            InputFormat::Csv
        );
    }

    #[test]
    fn path_json() {
        assert_eq!(
            detect_from_path(Path::new("data.json")).unwrap(),
            InputFormat::Json
        );
    }

    #[test]
    fn path_ndjson() {
        assert_eq!(
            detect_from_path(Path::new("data.ndjson")).unwrap(),
            InputFormat::Json
        );
    }

    #[test]
    fn path_unknown_extension_errors() {
        assert!(detect_from_path(Path::new("data.xyz")).is_err());
    }

    #[test]
    fn path_no_extension_errors() {
        assert!(detect_from_path(Path::new("datafile")).is_err());
    }

    // detect_from_bytes

    #[test]
    fn bytes_parquet_magic() {
        assert_eq!(
            detect_from_bytes(b"PAR1\x00\x00"),
            Some(InputFormat::Parquet)
        );
    }

    #[test]
    fn bytes_arrow_stream_magic() {
        let header = [0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00];
        assert_eq!(detect_from_bytes(&header), Some(InputFormat::Arrow));
    }

    #[test]
    fn bytes_arrow_file_magic() {
        assert_eq!(
            detect_from_bytes(b"ARROW1\0\0rest"),
            Some(InputFormat::Arrow)
        );
    }

    #[test]
    fn bytes_json_object() {
        assert_eq!(detect_from_bytes(b"{\"a\":1}"), Some(InputFormat::Json));
    }

    #[test]
    fn bytes_json_array() {
        assert_eq!(detect_from_bytes(b"[1,2,3]"), Some(InputFormat::Json));
    }

    #[test]
    fn bytes_json_with_leading_whitespace() {
        assert_eq!(detect_from_bytes(b"  \n{\"a\":1}"), Some(InputFormat::Json));
    }

    #[test]
    fn bytes_unrecognized_returns_none() {
        assert_eq!(detect_from_bytes(b"hello world"), None);
    }

    #[test]
    fn bytes_empty_returns_none() {
        assert_eq!(detect_from_bytes(b""), None);
    }
}
