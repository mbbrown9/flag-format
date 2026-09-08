# flagfmt

A command-line tool for feature flag definitions: a validating parser plus a
canonical pretty-printer, for a small text format that's meant to be easier
to hand-edit and diff-review than JSON or YAML.

## Why

Feature flags tend to end up in a JSON or YAML blob somewhere in the repo.
That works, but it's not great to hand-edit: no comments, inconsistent key
order across editors, and a diff can look huge just because someone's editor
reindented the file. `flagfmt` defines a small format for describing flags,
validates it (types, required fields, ranges, duplicate names), and
reformats it to a single canonical layout so diffs stay small and reviewable.
It can also emit the same validated data as JSON with `--json`, for whatever
downstream tooling wants to consume it.

## The format

```
# payments team
flag payments.new-checkout {
    enabled = true
    description = "Switch to the new checkout pipeline"
    rollout = 25%
    tags = [payments, checkout, beta]
}

flag search.vector-index {
    enabled = false
    description = "Use vector similarity search instead of keyword match"
}
```

- `#` starts a line comment.
- A flag name starts with a letter and may contain letters, digits, `.`,
  `_` or `-`. Flag names must be unique within a file.
- `enabled` (required): `true` or `false`.
- `description` (optional): a double-quoted string. Supports `\"`, `\\`,
  `\n` and `\t` escapes.
- `rollout` (optional): an integer percentage from `0%` to `100%`.
- `tags` (optional): a comma-separated list of bare identifiers in `[...]`.
  Duplicates within a flag are rejected.

Unknown fields, malformed values, and out-of-range rollouts are all parse
errors, not silently ignored.

## Usage

```
flagfmt flags.txt
```

Parses and validates `flags.txt`, then writes the canonical form to stdout:
flags sorted by name, tags sorted, four-space indentation, one blank line
between flags.

```
flagfmt --json flags.txt
```

Same parse and validation, but writes the flags as JSON instead:

```json
{
  "flags": [
    {
      "name": "payments.new-checkout",
      "enabled": true,
      "description": "Switch to the new checkout pipeline",
      "rollout": 25,
      "tags": ["beta", "checkout", "payments"]
    },
    {
      "name": "search.vector-index",
      "enabled": false,
      "description": "Use vector similarity search instead of keyword match",
      "rollout": null,
      "tags": []
    }
  ]
}
```

On a validation error, `flagfmt` reports the file, line and column and exits
with a non-zero status instead of printing anything:

```
$ flagfmt bad.flags
error: bad.flags:3:15: rollout must be between 0 and 100, got 150
```

## Building

Standard library only, no external crates:

```
cargo build --release
```

## Status

Early skeleton. The parser, validator and both printers work end to end for
the format described above. Not yet done, roughly in order: friendlier
parse error messages (currently token-shaped, not always human-shaped),
reading from stdin, a `--check` mode that validates without printing, and
a way to diff two flag files.

## License

MIT, see [LICENSE](LICENSE).
