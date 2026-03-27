Output demo.gif

Set FontSize 14
Set Width 900
Set Height 600
Set Theme "Dracula"
Set Shell "bash"
Set Padding 24

Hide
Type "export PATH=$PATH:$(pwd)/target/debug && clear"
Enter
Sleep 500ms
Show

# Pretty table output (default)
Type "aq tests/fixtures/sample.csv"
Enter
Sleep 2.5s

# Filter rows
Type "aq 'select(.salary > 70000)' tests/fixtures/sample.csv"
Enter
Sleep 2.5s

# Raw string output with -r
Type "aq -r '.name' tests/fixtures/sample.csv"
Enter
Sleep 2.5s

# TSV output
Type "aq -o tsv '{name, salary}' tests/fixtures/sample.csv"
Enter
Sleep 2.5s

# Variable binding with --arg
Type "aq --arg threshold 80000 'select(.salary > ($threshold | tonumber))' tests/fixtures/sample.csv"
Enter
Sleep 2.5s

# Null input mode
Type "aq -n '[range(5)] | add'"
Enter
Sleep 2s

# Parquet input
Type "aq tests/fixtures/sample.parquet"
Enter
Sleep 2.5s

# Inspect schema (shows correct Int64 types)
Type "aq --schema tests/fixtures/sample.parquet"
Enter
Sleep 2.5s

# Complex types: filter by list length
Type "aq 'select((.scores | length) > 2)' tests/fixtures/complex.parquet"
Enter
Sleep 3s

# Arrow IPC with numeric filter
Type "aq 'select(.age > 28)' tests/fixtures/sample.arrow"
Enter
Sleep 2.5s

# Aggregate: average salary
Type "aq -s '[.[].salary] | add / length' tests/fixtures/sample.csv"
Enter
Sleep 3s
