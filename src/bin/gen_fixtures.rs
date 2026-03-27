/// Generate binary test fixtures that can't be created from text files alone.
/// Run with: cargo run --bin gen_fixtures
use std::sync::Arc;

use arrow::array::{BooleanArray, Int64Array, Int64Builder, ListBuilder, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;

fn main() {
    write_sample_parquet();
    write_complex_parquet();
    println!("fixtures written to tests/fixtures/");
}

fn write_sample_parquet() {
    let schema = Arc::new(Schema::new(vec![
        Field::new("name", DataType::Utf8, false),
        Field::new("age", DataType::Int64, false),
        Field::new("dept", DataType::Utf8, false),
        Field::new("salary", DataType::Int64, false),
    ]));

    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(StringArray::from(vec!["Alice", "Bob", "Charlie", "Diana"])),
            Arc::new(Int64Array::from(vec![30, 25, 35, 28])),
            Arc::new(StringArray::from(vec![
                "Engineering",
                "Marketing",
                "Engineering",
                "HR",
            ])),
            Arc::new(Int64Array::from(vec![90000, 65000, 95000, 70000])),
        ],
    )
    .unwrap();

    let file = std::fs::File::create("tests/fixtures/sample.parquet").unwrap();
    let mut writer = ArrowWriter::try_new(file, schema, None).unwrap();
    writer.write(&batch).unwrap();
    writer.close().unwrap();
    println!("wrote sample.parquet");
}

fn write_complex_parquet() {
    let schema = Arc::new(Schema::new(vec![
        Field::new("name", DataType::Utf8, false),
        Field::new(
            "scores",
            DataType::List(Arc::new(Field::new("item", DataType::Int64, true))),
            false,
        ),
        Field::new(
            "tags",
            DataType::List(Arc::new(Field::new("item", DataType::Utf8, true))),
            false,
        ),
        Field::new(
            "ratios",
            DataType::List(Arc::new(Field::new("item", DataType::Float64, true))),
            false,
        ),
        Field::new("active", DataType::Boolean, false),
    ]));

    // scores column: [[90,85,92], [70,75], [95,88,91,94]]
    let mut scores_builder = ListBuilder::new(Int64Builder::new());
    for row in [&[90i64, 85, 92][..], &[70, 75], &[95, 88, 91, 94]] {
        for &v in row {
            scores_builder.values().append_value(v);
        }
        scores_builder.append(true);
    }

    // tags column
    let mut tags_builder = ListBuilder::new(arrow::array::StringBuilder::new());
    for row in [&["eng", "python"][..], &["mkt"], &["eng", "rust"]] {
        for &v in row {
            tags_builder.values().append_value(v);
        }
        tags_builder.append(true);
    }

    // ratios column
    let mut ratios_builder = ListBuilder::new(arrow::array::Float64Builder::new());
    for row in [&[0.9f64, 0.85, 0.95][..], &[0.7, 0.75], &[0.95, 0.88]] {
        for &v in row {
            ratios_builder.values().append_value(v);
        }
        ratios_builder.append(true);
    }

    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(StringArray::from(vec!["Alice", "Bob", "Charlie"])),
            Arc::new(scores_builder.finish()),
            Arc::new(tags_builder.finish()),
            Arc::new(ratios_builder.finish()),
            Arc::new(BooleanArray::from(vec![true, false, true])),
        ],
    )
    .unwrap();

    let file = std::fs::File::create("tests/fixtures/complex.parquet").unwrap();
    let mut writer = ArrowWriter::try_new(file, schema, None).unwrap();
    writer.write(&batch).unwrap();
    writer.close().unwrap();
    println!("wrote complex.parquet");
}
