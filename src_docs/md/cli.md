# Command line

The `vexy-json` program reads relaxed JSON from files or standard input and
writes standard JSON. It is built from the `vexy-json-cli` crate.

```text
vexy-json [OPTIONS] [FILE]...
```

With no `FILE`, the program reads standard input. With several files, it
processes them one after another (or in parallel with `--parallel`) and writes
each result in turn.

## How a run works

For each input, `vexy-json`:

1. Reads the bytes and checks that they are valid UTF-8. Invalid UTF-8 is an
   error that names the byte where decoding stopped.
2. Parses the text. It first tries a strict JSON parser (`serde_json`) and,
   if that fails, the forgiving Vexy JSON parser with the options from the
   command line.
3. On success, formats the value and writes it to standard output or to the
   `--output` file.
4. On failure, prints `Error:` and the message to standard error and exits
   with status `1`. With `--repair`, it first tries to close truncated input
   (see [Repairing incomplete input](repair.md)).

## Output options

| Option | Effect |
| --- | --- |
| `-o`, `--output <FILE>` | Write the result to `FILE` instead of standard output |
| `-p`, `--pretty` | Indent the output |
| `--indent <INDENT>` | Spaces per indentation level (default `2`) |
| `-c`, `--compact` | Write everything on one line with no extra whitespace |
| `-v`, `--validate` | Check the input and print `Valid JSON` instead of the value |

Without `--pretty` or `--compact`, objects and arrays are indented and
scalars (strings, numbers, booleans, `null`) are written on one line. No
newline is added after the output.

With `--validate` and more than one file (or in watch mode), each line names
the file: `settings.conf is valid JSON`.

## Syntax options

Each flag switches off one extension. They map to the
[parser options](options.md) of the same name.

| Flag | Disables |
| --- | --- |
| `--no-comments` | `//`, `#` and `/* … */` comments |
| `--no-trailing-commas` | trailing commas |
| `--no-unquoted-keys` | unquoted object keys |
| `--no-single-quotes` | single-quoted strings |
| `--no-implicit-top-level` | objects and arrays without outer braces or brackets |
| `--no-newline-as-comma` | newlines as separators |
| `--max-depth <MAX_DEPTH>` | maximum nesting depth (default `128`) |

## Input modes

| Option | Effect |
| --- | --- |
| `--ndjson` | Treat the input as newline-delimited JSON: one value per line |
| `--parallel-parse` | Parse one large input with the parallel parser |
| `--chunk-size <CHUNK_SIZE>` | Minimum chunk size in bytes for parallel parsing (default `65536`) |
| `--max-threads <MAX_THREADS>` | Thread limit for parallel parsing; `0` means automatic (default `0`) |
| `-j`, `--parallel` | Process several input files at the same time |
| `-w`, `--watch` | Process the files, then reprocess each one when it changes |
| `--lms` | Run `lms ls --json --llm` and parse its output |

`--chunk-size` and `--max-threads` apply to both `--parallel-parse` and
`--ndjson`.

In `--ndjson` mode each value is written separately. With `--pretty`, values
are separated by a blank line.

`--watch` needs at least one file. It prints
`Watching N files for changes... (Press Ctrl+C to stop)` and keeps running
until you stop it.

`--lms` is for the LM Studio command-line tool. It runs
`lms ls --json --llm`, captures the complete output in a temporary file
(256 MiB at most) and parses it. It cannot be combined with file arguments,
`--watch`, `--parallel`, `--repair`, `--parallel-parse` or `--ndjson`.

## Error and repair options

| Option | Effect |
| --- | --- |
| `--verbose-errors` | Report the line and column of a parse error and show the lines around it |
| `-r`, `--repair` | Close input that ends inside open objects or arrays, and warn about it |
| `--repair-details` | Requires `--repair`; the repair report is the same as with `--repair` alone |
| `--fallback` | Accepted for compatibility; the strict-then-forgiving fallback always runs |

`--repair` cannot be combined with `--watch`, `--ndjson` or `--parallel-parse`.

## Examples

Parse a file and pretty-print it with four-space indentation:

```bash
vexy-json --indent 4 config.json5
```

Write the normalized, compact result to a new file:

```bash
vexy-json -c settings.conf -o settings.json
```

Check several files at once:

```bash
vexy-json -v a.conf b.conf
```

```text
a.conf is valid JSON
b.conf is valid JSON
```

Find the line of a syntax error:

```bash
vexy-json --verbose-errors bad.conf
```

```text
Error:
Parse error in file 'bad.conf' at line 4, column 9: Expected , or ] or newline but found Some((Colon, Span { start: 36, end: 37 })) at position 37

   3     list: [1, 2
   4 >   other: 3
              ^
   5   }
```

Without `--verbose-errors` the message is one line:
`bad.conf: Expected , or ] or newline … at position 37`.

Reject anything that is not strict JSON:

```bash
vexy-json --no-comments --no-trailing-commas --no-unquoted-keys \
  --no-single-quotes --no-implicit-top-level --no-newline-as-comma data.json
```

## Exit status

| Status | Meaning |
| --- | --- |
| `0` | All inputs parsed (or were empty) |
| `1` | A file was missing, the input was not UTF-8, parsing failed or a repair was unsafe |
| `2` | Invalid command-line arguments, such as `--repair` together with `--ndjson` |

## Other options

| Option | Effect |
| --- | --- |
| `-h`, `--help` | Print help; `-h` prints a summary |
| `-V`, `--version` | Print the version |
