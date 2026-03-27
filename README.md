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

## Limitations

### Arrow → jq type mapping

Every Arrow column is converted to JSON before jq sees it:

| Arrow type | jq representation |
|---|---|
| `Int8/16/32/64`, `UInt8/16/32/64` | number |
| `Float16/32/64` | number |
| `Decimal128/256` | number or string (precision-dependent) |
| `Date32/64`, `Timestamp`, `Time*`, `Duration`, `Interval` | integer (epoch in column units) |
| `Utf8`, `LargeUtf8` | string |
| `Binary`, `LargeBinary` | base64 string |
| `Boolean` | boolean |
| `List<T>`, `LargeList<T>`, `FixedSizeList<T>` | array |
| `Struct` | object |
| `Map<K,V>` | array of `{key, value}` objects |
| `Dictionary<K,V>` | decoded to the dictionary value type |

### jq → Arrow type mapping (`-o arrow`)

When writing Arrow output, types are re-inferred from jq output values. Only a subset of Arrow types can be produced:

| jq output | Arrow type |
|---|---|
| integer | `Int64` |
| float | `Float64` |
| boolean | `Boolean` |
| string | `Utf8` |
| uniform integer array | `List<Int64>` |
| uniform float array | `List<Float64>` |
| uniform boolean array | `List<Boolean>` |
| uniform string array | `List<Utf8>` |
| mixed-type array | `List<Utf8>` (elements serialized) |
| object | `Utf8` (serialized as JSON string) |
| all-null column | `Utf8` |

Types not expressible in JSON (timestamps, decimals, binary, structs…) cannot be round-tripped through `-o arrow` — they arrive as integers or strings and leave as `Int64` or `Utf8`.

### Precision loss

- **Large integers**: jq uses IEEE 754 float64 internally, so `Int64` values beyond ±2^53 (~9 × 10^15) lose precision in expressions. Arithmetic and equality checks on such values may silently give wrong results.
- **Integer width**: `Int8/16/32` and `UInt8/16/32` widen to `Int64` on `-o arrow` output.
- **Float width**: `Float16/32` widen to `Float64`.
- **UInt64**: Values above 2^53 lose precision when passed through jq.

### Column ordering in Arrow output

`serde_json` stores object keys in alphabetical order. Projection expressions like `{name, age}` produce `{age, name}` in the Arrow output — alphabetically sorted, not in expression order.

### jaq vs jq compatibility

`aq` uses [jaq](https://github.com/01mf02/jaq) rather than jq. Most everyday programs work, but some features are absent or differ:

- **Not supported**: `$ENV`, `env`, `input`/`inputs`, `$__loc__`, `modulemeta`
- **`debug`**: output format differs from jq
- **`path` builtins**: `path()`, `getpath`, `setpath`, `delpaths` may behave differently
- **`?//` (alternative operator)**: semantics may differ from jq
- **`@format` strings**: `@base64`, `@uri`, `@csv`, `@tsv`, `@html`, `@json` are supported; `@base64d` requires valid padding
- **Multiple files**: use `aq 'expr' a.json b.json` instead of `jq -n '[inputs]'`

## License

Apache-2.0
