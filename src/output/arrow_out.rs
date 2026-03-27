use std::io::Write;
use std::sync::Arc;

use arrow::array::{
    ArrayRef, BooleanBuilder, Float64Array, Float64Builder, Int64Array, Int64Builder, ListBuilder,
    StringArray, StringBuilder,
};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::ipc::writer::StreamWriter;
use arrow::record_batch::RecordBatch;
use serde_json::Value;

use crate::error::Result;

/// Infer the element type of a list column by scanning all array values.
fn infer_list_element_type(values: &[Value], col: &str) -> DataType {
    let mut has_int = false;
    let mut has_float = false;
    let mut has_bool = false;
    let mut has_other = false;

    for v in values {
        if let Value::Object(map) = v {
            if let Some(Value::Array(arr)) = map.get(col) {
                for elem in arr {
                    match elem {
                        Value::Number(n) => {
                            if n.is_i64() || n.is_u64() {
                                has_int = true;
                            } else {
                                has_float = true;
                            }
                        }
                        Value::Bool(_) => has_bool = true,
                        Value::Null => {}
                        _ => has_other = true,
                    }
                }
            }
        }
    }

    let kinds = [has_int || has_float, has_bool, has_other]
        .iter()
        .filter(|&&x| x)
        .count();
    if kinds > 1 || has_other {
        DataType::Utf8
    } else if has_float {
        DataType::Float64
    } else if has_int {
        DataType::Int64
    } else if has_bool {
        DataType::Boolean
    } else {
        DataType::Utf8
    }
}

/// Infer the best Arrow DataType for a column by scanning all values.
fn infer_column_type(values: &[Value], col: &str) -> DataType {
    let mut has_int = false;
    let mut has_float = false;
    let mut has_bool = false;
    let mut has_array = false;
    let mut has_other = false;

    for v in values {
        if let Value::Object(map) = v {
            match map.get(col) {
                Some(Value::Number(n)) => {
                    if n.is_i64() || n.is_u64() {
                        has_int = true;
                    } else {
                        has_float = true;
                    }
                }
                Some(Value::Bool(_)) => has_bool = true,
                Some(Value::Array(_)) => has_array = true,
                Some(Value::Null) | None => {}
                _ => has_other = true,
            }
        }
    }

    // Any mix of incompatible types falls back to Utf8
    let kinds = [has_int || has_float, has_bool, has_array, has_other]
        .iter()
        .filter(|&&x| x)
        .count();
    if kinds > 1 || has_other {
        return DataType::Utf8;
    }

    if has_array {
        let elem = infer_list_element_type(values, col);
        DataType::List(Arc::new(Field::new("item", elem, true)))
    } else if has_bool {
        DataType::Boolean
    } else if has_float {
        DataType::Float64
    } else if has_int {
        DataType::Int64
    } else {
        DataType::Utf8
    }
}

