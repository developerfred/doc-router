# Contributing

Thanks for considering contributing to `doc-router`.

## The one rule

**Every PR must include a benchmark run.** We measure, we don't guess.

If your change affects routing decisions, include:
- A `tasks.jsonl` addition (if you add a new task type)
- Output of `doc-router bench --format json` before and after

## Good first issues

- Add a new documentation format to the scanner (`.rst`, `.adoc`)
- Add a scoring heuristic for code blocks in Markdown
- Add a `--max-tokens` flag to `route`

## Running locally

```bash
cargo build --release
./target/release/doc-router route --task "add rate limiting" --format human
./target/release/doc-router bench --corpus ./corpus --tasks ./tasks.jsonl

