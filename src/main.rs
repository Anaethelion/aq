use clap::Parser;
use std::path::Path;

mod cli;
mod convert;
mod detect;
mod engine;
mod error;
mod output;
mod reader;

use cli::Args;
use error::Result;

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut args = Args::parse();

    // No args + interactive stdin → show help instead of hanging.
    if args.expr.is_none()
        && args.files.is_empty()
        && !args.null_input
        && !args.schema
        && is_terminal::IsTerminal::is_terminal(&std::io::stdin())
    {
        let mut cmd = <Args as clap::CommandFactory>::command();
        cmd.print_help()?;
        println!();
        return Ok(());
    }

    let delimiter: Option<u8> = args.delimiter.map(|c| c as u8);

    // Disambiguate: if the first positional arg looks like a file
    // (existing path or recognized extension), treat it as a FILE not an EXPR.
    if !args.null_input {
        if let Some(ref expr_str) = args.expr {
            let p = Path::new(expr_str.as_str());
            if p.exists()
                || matches!(
                    p.extension().and_then(|e| e.to_str()),
                    Some("parquet" | "arrow" | "csv" | "json" | "ndjson")
                )
            {
                let path = std::path::PathBuf::from(args.expr.take().unwrap());
                args.files.insert(0, path);
            }
        }
    }

    // Read input into Arrow record batches (skipped for --null-input)
    let rows = if args.null_input {
        vec![serde_json::Value::Null]
    } else {
        let batches = if args.files.is_empty() {
            reader::read_stdin(args.format.as_ref(), delimiter)?
        } else {
            let mut all = Vec::new();
            for path in &args.files {
                let batches = reader::read_file(path, args.format.as_ref(), delimiter)?;
                all.push(batches);
            }
            reader::concat_batches(all)?
        };

        // --schema: print schema and exit
        if args.schema {
            if let Some(batch) = batches.first() {
                println!("{}", batch.schema());
            } else {
                eprintln!("(empty — no record batches)");
            }
            return Ok(());
        }

        convert::batches_to_json_rows(&batches)?
    };

    // Build expression, prepending any --arg variable bindings.
    let base_expr = args
        .expr
        .as_deref()
        .unwrap_or(if args.slurp { ".[]" } else { "." });

    let expr: String = if args.arg.is_empty() {
        base_expr.to_string()
    } else {
        let mut full = String::new();
        for chunk in args.arg.chunks(2) {
            let name = &chunk[0];
            let value = chunk[1].replace('\\', "\\\\").replace('"', "\\\"");
            full.push_str(&format!("\"{}\" as ${} | ", value, name));
        }
        full.push_str(base_expr);
        full
    };

    // Compile and run jq expression
    let filter = engine::compile(&expr)?;
    let results = if args.slurp {
        engine::run_slurp(&filter, rows)?
    } else {
        engine::run(&filter, rows)?
    };

    // Output
    let fmt = args.output.clone().unwrap_or_else(output::default_format);
    let opts = output::RenderOptions {
        no_header: args.no_header,
        compact: args.compact,
        raw: args.raw,
        delimiter: delimiter.unwrap_or(b','),
    };
    output::render(&results, &fmt, &opts)?;

    Ok(())
}
