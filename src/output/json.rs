use std::io::Write;

use serde_json::Value;

use crate::error::Result;

/// Write each value as a newline-delimited JSON (one per line).
pub fn write_ndjson<W: Write>(out: &mut W, values: &[Value]) -> Result<()> {
    for val in values {
        serde_json::to_writer(&mut *out, val)?;
        writeln!(out)?;
    }
    Ok(())
}

/// Write each value as a raw string (no JSON quoting for strings; other types print as JSON).
pub fn write_raw<W: Write>(out: &mut W, values: &[Value]) -> Result<()> {
    for val in values {
        match val {
            Value::String(s) => writeln!(out, "{s}")?,
            other => {
                serde_json::to_writer(&mut *out, other)?;
                writeln!(out)?;
            }
        }
    }
    Ok(())
}

/// Write values as a pretty-printed JSON array.
pub fn write_pretty<W: Write>(out: &mut W, values: &[Value], compact: bool) -> Result<()> {
    let arr = Value::Array(values.to_vec());
    if compact {
        serde_json::to_writer(&mut *out, &arr)?;
    } else {
        serde_json::to_writer_pretty(&mut *out, &arr)?;
    }
    writeln!(out)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn ndjson_one_object_per_line() {
        let values = vec![json!({"a": 1}), json!({"b": 2})];
        let mut out = Vec::new();
        write_ndjson(&mut out, &values).unwrap();
        let text = String::from_utf8(out).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(serde_json::from_str::<serde_json::Value>(lines[0]).unwrap(), json!({"a": 1}));
        assert_eq!(serde_json::from_str::<serde_json::Value>(lines[1]).unwrap(), json!({"b": 2}));
    }

    #[test]
    fn ndjson_empty_input_writes_nothing() {
        let mut out = Vec::new();
        write_ndjson(&mut out, &[]).unwrap();
        assert!(out.is_empty());
    }

    #[test]
    fn pretty_json_is_valid_array() {
        let values = vec![json!({"a": 1}), json!({"b": 2})];
        let mut out = Vec::new();
        write_pretty(&mut out, &values, false).unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(&out).unwrap();
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 2);
    }

    #[test]
    fn compact_json_has_no_newlines_in_body() {
        let values = vec![json!({"a": 1}), json!({"b": 2})];
        let mut out = Vec::new();
        write_pretty(&mut out, &values, true).unwrap();
        let text = String::from_utf8(out).unwrap();
        // compact output is a single line (trailing newline only)
        assert_eq!(text.lines().count(), 1);
    }

    #[test]
    fn raw_output_unquotes_strings() {
        let values = vec![json!("Alice"), json!("Bob")];
        let mut out = Vec::new();
        write_raw(&mut out, &values).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert_eq!(text, "Alice\nBob\n");
        assert!(!text.contains('"'));
    }

    #[test]
    fn raw_output_non_string_as_json() {
        let values = vec![json!(42), json!(true), json!(null)];
        let mut out = Vec::new();
        write_raw(&mut out, &values).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert_eq!(text, "42\ntrue\nnull\n");
    }

    #[test]
    fn pretty_json_has_indentation() {
        let values = vec![json!({"a": 1})];
        let mut out = Vec::new();
        write_pretty(&mut out, &values, false).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains('\n'));
        assert!(text.contains("  "));
    }
}
