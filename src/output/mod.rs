use std::io::{self, Write};

use is_terminal::IsTerminal;
use serde_json::Value;

use crate::cli::OutputFormat;
use crate::error::Result;

pub mod arrow_out;
pub mod csv_out;
pub mod json;
pub mod table;

pub fn default_format() -> OutputFormat {
    if io::stdout().is_terminal() {
        OutputFormat::Table
    } else {
        OutputFormat::Ndjson
    }
}

pub struct RenderOptions {
    pub no_header: bool,
    pub compact: bool,
    pub raw: bool,
    pub delimiter: u8,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            no_header: false,
            compact: false,
            raw: false,
            delimiter: b',',
        }
    }
}

pub fn render(values: &[Value], format: &OutputFormat, opts: &RenderOptions) -> Result<()> {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    render_to(&mut out, values, format, opts)
}

pub fn render_to<W: Write>(
    out: &mut W,
    values: &[Value],
    format: &OutputFormat,
    opts: &RenderOptions,
) -> Result<()> {
    match format {
        OutputFormat::Table => {
            let rendered = table::render(values, opts.no_header);
            if !rendered.is_empty() {
                writeln!(out, "{}", rendered)?;
            }
        }
        OutputFormat::Ndjson => {
            if opts.raw {
                json::write_raw(out, values)?;
            } else {
                json::write_ndjson(out, values)?;
            }
        }
        OutputFormat::Json => json::write_pretty(out, values, opts.compact)?,
        OutputFormat::Csv => csv_out::write(out, values, opts.no_header, opts.delimiter)?,
        OutputFormat::Tsv => csv_out::write(out, values, opts.no_header, b'\t')?,
        OutputFormat::Arrow => arrow_out::write(out, values)?,
    }
    Ok(())
}
