# Quickstart

This page follows one input through the three most common interfaces: the
command line, Rust and Python. Each produces the same value.

The input is a small settings file, `settings.conf`, that a person typed by
hand:

```text
// cache settings
name: 'cache'
ports: [8080, 8081,]
enabled: true
```

It has a comment, no outer braces, unquoted keys, a single-quoted string, a
trailing comma and newlines instead of commas. A strict JSON parser rejects it
at the first character.

## On the command line

Prerequisite: the `vexy-json` binary is installed (see
[Installation](installation.md)).

1. Parse the file:

    ```bash
    vexy-json settings.conf
    ```

    The result is pretty-printed JSON on standard output:

    ```json
    {
      "name": "cache",
      "ports": [
        8080,
        8081
      ],
      "enabled": true
    }
    ```

2. Get the same value on one line with `--compact`:

    ```bash
    vexy-json --compact settings.conf
    ```

3. Read from standard input instead of a file by leaving out the file name:

    ```bash
    printf "{a: 1, b: [2, 3,],}" | vexy-json -c
    ```

    ```json
    {"a":1,"b":[2,3]}
    ```

Object keys are stored in a hash map, so the order of keys in the output can
differ from the order in the input.

## In Rust

Prerequisite: `vexy-json` is a dependency of your crate.

```rust
use vexy_json::parse;

fn main() -> vexy_json::Result<()> {
    let input = "// cache settings\nname: 'cache'\nports: [8080, 8081,]\nenabled: true";
    let value = parse(input)?;

    // Index objects with &str and arrays with usize.
    println!("{}", value["name"]);          // "cache"
    println!("{}", value["ports"][1]);      // 8081
    println!("{:?}", value["enabled"].as_bool()); // Some(true)
    Ok(())
}
```

`parse` returns `vexy_json::Result<Value>`. A missing key or index returns
`Value::Null` instead of panicking. See [Rust API](rust-api.md).

## In Python

Prerequisite: `vexy-json` is installed in your Python environment.

```python
import vexy_json

text = "// cache settings\nname: 'cache'\nports: [8080, 8081,]\nenabled: true"
value = vexy_json.parse(text)

assert value == {"name": "cache", "ports": [8080, 8081], "enabled": True}
```

`vexy_json.parse` (also available as `vexy_json.loads`) returns plain Python
`dict`, `list`, `str`, `int`, `float`, `bool` and `None` objects. Invalid input
raises `ValueError`. See [Python](python.md).

## When input is rejected

Input that the parser cannot read raises an error with a byte position:

```bash
printf "{a: 1" | vexy-json
```

```text
Error:
<stdin>: Expected , or } or newline but found Eof at position 5
```

The command exits with status `1`. For input that was cut off in the middle,
such as a truncated response from a language model, see
[Repairing incomplete input](repair.md).

## Next steps

- [Accepted syntax](syntax.md) lists every extension with an example.
- [Parser options](options.md) explains how to turn extensions off.
- [Command line](cli.md) documents every option of `vexy-json`.
