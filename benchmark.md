## Benchmark

Corpus: real documentation trees from three open-source projects,
cloned via sparse checkout (`docs/` only). 20 task descriptions per repo.

### Per-repository results

| Repository | Doc tokens | Median selected | Range | Median savings | Skips |
|---|---|---|---|---|---|

### How to reproduce

```bash
bash scripts/bench.sh
bash scripts/bench-final.sh
