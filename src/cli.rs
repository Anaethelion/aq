use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(
    name = "aq",
    version,
    about = "jq for Apache Arrow — query Parquet, Arrow IPC, CSV, and NDJSON files",
    long_about = None,
)]
pub struct Args {
    /// jq expression (default: `.` — pass each row through)
    #[arg(value_name = "EXPR")]
    pub expr: Option<String>,

    /// Input files (reads stdin if omitted)
    #[arg(value_name = "FILE")]
    pub files: Vec<std::path::PathBuf>,

    /// Force input format
    #[arg(short, long, value_name = "FORMAT")]
    pub format: Option<InputFormat>,

    /// Print schema and exit
    #[arg(long)]
    pub schema: bool,

    /// Output format (default: table when TTY, ndjson when piped)
    #[arg(short, long = "output", value_name = "FORMAT")]
    pub output: Option<OutputFormat>,

    /// Suppress column headers in table/csv/tsv output
    #[arg(long)]
    pub no_header: bool,

    /// Compact output (no pretty-printing for JSON)
    #[arg(short, long)]
    pub compact: bool,

    /// Output raw strings without JSON quoting (strings only)
    #[arg(short, long)]
    pub raw: bool,

    /// Slurp all rows into a JSON array before applying the expression.
    /// Enables aggregate operations like `[.[] | select(.age > 30)] | length`.
    #[arg(short, long)]
    pub slurp: bool,

    /// Use null as input instead of reading any file
    #[arg(short = 'n', long)]
    pub null_input: bool,

    /// Bind a string variable: --arg NAME VALUE makes $NAME available in the expression
    #[arg(long, value_names = ["NAME", "VALUE"], num_args = 2, action = clap::ArgAction::Append)]
    pub arg: Vec<String>,

    /// Field delimiter for CSV input/output (default: ',')
    #[arg(short, long, value_name = "CHAR")]
    pub delimiter: Option<char>,
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum InputFormat {
    Arrow,
    Parquet,
    Csv,
    Json,
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum OutputFormat {
    Table,
    Json,
    Ndjson,
    Csv,
    Tsv,
    Arrow,
}
