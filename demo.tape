Output demo.gif

Set FontSize 14
Set Width 900
Set Height 540
Set Theme "Dracula"
Set Shell "bash"
Set Padding 24

Hide
Type "export PATH=$PATH:$(pwd)/target/debug && clear"
Enter
Sleep 500ms
Show

# Show the raw data
Type "cat tests/fixtures/sample.csv"
Enter
Sleep 2s

# Pretty table output (default)
Type "aq tests/fixtures/sample.csv"
Enter
Sleep 2.5s

# Filter rows
Type "aq 'select(.salary > 70000)' tests/fixtures/sample.csv"
Enter
Sleep 2.5s

# Project fields
Type "aq '{name, dept}' tests/fixtures/sample.csv"
Enter
Sleep 2.5s

# Output formats
Type "aq -o ndjson tests/fixtures/sample.csv"
Enter
Sleep 2.5s

Type "aq -o json 'select(.age > 28)' tests/fixtures/sample.csv"
Enter
Sleep 2.5s

Type "aq -o csv '{name, salary}' tests/fixtures/sample.csv"
Enter
Sleep 2.5s

# Arrow IPC round-trip
Type "aq tests/fixtures/sample.arrow"
Enter
Sleep 2.5s

Type "aq 'select(.age > 28)' tests/fixtures/sample.arrow"
Enter
Sleep 2.5s

# Inspect schema
Type "aq --schema tests/fixtures/sample.arrow"
Enter
Sleep 2.5s

# Aggregate: average salary
Type "aq -s '[.[].salary] | add / length' tests/fixtures/sample.csv"
Enter
Sleep 3s
