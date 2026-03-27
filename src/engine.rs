use jaq_core::load::{Arena, File, Loader};
use jaq_core::{Ctx, Vars};
use jaq_json::Val;
use serde_json::Value;

use crate::error::{ArrowCliError, Result};

pub struct Filter {
    filter: jaq_core::compile::Filter<jaq_core::Native<jaq_core::data::JustLut<Val>>>,
}

/// Compile a jq expression into a reusable filter.
pub fn compile(expr: &str) -> Result<Filter> {
    let defs = jaq_core::defs()
        .chain(jaq_std::defs())
        .chain(jaq_json::defs());

    let funs = jaq_core::funs::<jaq_core::data::JustLut<Val>>()
        .chain(jaq_std::funs())
        .chain(jaq_json::funs());

    let program = File {
        code: expr,
        path: (),
    };
    let loader = Loader::new(defs);
    let arena = Arena::default();

    let modules = loader.load(&arena, program).map_err(|errs| {
        let msg = errs
            .iter()
            .map(|(file, _)| format!("cannot parse `{}`", file.code))
            .collect::<Vec<_>>()
            .join("; ");
        ArrowCliError::JqParse(msg)
    })?;

    let filter = jaq_core::Compiler::default()
        .with_funs(funs)
        .compile(modules)
        .map_err(|errs| {
            let msg = errs
                .iter()
                .map(|(file, _)| format!("undefined in `{}`", file.code))
                .collect::<Vec<_>>()
                .join("; ");
            ArrowCliError::JqParse(msg)
        })?;

    Ok(Filter { filter })
}

/// Run a compiled jq filter over rows in streaming mode (each row is a separate input,
/// like real jq processing NDJSON). Use this for per-row expressions like `.field`,
/// `select(.age > 30)`, etc.
pub fn run(filter: &Filter, rows: Vec<Value>) -> Result<Vec<Value>> {
    let mut results = Vec::new();
    for row in rows {
        run_one(filter, row, &mut results)?;
    }
    Ok(results)
}

/// Run a compiled jq filter in slurp mode: all rows are wrapped in a JSON array and
/// the filter runs once on that array. Use this for aggregate operations like
/// `[.[] | select(.age > 30)] | length`.
pub fn run_slurp(filter: &Filter, rows: Vec<Value>) -> Result<Vec<Value>> {
    let mut results = Vec::new();
    run_one(filter, Value::Array(rows), &mut results)?;
    Ok(results)
}

/// Extract a readable message from a jaq-core Exn debug string.
fn format_exn(debug: &str) -> String {
    // Most common: Exn(Err(Error(Str([Str("message")]))))
    if let Some(rest) = debug.strip_prefix("Exn(Err(Error(Str([Str(\"") {
        if let Some(msg) = rest.strip_suffix("\")]))))") {
            return msg.to_string();
        }
    }
    // Fallback: strip outer Exn(Err(Error(...))) wrapper for readability
    if let Some(inner) = debug
        .strip_prefix("Exn(Err(Error(")
        .and_then(|s| s.strip_suffix("))))"))
    {
        return inner.to_string();
    }
    debug.to_string()
}

fn run_one(filter: &Filter, input: Value, out: &mut Vec<Value>) -> Result<()> {
    let json_bytes = serde_json::to_vec(&input)?;
    let val: Val = jaq_json::read::parse_single(&json_bytes)
        .map_err(|e| ArrowCliError::JqRuntime(format!("input parse error: {e:?}")))?;

    let ctx = Ctx::<jaq_core::data::JustLut<Val>>::new(&filter.filter.lut, Vars::new([]));

    for output in filter.filter.id.run((ctx, val)) {
        match output {
            Ok(v) => {
                let json_str = format!("{v}");
                let serde_val: Value = serde_json::from_str(&json_str)?;
                out.push(serde_val);
            }
            Err(e) => return Err(ArrowCliError::JqRuntime(format_exn(&format!("{e:?}")))),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn identity_pass_through() {
        let f = compile(".").unwrap();
        let rows = vec![json!({"a": 1}), json!({"b": 2})];
        let out = run(&f, rows).unwrap();
        assert_eq!(out, vec![json!({"a": 1}), json!({"b": 2})]);
    }

    #[test]
    fn field_extraction() {
        let f = compile(".name").unwrap();
        let rows = vec![json!({"name": "Alice", "age": 30})];
        let out = run(&f, rows).unwrap();
        assert_eq!(out, vec![json!("Alice")]);
    }

    #[test]
    fn select_keeps_matching_rows() {
        let f = compile("select(.age > 28)").unwrap();
        let rows = vec![
            json!({"name": "Alice", "age": 30}),
            json!({"name": "Bob",   "age": 25}),
        ];
        let out = run(&f, rows).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0]["name"], "Alice");
    }

    #[test]
    fn select_drops_all_when_none_match() {
        let f = compile("select(.age > 100)").unwrap();
        let rows = vec![json!({"age": 30}), json!({"age": 25})];
        assert!(run(&f, rows).unwrap().is_empty());
    }

    #[test]
    fn projection() {
        let f = compile("{name, age}").unwrap();
        let rows = vec![json!({"name": "Alice", "age": 30, "salary": 90000})];
        let out = run(&f, rows).unwrap();
        assert_eq!(out, vec![json!({"name": "Alice", "age": 30})]);
    }

    #[test]
    fn arithmetic_transform() {
        let f = compile(".salary * 2").unwrap();
        let rows = vec![json!({"salary": 50000})];
        let out = run(&f, rows).unwrap();
        assert_eq!(out, vec![json!(100000)]);
    }

    #[test]
    fn slurp_length() {
        let f = compile("length").unwrap();
        let rows = vec![json!({"a": 1}), json!({"b": 2}), json!({"c": 3})];
        let out = run_slurp(&f, rows).unwrap();
        assert_eq!(out, vec![json!(3)]);
    }

    #[test]
    fn slurp_aggregate_sum() {
        let f = compile("[.[].salary] | add").unwrap();
        let rows = vec![json!({"salary": 90000}), json!({"salary": 65000})];
        let out = run_slurp(&f, rows).unwrap();
        assert_eq!(out, vec![json!(155000)]);
    }

    #[test]
    fn invalid_expression_errors() {
        assert!(compile("select(.).bad syntax |||").is_err());
    }

    #[test]
    fn runtime_error_propagates() {
        let f = compile(".foo | .bar | error").unwrap();
        let rows = vec![json!({"foo": {"bar": "boom"}})];
        assert!(run(&f, rows).is_err());
    }
}
