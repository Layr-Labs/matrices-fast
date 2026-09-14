# 0162 — Kernelized reduce (OSS simplicial/almost-simplicial): measured NEGATIVE as an ordering variant

- **Date:** 2026-09-13
- **Score:** 0.790135 → **0.790135** — **ZERO of 300 rows moved** by the
  kernel candidate; reverted (instrument kept).
- **Status:** negative, closed with evidence. The scout's Rank-1 estimate
  (1–3e-3) assumed the 0062 deg≤3 precedent transfers; it does not, and the
  record now says why.

## What was built

`core_lift::reduce_kernel`: the K=3 rule extended with the Ost/Schulz/Strasser
(ALENEX 2021, arXiv:2004.11315) fill-bounded rules to a fixpoint under one
40M pair-check ledger — **simplicial** (live neighbourhood already a clique:
zero fill, any degree) and **almost-simplicial** (exactly one missing
neighbour pair: one fill edge, degree ≤ 12). Every step is the same exact
elimination `reduce_impl` performs (close the neighbourhood, charge
`c_v = |live|+1`), so the prefix term is exact and splice accounting is
unchanged. Wired as an additional core attempt in the stage-9 block (same
bands as the extra depths, ≥10 %-smaller freshness rule, exact strict
acceptance) — zero score risk by construction.

## The census signal was real (probe_kernel_census)

- **Chordal collapses to a 1-vertex core** (the fixpoint is a perfect
  elimination): catmix400 (n 2001), polygon75 (5846), meanvar-orl400_05_e_7
  (5205), pedigree_sim400, maxmin, st_e41, nvs02/19, himmel11, the autocorr /
  cvxnonsep_psig families.
- **Partial shrinks**: sfacloc2_3_80 −86 %, sfacloc2_4_80 −81 %, qspp −24-26 %,
  glider400 −21 %, transswitch0300p −10 %, powerflow0300p −9 %.
- Worst kernel call on the corpus: 0.06 s (meanvar, 175k nnz) — work-removing,
  bounded, cap-safe.

## Why it moved nothing (the lesson)

1. **The chordal rows are the unwinnable AMD ties.** On a chordal graph every
   PEO is fill-free, so AMD's own ordering already achieves zero fill and the
   row sits at ratio exactly 1.0000 — the same forced-optimality argument as
   0151's squfl/emfl closure. Collapsing the graph perfectly does not beat a
   perfect incumbent.
2. **The kernel prefix is a min-degree-class ordering.** Ascending-(degree,
   index) elimination is the objective the portfolio's AMD/AMF machinery
   already dominates; on the partial-shrink rows the spliced candidate lost
   to the polished incumbent every time.
3. The literature's use of these rules is **instance reduction for exact
   solvers** (they beat METIS on road networks by shrinking the instance the
   exact search then solves) — not heuristic ordering quality per se. A value
   path here would need the small kernel core fed to the exact search
   machinery (subtree/exact windows on the CORE graph), which is a different,
   larger experiment.

## Files

- `core_lift.rs`: `reduce_kernel` / `reduce_kernel_impl` (KEPT — instrument).
- `probe.rs`: `probe_kernel_census` (KEPT — test-only).
- `mod.rs`: stage-9 wiring REMOVED with the receipt comment above.

## Reproduction

```sh
cargo test --release -p ssi-candidate-worker --offline --locked -- \
  --ignored --nocapture --test-threads=1 probe_kernel_census
git revert-ready state: commit 37332f3 (wired, score-identical 0.790135) →
reverted on top (this record's commit).
```