fn build_object_column(values: &[Value], col: &str, dtype: &DataType) -> ArrayRef {
    match dtype {
        DataType::Int64 => {
            let vals: Vec<Option<i64>> = values
                .iter()
                .map(|v| {
                    if let Value::Object(map) = v {
                        map.get(col).and_then(|v| v.as_i64())
                    } else {
                        None
                    }
                })
                .collect();
            Arc::new(Int64Array::from(vals))
        }
        DataType::Float64 => {
            let vals: Vec<Option<f64>> = values
                .iter()
                .map(|v| {
                    if let Value::Object(map) = v {
                        map.get(col).and_then(|v| v.as_f64())
                    } else {
                        None
                    }
                })
                .collect();
            Arc::new(Float64Array::from(vals))
        }
        DataType::Boolean => {
            let vals: Vec<Option<bool>> = values
                .iter()
                .map(|v| {
                    if let Value::Object(map) = v {
                        map.get(col).and_then(|v| v.as_bool())
                    } else {
                        None
                    }
                })
                .collect();
            Arc::new(arrow::array::BooleanArray::from(vals))
        }
        DataType::List(item_field) => match item_field.data_type() {
            DataType::Int64 => {
                let mut builder = ListBuilder::new(Int64Builder::new());
                for v in values {
                    if let Value::Object(map) = v {
                        if let Some(Value::Array(arr)) = map.get(col) {
                            for elem in arr {
                                builder.values().append_option(elem.as_i64());
                            }
                            builder.append(true);
                        } else {
                            builder.append(false);
                        }
                    } else {
                        builder.append(false);
                    }
                }
                Arc::new(builder.finish())
            }
            DataType::Float64 => {
                let mut builder = ListBuilder::new(Float64Builder::new());
                for v in values {
                    if let Value::Object(map) = v {
                        if let Some(Value::Array(arr)) = map.get(col) {
                            for elem in arr {
                                builder.values().append_option(elem.as_f64());
                            }
                            builder.append(true);
                        } else {
                            builder.append(false);
                        }
                    } else {
                        builder.append(false);
                    }
                }
                Arc::new(builder.finish())
            }
            DataType::Boolean => {
                let mut builder = ListBuilder::new(BooleanBuilder::new());
                for v in values {
                    if let Value::Object(map) = v {
                        if let Some(Value::Array(arr)) = map.get(col) {
                            for elem in arr {
                                builder.values().append_option(elem.as_bool());
                            }
                            builder.append(true);
                        } else {
                            builder.append(false);
                        }
                    } else {
                        builder.append(false);
                    }
                }
                Arc::new(builder.finish())
            }
            _ => {
                // Utf8 list
                let mut builder = ListBuilder::new(StringBuilder::new());
                for v in values {
                    if let Value::Object(map) = v {
                        if let Some(Value::Array(arr)) = map.get(col) {
                            for elem in arr {
                                match elem {
                                    Value::String(s) => builder.values().append_value(s),
                                    Value::Null => builder.values().append_null(),
                                    other => builder.values().append_value(other.to_string()),
                                }
                            }
                            builder.append(true);
                        } else {
                            builder.append(false);
                        }
                    } else {
                        builder.append(false);
                    }
                }
                Arc::new(builder.finish())
            }
        },
        _ => {
            // Utf8 scalar
            let vals: Vec<Option<String>> = values
                .iter()
                .map(|v| {
                    if let Value::Object(map) = v {
                        map.get(col).map(|v| match v {
                            Value::String(s) => s.clone(),
                            Value::Null => String::new(),
                            other => other.to_string(),
                        })
                    } else {
                        None
                    }
                })
                .collect();
            Arc::new(StringArray::from(
                vals.iter().map(|s| s.as_deref()).collect::<Vec<_>>(),
            ))
        }
    }
}

