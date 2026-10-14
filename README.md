# doc-router

> Your agent spends most of its context reading documentation it doesn't use.
> doc-router tells it what matters, and when to skip reading entirely.

## The problem

- Agents load AGENTS.md, README.md, and docs/ wholesale into context.
- Most of those tokens are irrelevant to the task at hand.
- 97% of llms.txt files are never read.
- Passive indexes fail. Active retrieval is unreliable.

## What doc-router does

Given a task and a project directory, doc-router returns the minimal set
of documentation sections the agent should read, with reasons, token
estimates, and an explicit decision to skip when nothing is relevant.

It is not a documentation format. It is a decision to not read documentation.

## Install

    cargo install --git https://github.com/YOUR_USER/doc-router

## Quick start

    doc-router route --task "add rate limiting to the payments endpoint" --format human

Output:

    Task: add rate limiting to the payments endpoint
    Decision: Read
    Total document tokens: 12400
    Navigated tokens: 520
    Savings: 95.8%

## Use via MCP

Add to your MCP config (Claude Code, Cursor, etc.):

    {
      "mcpServers": {
        "doc-router": {
          "command": "doc-router",
          "args": ["mcp", "--root", "/path/to/project"]
        }
      }
    }

Tools exposed: doc_route(task), doc_expand(section_id).

## Benchmark

Corpus: real documentation trees from three open-source projects
(cloned via sparse checkout, docs/ only). 20 task descriptions per repo.

| Repository | Doc tokens | Median selected | Range | Skips |
|---|---|---|---|---|
| fastapi | 2,960,655 | 787 | 0-798 | 3/20 |
| httpx | 47,567 | 693 | 0-796 | 10/20 |
| next.js | 856,536 | 794 | 0-800 | 2/20 |

Key finding: the router decides to skip documentation entirely in
15 of 60 tasks. When it does read, it selects ~700 tokens on average,
regardless of whether the corpus is 47K or 3M tokens.

### Reproduce

    bash scripts/bench.sh
    bash scripts/each-repo.sh

## Design principles

1. Section-first. Units of retrieval are headings, not files.
2. Passive by default. The router runs once, cheaply.
3. Rejection-friendly. Empty output is valid and common.
4. Token-aware. Every recommendation carries a cost.
5. Measured, not guessed. Every claim has a benchmark.

## Roadmap

- [x] v0.1 - keyword router + CLI + MCP server
- [ ] v0.2 - local embeddings (via fastembed)
- [ ] v0.3 - adaptive thresholds based on task feedback

## License

MIT
