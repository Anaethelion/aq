use std::io::Write;

use serde_json::Value;

use crate::error::Result;

/// Write JSON values as delimiter-separated values.
pub fn write<W: Write>(out: &mut W, values: &[Value], no_header: bool, delimiter: u8) -> Result<()> {
    if values.is_empty() {
        return Ok(());
    }

    let sep = delimiter as char;

    // Collect columns from all objects
    let mut columns: Vec<String> = Vec::new();
    for val in values {
        if let Value::Object(map) = val {
            for key in map.keys() {
                if !columns.contains(key) {
                    columns.push(key.clone());
                }
            }
        }
    }

    if !no_header && !columns.is_empty() {
        writeln!(out, "{}", columns.join(&sep.to_string()))?;
    }

    for val in values {
        let row: Vec<String> = match val {
            Value::Object(map) => columns
                .iter()
                .map(|col| map.get(col).map(|v| csv_cell(v, delimiter)).unwrap_or_default())
                .collect(),
            other => vec![csv_cell(other, delimiter)],
        };
        writeln!(out, "{}", row.join(&sep.to_string()))?;
    }
    Ok(())
}

fn csv_cell(val: &Value, delimiter: u8) -> String {
    let sep = delimiter as char;
    match val {
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => quote_if_needed(s, sep),
        other => {
            let s = other.to_string();
            quote_if_needed(&s, sep)
        }
    }
}

fn quote_if_needed(s: &str, sep: char) -> String {
    if s.contains(sep) || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn csv(values: &[serde_json::Value], no_header: bool) -> String {
        let mut out = Vec::new();
        write(&mut out, values, no_header, b',').unwrap();
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn basic_with_header() {
        let values = vec![json!({"name": "Alice", "age": 30})];
        let out = csv(&values, false);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("name") && lines[0].contains("age"));
        assert!(lines[1].contains("Alice") && lines[1].contains("30"));
    }

    #[test]
    fn no_header_skips_header_row() {
        let values = vec![json!({"name": "Alice", "age": 30})];
        let out = csv(&values, true);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("Alice") && lines[0].contains("30"));
    }

    #[test]
    fn empty_input_writes_nothing() {
        assert!(csv(&[], false).is_empty());
    }

    #[test]
    fn comma_in_value_is_quoted() {
        let values = vec![json!({"name": "Smith, John"})];
        let out = csv(&values, true);
        assert!(out.contains("\"Smith, John\""));
    }

    #[test]
    fn quote_in_value_is_escaped() {
        let values = vec![json!({"name": "say \"hi\""})];
        let out = csv(&values, true);
        assert!(out.contains("\"say \"\"hi\"\"\""));
    }

    #[test]
    fn null_renders_as_empty() {
        let values = vec![json!({"name": null})];
        let out = csv(&values, true);
        assert_eq!(out.trim(), "");
    }

    #[test]
    fn multiple_rows() {
        let values = vec![
            json!({"name": "Alice", "age": 30}),
            json!({"name": "Bob",   "age": 25}),
        ];
        let out = csv(&values, false);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[1].contains("Alice"));
        assert!(lines[2].contains("Bob"));
    }

    #[test]
    fn custom_delimiter_tab() {
        let values = vec![json!({"name": "Alice", "age": 30})];
        let mut out = Vec::new();
        write(&mut out, &values, true, b'\t').unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains('\t'));
        assert!(!text.contains(','));
    }

    #[test]
    fn custom_delimiter_semicolon() {
        let values = vec![json!({"a": 1, "b": 2})];
        let mut out = Vec::new();
        write(&mut out, &values, false, b';').unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains(';'));
    }
}
