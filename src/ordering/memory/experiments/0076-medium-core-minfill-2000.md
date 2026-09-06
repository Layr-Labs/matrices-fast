# 0076 - Medium-core residual MinFill (1000 < cn <= 2000)

- **Date:** 2026-09-06
- **Score:** 0.832118 -> 0.832049 (geomean vs AMD; lower is better)
- **Status:** win locally; submitted for official validation

## Hypothesis

After the exact degree-<=3 prefix, residual cores with `1000 < cn <= 2000`
are still small enough for exact MinFill when they are sparse. The 0075
candidate stopped at `cn <= 1000`; extending the same fixed-budget MinFill to
the next core-size band under a stricter core-nnz gate should find candidates
missed by the existing AMD/AMF grid without moving the worst-case time.

This implements autoresearch hypotheses `hyp_6d92dd0e` /
`hyp_9a23b1b1` (extend residual-core exact MinFill from 1000 to 2000 under a
strict core-nnz gate and fixed pair-check budget, strict best-of admission).

## What changed

`leader_order` in `src/ordering/mod.rs`: beside the existing `cn <= 1000`
residual MinFill, cores with `1000 < cn <= 2000` and `core_nnz <= 20_000`
run the same `minfill_order` (fixed 40M pair-check budget, `cn x cn`
membership matrix <= 4 MB), validated with `is_bijection`, scored with the
exact prefix/core split (`prefix_flops + flops_of(core)`), and admitted only
through the existing strict best-of path for `terminal_core_candidate`.
The outer gate is unchanged (`n` in 1000..10000, `nnz <= 50_000`,
`cn` in 8..4000, `core_nnz <= 30_000`), so the extension only narrows the
admission window and never widens the structural envelope. The `cn <= 1000`
path is byte-identical to 0075.

## Result

Complete trusted 300-matrix runs (`cargo run`, `yukon run`) and the
test-only timing probe agree:

| bucket | flop geomean before | flop geomean after | fill before | fill after |
|---|---|---:|---:|---:|
| `lt_1k` | 0.889764 | 0.889764 | 0.960528 | 0.960528 |
| `1k_10k` | 0.864765 | 0.864536 | 0.955872 | 0.955826 |
| `gt_10k` | 0.764398 | 0.764398 | 0.909871 | 0.909871 |

Weighted score `0.832118 -> 0.832049` (~0.7 bip); fill tiebreak
`0.938869 -> 0.938855`. The gain is confined to `1k_10k`; the other buckets
are byte-identical controls. `probe_timing_and_score` reports worst local
`order()` `0.582 s`, identical to the 0075 series and well below the `1.019 s`
revision known to have passed the grader. `cargo test -p
ssi-candidate-worker`: 66 passed, 0 failed.

## Why it won / lost

MinFill supplies a different greedy objective (minimum deficiency) on the
reduced graph, while `core_lift::splice` keeps the candidate's score exact
for the full ordering. Medium sparse cores are large enough that AMD/AMF can
leave a different local optimum, yet sparse enough (`core_nnz <= 20k`, avg
degree <= ~10-20) that the fixed-budget exact MinFill usually finishes
without hitting its fallback degree-ordered fill. Strict best-of admission
makes score risk structurally zero: a candidate that falls back or loses is
simply not installed.

Cost is funded inside the existing envelope: no new outer gate, no extra
AMF pass, one bounded MinFill per qualifying core, with the 40M pair-check
budget and the 4 MB membership matrix as hard caps. The worst-case time did
not move in this run series, but absolute timings are machine-dependent
(~1.6x run-to-run variance documented in 0003), so the comparative rule
applies: stay at or below a passing revision's worst case.

## Follow-ups

- Do not widen to `cn > 2000` or `core_nnz > 20k` without a replacement
  budget and a fresh timing probe; MinFill pair work grows with degree
  squared per pivot.
- A per-band pair-budget split (e.g. smaller budget above 1000) remains
  untested; the current change reuses the single 40M budget.
- Multi-depth prefixes K in {2,4,5} on distinct cores remain untested (see
  open-questions).

## Links

- Techniques: [best-of-portfolio](../techniques/best-of-portfolio.md)
- Prior: [0075-residual-core-minfill](0075-residual-core-minfill.md)
- Research queue: [open-questions](../open-questions.md)
