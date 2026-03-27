use comfy_table::{ContentArrangement, Table};
use serde_json::Value;

/// Render a list of JSON values as a pretty table.
/// All values should be objects; if they aren't, falls back to rendering them as strings.
pub fn render(values: &[Value], no_header: bool) -> String {
    if values.is_empty() {
        return String::new();
    }

    // Collect column names from the union of all object keys (in first-occurrence order)
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

    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);

    if !no_header && !columns.is_empty() {
        table.set_header(columns.iter().map(|c| c.as_str()).collect::<Vec<_>>());
    }

    for val in values {
        match val {
            Value::Object(map) => {
                if columns.is_empty() {
                    // No columns detected — print key: value pairs
                    for (k, v) in map {
                        table.add_row(vec![k.as_str(), &format_value(v)]);
                    }
                } else {
                    let row: Vec<String> = columns
                        .iter()
                        .map(|col| {
                            map.get(col)
                                .map(format_value)
                                .unwrap_or_else(|| "null".to_string())
                        })
                        .collect();
                    table.add_row(row);
                }
            }
            other => {
                table.add_row(vec![format_value(other)]);
            }
        }
    }

    table.to_string()
}

fn format_value(val: &Value) -> String {
    match val {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}
