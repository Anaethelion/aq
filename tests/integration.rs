use std::io::Write as _;
use std::process::{Command, Stdio};

fn aq() -> Command {
    Command::new(env!("CARGO_BIN_EXE_aq"))
}

// ── CSV basics ────────────────────────────────────────────────────────────────

#[test]
fn csv_table_output() {
    let out = aq().args(["tests/fixtures/sample.csv"]).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Alice"));
    assert!(stdout.contains("Engineering"));
}

#[test]
fn csv_table_contains_all_columns() {
    let out = aq().args(["tests/fixtures/sample.csv"]).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    // header row should contain column names
    assert!(stdout.contains("name"));
    assert!(stdout.contains("age"));
    assert!(stdout.contains("dept"));
    assert!(stdout.contains("salary"));
    // all four rows present
    assert!(stdout.contains("Alice"));
    assert!(stdout.contains("Bob"));
    assert!(stdout.contains("Charlie"));
    assert!(stdout.contains("Diana"));
}

#[test]
fn csv_filter_by_salary() {
    let out = aq()
        .args(["select(.salary > 70000)", "tests/fixtures/sample.csv"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Alice"));
    assert!(stdout.contains("Charlie"));
    assert!(!stdout.contains("Bob"));
    assert!(!stdout.contains("Diana"));
}

#[test]
fn csv_projection() {
    let out = aq()
        .args(["{name, dept}", "tests/fixtures/sample.csv"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Alice"));
    assert!(!stdout.contains("90000"));
}

#[test]
fn csv_output_ndjson() {
    let out = aq()
        .args(["-o", "ndjson", "tests/fixtures/sample.csv"])
        .output()
        .unwrap();
    assert!(out.status.success());
    for line in String::from_utf8(out.stdout).unwrap().lines() {
        serde_json::from_str::<serde_json::Value>(line).expect("each line should be valid JSON");
    }
}

#[test]
fn csv_output_json() {
    let out = aq()
        .args(["-o", "json", "tests/fixtures/sample.csv"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let parsed: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("output should be valid JSON");
    assert!(parsed.is_array());
    assert_eq!(parsed.as_array().unwrap().len(), 4);
}

#[test]
fn csv_output_json_compact() {
    let out = aq()
        .args(["-o", "json", "-c", "tests/fixtures/sample.csv"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8(out.stdout).unwrap().lines().count(), 1);
}

#[test]
fn csv_output_csv() {
    let out = aq()
        .args(["-o", "csv", "tests/fixtures/sample.csv"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Alice"));
}

#[test]
fn csv_output_tsv() {
    let out = aq()
        .args(["-o", "tsv", "tests/fixtures/sample.csv"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains('\t'));
    assert!(!stdout.contains(','));
    assert!(stdout.contains("Alice"));
}

#[test]
fn csv_no_header() {
    let with = aq()
        .args(["-o", "csv", "tests/fixtures/sample.csv"])
        .output()
        .unwrap();
    let without = aq()
        .args(["-o", "csv", "--no-header", "tests/fixtures/sample.csv"])
        .output()
        .unwrap();
    let with_lines = String::from_utf8(with.stdout).unwrap().lines().count();
    let without_lines = String::from_utf8(without.stdout).unwrap().lines().count();
    assert_eq!(with_lines, without_lines + 1);
}

#[test]
fn csv_slurp_count() {
    let out = aq()
        .args(["-s", "length", "tests/fixtures/sample.csv"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8(out.stdout).unwrap().trim(), "4");
}

#[test]
fn csv_schema() {
    let out = aq()
        .args(["--schema", "tests/fixtures/sample.csv"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("name"));
    assert!(stdout.contains("salary"));
}

#[test]
fn csv_custom_delimiter_semicolon() {
    // write a semicolon-delimited file and read it back
    let tmp = std::env::temp_dir().join("aq_test_semicolon.csv");
    std::fs::write(&tmp, "name;age\nAlice;30\nBob;25\n").unwrap();
    let out = aq()
        .args(["-d", ";", tmp.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Alice"));
    assert!(stdout.contains("30"));
}

#[test]
fn csv_with_missing_values() {
    let tmp = std::env::temp_dir().join("aq_test_missing.csv");
    std::fs::write(&tmp, "name,age,dept\nAlice,30,Engineering\nBob,,Marketing\n").unwrap();
    let out = aq().args([tmp.to_str().unwrap()]).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Alice"));
    assert!(stdout.contains("Bob"));
}

// ── NDJSON input ──────────────────────────────────────────────────────────────

#[test]
fn ndjson_table_output() {
    let out = aq()
        .args(["tests/fixtures/sample.ndjson"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8(out.stdout).unwrap().contains("Alice"));
}

#[test]
fn ndjson_filter() {
    let out = aq()
        .args(["select(.age > 28)", "tests/fixtures/sample.ndjson"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Alice"));
    assert!(stdout.contains("Charlie"));
    assert!(!stdout.contains("Bob"));
}

#[test]
fn ndjson_schema() {
    let out = aq()
        .args(["--schema", "tests/fixtures/sample.ndjson"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8(out.stdout).unwrap().contains("name"));
}

#[test]
fn ndjson_empty_file_returns_no_rows() {
    let tmp = std::env::temp_dir().join("aq_test_empty.ndjson");
    std::fs::write(&tmp, "").unwrap();
    let out = aq().args([tmp.to_str().unwrap()]).output().unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8(out.stdout).unwrap().trim().is_empty());
}

#[test]
fn ndjson_whitespace_only_returns_no_rows() {
    let tmp = std::env::temp_dir().join("aq_test_ws.ndjson");
    std::fs::write(&tmp, "   \n\n  ").unwrap();
    let out = aq().args([tmp.to_str().unwrap()]).output().unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8(out.stdout).unwrap().trim().is_empty());
}

// ── Arrow IPC ─────────────────────────────────────────────────────────────────

#[test]
fn arrow_schema_has_int64_types() {
    let out = aq()
        .args(["--schema", "tests/fixtures/sample.arrow"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Int64"), "expected Int64 in schema, got: {stdout}");
}

#[test]
fn arrow_numeric_filter_works() {
    let out = aq()
        .args(["select(.age > 28)", "tests/fixtures/sample.arrow"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Alice"));
    assert!(stdout.contains("Charlie"));
    assert!(!stdout.contains("Bob"));
    assert!(!stdout.contains("Diana"));
}

#[test]
fn arrow_table_output() {
    let out = aq()
        .args(["tests/fixtures/sample.arrow"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Alice"));
    assert!(stdout.contains("90000"));
}

#[test]
fn arrow_multi_batch_round_trip() {
    // Pipe two arrow files concatenated — both pass through a filter
    let data1 = std::fs::read("tests/fixtures/sample.arrow").unwrap();
    let data2 = std::fs::read("tests/fixtures/sample.arrow").unwrap();

    // Concatenate by writing both through aq to arrow then piping
    let out = aq()
        .args(["-o", "ndjson", "tests/fixtures/sample.arrow", "tests/fixtures/sample.arrow"])
        .output()
        .unwrap();
    assert!(out.status.success());
    // 4 rows × 2 files = 8 rows
    assert_eq!(String::from_utf8(out.stdout).unwrap().lines().count(), 8);
    drop((data1, data2));
}

// ── Parquet ───────────────────────────────────────────────────────────────────

#[test]
fn parquet_table_output() {
    let out = aq()
        .args(["tests/fixtures/sample.parquet"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Alice"));
    assert!(stdout.contains("90000"));
}

#[test]
fn parquet_schema() {
    let out = aq()
        .args(["--schema", "tests/fixtures/sample.parquet"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("name"));
    assert!(stdout.contains("salary"));
    assert!(stdout.contains("Int64"));
}

#[test]
fn parquet_filter_by_salary() {
    let out = aq()
        .args(["select(.salary > 70000)", "tests/fixtures/sample.parquet"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Alice"));
    assert!(stdout.contains("Charlie"));
    assert!(!stdout.contains("Bob"));
    assert!(!stdout.contains("Diana"));
}

#[test]
fn parquet_output_ndjson() {
    let out = aq()
        .args(["-o", "ndjson", "tests/fixtures/sample.parquet"])
        .output()
        .unwrap();
    assert!(out.status.success());
    for line in String::from_utf8(out.stdout).unwrap().lines() {
        serde_json::from_str::<serde_json::Value>(line).unwrap();
    }
}

#[test]
fn parquet_slurp_count() {
    let out = aq()
        .args(["-s", "length", "tests/fixtures/sample.parquet"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8(out.stdout).unwrap().trim(), "4");
}

#[test]
fn parquet_complex_list_fields() {
    let out = aq()
        .args(["tests/fixtures/complex.parquet"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Alice"));
    assert!(stdout.contains("scores") || stdout.contains("90"));
}

#[test]
fn parquet_stdin_pipe() {
    let data = std::fs::read("tests/fixtures/sample.parquet").unwrap();
    let mut child = aq()
        .args(["select(.salary > 70000)"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(&data).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Alice"));
    assert!(!stdout.contains("Bob"));
}

// ── complex types (list fields) ───────────────────────────────────────────────

#[test]
fn complex_ndjson_table_output() {
    let out = aq()
        .args(["tests/fixtures/complex.ndjson"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Alice"));
    assert!(stdout.contains("Charlie"));
}

#[test]
fn complex_arrow_schema_has_boolean_and_list_types() {
    let out = aq()
        .args(["--schema", "tests/fixtures/complex.arrow"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Boolean"), "expected Boolean in schema, got: {stdout}");
    assert!(stdout.contains("List"), "expected List types in schema, got: {stdout}");
    assert!(stdout.contains("Int64"));
    assert!(stdout.contains("Float64"));
}

#[test]
fn complex_arrow_filter_by_boolean_scalar() {
    let out = aq()
        .args(["select(.active)", "tests/fixtures/complex.arrow"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Alice"));
    assert!(stdout.contains("Charlie"));
    assert!(!stdout.contains("Bob"));
}

#[test]
fn complex_arrow_filter_by_boolean_in_list() {
    let out = aq()
        .args(["select(.flags[0])", "tests/fixtures/complex.arrow"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Alice"));
    assert!(stdout.contains("Charlie"));
    assert!(!stdout.contains("Bob"));
}

#[test]
fn complex_arrow_filter_by_list_length() {
    let out = aq()
        .args(["select((.scores | length) > 2)", "tests/fixtures/complex.arrow"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Alice"));
    assert!(stdout.contains("Charlie"));
    assert!(!stdout.contains("Bob"));
}

#[test]
fn complex_arrow_access_list_element() {
    let out = aq()
        .args([".scores[0]", "tests/fixtures/complex.arrow"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let vals: Vec<i64> = stdout.lines().map(|l| l.trim().parse().unwrap()).collect();
    assert_eq!(vals, vec![90, 70, 95]);
}

#[test]
fn complex_arrow_filter_by_tag_membership() {
    let out = aq()
        .args(["select(.tags | contains([\"eng\"]))", "tests/fixtures/complex.arrow"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Alice"));
    assert!(stdout.contains("Charlie"));
    assert!(!stdout.contains("Bob"));
}

#[test]
fn complex_arrow_project_list_field() {
    let out = aq()
        .args(["{name, scores}", "tests/fixtures/complex.arrow"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("scores"));
    assert!(!stdout.contains("tags"));
    assert!(!stdout.contains("ratios"));
}

#[test]
fn complex_ndjson_to_arrow_round_trip_preserves_lists() {
    let arrow_data: Vec<u8> = {
        let out = aq()
            .args(["-o", "arrow", "tests/fixtures/complex.ndjson"])
            .output()
            .unwrap();
        assert!(out.status.success());
        out.stdout
    };
    let mut child = aq()
        .args(["select((.scores | length) > 2)"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(&arrow_data).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Alice"));
    assert!(stdout.contains("Charlie"));
    assert!(!stdout.contains("Bob"));
}

// ── stdin piping ──────────────────────────────────────────────────────────────

#[test]
fn stdin_arrow_filter() {
    let arrow_data = std::fs::read("tests/fixtures/sample.arrow").unwrap();
    let mut child = aq()
        .args(["select(.age > 28)"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(&arrow_data).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Alice"));
    assert!(!stdout.contains("Bob"));
}

#[test]
fn stdin_ndjson_filter() {
    let data = std::fs::read("tests/fixtures/sample.ndjson").unwrap();
    let mut child = aq()
        .args(["select(.age > 28)"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(&data).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Alice"));
    assert!(!stdout.contains("Bob"));
}

#[test]
fn stdin_csv_with_format_flag() {
    let data = std::fs::read("tests/fixtures/sample.csv").unwrap();
    let mut child = aq()
        .args(["-f", "csv"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(&data).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8(out.stdout).unwrap().contains("Alice"));
}

#[test]
fn stdin_empty_ndjson_returns_no_output() {
    let mut child = aq()
        .args(["-f", "json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"").unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8(out.stdout).unwrap().trim().is_empty());
}

// ── --format override ─────────────────────────────────────────────────────────

#[test]
fn format_override_reads_csv_correctly() {
    let out = aq()
        .args(["-f", "csv", "tests/fixtures/sample.csv"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8(out.stdout).unwrap().contains("Alice"));
}

// ── multiple files ────────────────────────────────────────────────────────────

#[test]
fn multiple_files_concatenated() {
    let out = aq()
        .args([
            "-o", "ndjson",
            "tests/fixtures/sample.csv",
            "tests/fixtures/sample.csv",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8(out.stdout).unwrap().lines().count(), 8);
}

// ── new flags ─────────────────────────────────────────────────────────────────

#[test]
fn raw_output_unquotes_strings() {
    let out = aq()
        .args(["-r", ".name", "tests/fixtures/sample.csv"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Alice"));
    // names should not be JSON-quoted
    assert!(!stdout.contains("\"Alice\""));
}

#[test]
fn null_input_with_expression() {
    let out = aq()
        .args(["-n", "1 + 1"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8(out.stdout).unwrap().trim(), "2");
}

#[test]
fn null_input_with_literal() {
    let out = aq()
        .args(["-n", "[1,2,3] | length"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8(out.stdout).unwrap().trim(), "3");
}

#[test]
fn arg_binding_string() {
    let out = aq()
        .args(["--arg", "dept", "Engineering", "select(.dept == $dept)", "tests/fixtures/sample.csv"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Alice"));
    assert!(stdout.contains("Charlie"));
    assert!(!stdout.contains("Bob"));
}

#[test]
fn arg_binding_multiple() {
    let out = aq()
        .args([
            "--arg", "min", "70000",
            "--arg", "dept", "Engineering",
            "select(.dept == $dept and .salary >= ($min | tonumber))",
            "tests/fixtures/sample.csv",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Alice"));
    assert!(stdout.contains("Charlie"));
    assert!(!stdout.contains("Bob"));
    assert!(!stdout.contains("Diana"));
}

#[test]
fn version_flag() {
    let out = aq().arg("--version").output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("aq"));
}

// ── error handling ────────────────────────────────────────────────────────────

#[test]
fn invalid_jq_expression_exits_nonzero() {
    let out = aq()
        .args(["select(.(((bad", "tests/fixtures/sample.csv"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(!String::from_utf8(out.stderr).unwrap().is_empty());
}

#[test]
fn invalid_jq_error_message_is_readable() {
    let out = aq()
        .args(["select(.(((bad", "tests/fixtures/sample.csv"])
        .output()
        .unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    // should not expose raw Rust debug types like "File { code:" or "Arena"
    assert!(!stderr.contains("Arena"));
    assert!(stderr.contains("error"));
}

#[test]
fn file_not_found_exits_nonzero() {
    let out = aq().args(["nonexistent_file.csv"]).output().unwrap();
    assert!(!out.status.success());
    assert!(!String::from_utf8(out.stderr).unwrap().is_empty());
}

#[test]
fn unknown_extension_exits_nonzero() {
    let out = aq().args(["some_file.xyz"]).output().unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8(out.stderr).unwrap().contains("error"));
}
