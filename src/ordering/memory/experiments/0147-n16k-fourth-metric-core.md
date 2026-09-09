# 0147 — Tight 4th metric core on n>16k, paid by dropping SqDiv

- **Date:** 2026-09-09
- **Base:** tip `be9bae1` (273a / 83a9250e). Local claim to beat **0.794121**, worst order() **1.105 s**. Hidden bar 0.843476.
- **Score:** not measured. The 0.794121 `yukon run` scored the shipped binary; `indep_first.rs` mtime was after `results.tsv`.
- **Status:** superseded by 0148 before a dedicated run. Do not treat 0.794121 as a 0147 result.

## Hypothesis

273a keeps `metric_k = 4` only for `n <= 16_000` and top-3 otherwise, so the
lee4_10 class (`n = 17809`) never metrics the 4th-lowest AMD core. Admitting
that core unconditionally would grow the 1.105 s path. A substitute: only when
`n > 16_000` and the 4th core's AMD total is inside **5/4** of the best
(tighter than the existing 3/2 competitive gate) and the core is inside the
metric envelope, run the 0145 census trio (DegDivNvSqrtWf, DegPlusDegme,
DegSqrt) on that core, and drop SqDiv on the existing n>16k metric cores
(0143 found no core wins for SqDiv). Task count stays 18. `n <= 16_000` is
unchanged. Second-colour stays `nnz <= 80k`. No extra set, no extra pass.

## What changed

`src/ordering/indep_first.rs` only, inside `run` phase-2 metric scheduling.

## Result

Not measured on its own binary. The completed local run at **0.794121** / fill **0.926126** is the shipped 273a ordering. 0147 was reverted before a dedicated run so the next substitute could be scored alone.

## Why it won / lost

Unmeasured. Superseded by 0148 (always swap SqDiv for DegP125 on n>16k, top-3 cores unchanged) so one yukon run would not mix two dirty trees.
