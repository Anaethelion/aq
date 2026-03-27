# aq

`jq` for Apache Arrow — query and transform Parquet, Arrow IPC, CSV, and NDJSON files using jq-style expressions.

![demo](demo.gif)

```
aq [OPTIONS] [EXPR] [FILE]...
```

Each row in the file is treated as a JSON object. The expression runs on each row, just like `jq` processes NDJSON.

## Install

```bash
cargo install --path .
```

## Examples

```bash
# Print all rows as a table
aq data.parquet

# Extract a single field
aq '.first_name' data.parquet

# Filter rows
aq 'select(.salary > 50000)' data.parquet

# Project fields
aq '{name: .first_name, salary}' data.parquet

# Pipe to jq
aq -o ndjson data.parquet | jq '.first_name'

# Inspect schema
aq --schema data.parquet

# Read from stdin
cat data.arrow | aq 'select(.age > 30)'

# Count matching rows (--slurp collects all rows into an array first)
aq --slurp '[.[] | select(.salary > 50000)] | length' data.parquet

# Round-trip via Arrow IPC
aq -o arrow data.parquet | aq 'select(.still_hired)'
```

## Options

| Flag | Description |
|------|-------------|
| `-f, --format <FORMAT>` | Force input format: `arrow`, `parquet`, `csv`, `json` |
| `-o, --output <FORMAT>` | Output format: `table`, `ndjson`, `json`, `csv`, `arrow` (default: `table` on TTY, `ndjson` when piped) |
| `--schema` | Print schema and exit |
| `-s, --slurp` | Collect all rows into a JSON array before filtering — enables aggregates |
| `--no-header` | Suppress headers in table/csv output |
| `-c, --compact` | Compact JSON output |

## Expression syntax

Expressions use [jq](https://jqlang.github.io/jq/manual/) syntax. By default, the expression runs once per row (streaming mode).

```bash
# Per-row (streaming, default)
aq '.name' data.parquet                          # field access
aq 'select(.age > 30)' data.parquet              # filter
aq '{name, age}' data.parquet                    # projection
aq '.salary * 1.1' data.parquet                  # transform

# Aggregate (requires --slurp / -s)
aq -s 'length' data.parquet                      # row count
aq -s '[.[] | select(.age > 30)] | length' data.parquet
aq -s '[.[].salary] | add / length' data.parquet  # average salary
```

## Supported formats

| Format | Extensions | Notes |
|--------|-----------|-------|
| Parquet | `.parquet` | |
| Arrow IPC | `.arrow` | File and stream formats |
| CSV | `.csv` | Schema inferred from header |
| NDJSON | `.json`, `.ndjson` | One JSON object per line |

Format is auto-detected from the file extension or magic bytes. Use `-f` to override.

## License

Apache-2.0