/// Write JSON values as Arrow IPC stream, preserving numeric, boolean, string,
/// and list (array) types per column.
pub fn write<W: Write>(out: &mut W, values: &[Value]) -> Result<()> {
    if values.is_empty() {
        return Ok(());
    }

    // Collect columns in insertion order
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

    let (schema, batch) = if columns.is_empty() {
        // Non-object values: single "value" string column
        let schema = Arc::new(Schema::new(vec![Field::new("value", DataType::Utf8, true)]));
        let arr: ArrayRef = Arc::new(StringArray::from(
            values
                .iter()
                .map(|v| match v {
                    Value::String(s) => Some(s.as_str().to_string()),
                    other => Some(other.to_string()),
                })
                .collect::<Vec<_>>()
                .iter()
                .map(|s| s.as_deref())
                .collect::<Vec<_>>(),
        ));
        let batch = RecordBatch::try_new(schema.clone(), vec![arr])?;
        (schema, batch)
    } else {
        let dtypes: Vec<DataType> = columns
            .iter()
            .map(|c| infer_column_type(values, c))
            .collect();

        let fields: Vec<Field> = columns
            .iter()
            .zip(dtypes.iter())
            .map(|(c, dt)| Field::new(c.as_str(), dt.clone(), true))
            .collect();
        let schema = Arc::new(Schema::new(fields));

        let arrays: Vec<ArrayRef> = columns
            .iter()
            .zip(dtypes.iter())
            .map(|(col, dtype)| build_object_column(values, col, dtype))
            .collect();

        let batch = RecordBatch::try_new(schema.clone(), arrays)?;
        (schema, batch)
    };

    let mut writer = StreamWriter::try_new(out, &schema)?;
    writer.write(&batch)?;
    writer.finish()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{Array, BooleanArray, Float64Array, Int64Array, ListArray, StringArray};
    use arrow::ipc::reader::StreamReader;
    use serde_json::json;

    fn run(values: &[Value]) -> RecordBatch {
        let mut buf = Vec::new();
        write(&mut buf, values).unwrap();
        let mut reader = StreamReader::try_new(std::io::Cursor::new(buf), None).unwrap();
        reader.next().unwrap().unwrap()
    }

    // ── infer_column_type: scalars ────────────────────────────────────────────

    #[test]
    fn infer_int() {
        assert_eq!(
            infer_column_type(&[json!({"x": 1}), json!({"x": 2})], "x"),
            DataType::Int64
        );
    }

    #[test]
    fn infer_float() {
        assert_eq!(
            infer_column_type(&[json!({"x": 1.5}), json!({"x": 2.0})], "x"),
            DataType::Float64
        );
    }

    #[test]
    fn infer_mixed_int_float_gives_float() {
        assert_eq!(
            infer_column_type(&[json!({"x": 1}), json!({"x": 1.5})], "x"),
            DataType::Float64
        );
    }

    #[test]
    fn infer_bool() {
        assert_eq!(
            infer_column_type(&[json!({"x": true}), json!({"x": false})], "x"),
            DataType::Boolean
        );
    }

    #[test]
    fn infer_string() {
        assert_eq!(
            infer_column_type(&[json!({"x": "a"}), json!({"x": "b"})], "x"),
            DataType::Utf8
        );
    }

    #[test]
    fn infer_nulls_only_gives_utf8() {
        assert_eq!(
            infer_column_type(&[json!({"x": null}), json!({})], "x"),
            DataType::Utf8
        );
    }

    #[test]
    fn infer_mixed_bool_and_int_gives_utf8() {
        assert_eq!(
            infer_column_type(&[json!({"x": true}), json!({"x": 1})], "x"),
            DataType::Utf8
        );
    }

    // ── infer_column_type: lists ──────────────────────────────────────────────

    #[test]
    fn infer_list_of_int() {
        assert_eq!(
            infer_column_type(&[json!({"x": [1, 2, 3]})], "x"),
            DataType::List(Arc::new(Field::new("item", DataType::Int64, true)))
        );
    }

    #[test]
    fn infer_list_of_float() {
        assert_eq!(
            infer_column_type(&[json!({"x": [1.1, 2.2]})], "x"),
            DataType::List(Arc::new(Field::new("item", DataType::Float64, true)))
        );
    }

    #[test]
    fn infer_list_of_mixed_int_and_float_gives_float() {
        assert_eq!(
            infer_column_type(&[json!({"x": [1, 2.5]})], "x"),
            DataType::List(Arc::new(Field::new("item", DataType::Float64, true)))
        );
    }

    #[test]
    fn infer_list_of_strings() {
        assert_eq!(
            infer_column_type(&[json!({"x": ["a", "b"]})], "x"),
            DataType::List(Arc::new(Field::new("item", DataType::Utf8, true)))
        );
    }

    #[test]
    fn infer_list_of_booleans() {
        assert_eq!(
            infer_column_type(&[json!({"x": [true, false, true]})], "x"),
            DataType::List(Arc::new(Field::new("item", DataType::Boolean, true)))
        );
    }

    #[test]
    fn infer_list_of_mixed_bool_and_int_gives_utf8() {
        assert_eq!(
            infer_column_type(&[json!({"x": [true, 1]})], "x"),
            DataType::List(Arc::new(Field::new("item", DataType::Utf8, true)))
        );
    }

    #[test]
    fn infer_list_mixed_with_scalar_gives_utf8() {
        // column has an array in one row and a scalar in another → Utf8
        assert_eq!(
            infer_column_type(&[json!({"x": [1, 2]}), json!({"x": 3})], "x"),
            DataType::Utf8
        );
    }

    // ── round-trip: scalars ───────────────────────────────────────────────────

    #[test]
    fn round_trip_preserves_int64() {
        let values = vec![
            json!({"age": 30, "name": "Alice"}),
            json!({"age": 25, "name": "Bob"}),
        ];
        let batch = run(&values);
        let schema = batch.schema();
        assert_eq!(
            schema.field_with_name("age").unwrap().data_type(),
            &DataType::Int64
        );
        assert_eq!(
            schema.field_with_name("name").unwrap().data_type(),
            &DataType::Utf8
        );
        let ages = batch.column_by_name("age").unwrap();
        let ages = ages.as_any().downcast_ref::<Int64Array>().unwrap();
        assert_eq!(ages.value(0), 30);
        assert_eq!(ages.value(1), 25);
    }

    #[test]
    fn round_trip_preserves_float64() {
        let values = vec![json!({"score": 9.5}), json!({"score": 7.0})];
        let batch = run(&values);
        assert_eq!(
            batch.schema().field_with_name("score").unwrap().data_type(),
            &DataType::Float64
        );
        let col = batch.column_by_name("score").unwrap();
        let col = col.as_any().downcast_ref::<Float64Array>().unwrap();
        assert!((col.value(0) - 9.5).abs() < f64::EPSILON);
    }

    #[test]
    fn round_trip_preserves_boolean() {
        let values = vec![json!({"active": true}), json!({"active": false})];
        let batch = run(&values);
        assert_eq!(
            batch
                .schema()
                .field_with_name("active")
                .unwrap()
                .data_type(),
            &DataType::Boolean
        );
        let col = batch.column_by_name("active").unwrap();
        let col = col.as_any().downcast_ref::<BooleanArray>().unwrap();
        assert!(col.value(0));
        assert!(!col.value(1));
    }

    #[test]
    fn round_trip_string_column() {
        let values = vec![json!({"dept": "Engineering"}), json!({"dept": "HR"})];
        let batch = run(&values);
        assert_eq!(
            batch.schema().field_with_name("dept").unwrap().data_type(),
            &DataType::Utf8
        );
        let col = batch.column_by_name("dept").unwrap();
        let col = col.as_any().downcast_ref::<StringArray>().unwrap();
        assert_eq!(col.value(0), "Engineering");
    }

    // ── round-trip: lists ─────────────────────────────────────────────────────

    #[test]
    fn round_trip_list_of_int64() {
        let values = vec![json!({"scores": [90, 85, 92]}), json!({"scores": [70, 75]})];
        let batch = run(&values);
        let expected_dtype = DataType::List(Arc::new(Field::new("item", DataType::Int64, true)));
        assert_eq!(
            batch
                .schema()
                .field_with_name("scores")
                .unwrap()
                .data_type(),
            &expected_dtype
        );
        let col = batch.column_by_name("scores").unwrap();
        let col = col.as_any().downcast_ref::<ListArray>().unwrap();
        assert_eq!(col.len(), 2);
        // first row has 3 elements
        let row0 = col.value(0);
        let row0 = row0.as_any().downcast_ref::<Int64Array>().unwrap();
        assert_eq!(row0.values(), &[90, 85, 92]);
        // second row has 2 elements
        let row1 = col.value(1);
        let row1 = row1.as_any().downcast_ref::<Int64Array>().unwrap();
        assert_eq!(row1.values(), &[70, 75]);
    }

    #[test]
    fn round_trip_list_of_float64() {
        let values = vec![
            json!({"ratios": [0.9, 0.85, 0.95]}),
            json!({"ratios": [0.7, 0.75]}),
        ];
        let batch = run(&values);
        let expected_dtype = DataType::List(Arc::new(Field::new("item", DataType::Float64, true)));
        assert_eq!(
            batch
                .schema()
                .field_with_name("ratios")
                .unwrap()
                .data_type(),
            &expected_dtype
        );
        let col = batch.column_by_name("ratios").unwrap();
        let col = col.as_any().downcast_ref::<ListArray>().unwrap();
        let row0 = col.value(0);
        let row0 = row0.as_any().downcast_ref::<Float64Array>().unwrap();
        assert!((row0.value(0) - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn round_trip_list_of_strings() {
        let values = vec![json!({"tags": ["eng", "python"]}), json!({"tags": ["mkt"]})];
        let batch = run(&values);
        let expected_dtype = DataType::List(Arc::new(Field::new("item", DataType::Utf8, true)));
        assert_eq!(
            batch.schema().field_with_name("tags").unwrap().data_type(),
            &expected_dtype
        );
        let col = batch.column_by_name("tags").unwrap();
        let col = col.as_any().downcast_ref::<ListArray>().unwrap();
        let row0 = col.value(0);
        let row0 = row0.as_any().downcast_ref::<StringArray>().unwrap();
        assert_eq!(row0.value(0), "eng");
        assert_eq!(row0.value(1), "python");
    }

    #[test]
    fn round_trip_list_with_nulls() {
        let values = vec![json!({"scores": [1, null, 3]}), json!({"scores": null})];
        let batch = run(&values);
        let col = batch.column_by_name("scores").unwrap();
        let col = col.as_any().downcast_ref::<ListArray>().unwrap();
        assert!(col.is_valid(0));
        assert!(col.is_null(1));
    }

    #[test]
    fn round_trip_list_of_booleans() {
        let values = vec![
            json!({"flags": [true, false, true]}),
            json!({"flags": [false, true]}),
        ];
        let batch = run(&values);
        let expected_dtype = DataType::List(Arc::new(Field::new("item", DataType::Boolean, true)));
        assert_eq!(
            batch.schema().field_with_name("flags").unwrap().data_type(),
            &expected_dtype
        );
        let col = batch.column_by_name("flags").unwrap();
        let col = col.as_any().downcast_ref::<ListArray>().unwrap();
        let row0 = col.value(0);
        let row0 = row0
            .as_any()
            .downcast_ref::<arrow::array::BooleanArray>()
            .unwrap();
        assert!(row0.value(0));
        assert!(!row0.value(1));
        assert!(row0.value(2));
    }

    #[test]
    fn round_trip_null_list_entry() {
        // a row missing the array field entirely should produce a null list
        let values = vec![json!({"scores": [1, 2]}), json!({})];
        let batch = run(&values);
        let col = batch.column_by_name("scores").unwrap();
        let col = col.as_any().downcast_ref::<ListArray>().unwrap();
        assert!(col.is_valid(0));
        assert!(col.is_null(1));
    }
}
