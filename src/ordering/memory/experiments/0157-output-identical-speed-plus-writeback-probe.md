# 0157 — Output-identical speed work (custom-metric kernel) + best_flops write-back, submitted as a cap probe

**Base:** promoted `ab30c0e` (submission `a9905f2`, hidden 0.842857). Dev on this box (M5, sandboxed probe, idle): **0.792439**, worst `order()` **0.686 s**.

## Why this exists

The public record says the benchmark is blocked on the 2 s cap, not on score: 358 of the
first 572 submissions died at the cap, and byte-identical re-grades of the promoted tree
both passed (0.842833) and died within one day. Measured here on a GitHub `ubuntu-latest`
runner (fork-only timing workflow, Xeon 8573C, 2 cores / 4 threads): per-row `order()`
time is **2.59× this Mac's (median; p90 2.9×, max 3.4×)**, the tip's worst row is
**1.79 s** there (crudeoil_lee4_10) and ten rows exceed 1.5 s. Under bubblewrap and on a
hidden corpus that is 2.0 s. So the first thing to buy is time, with zero output change.

## Change 1 — `custom_metrics::order_variant` / `metric_sweep::order_generic` 1.5× per pass (output-identical)

A samply CPU profile of the ten slowest dev rows put 19.7 % of all self time in
`custom_metrics::order_variant` (the quotient-graph metric passes), more than any vendored
kernel. `src/ordering/qg_kernel.rs` now holds the one elimination step both callers carried
as private copies:

- workspace arrays split-borrowed into local slices once per step (LLVM reloaded every Vec
  header after each heap store; now they stay in registers) — 1.30× alone;
- Pass 1 / Pass 2 indexing unchecked under one `unsafe` block whose SAFETY note states the
  vendor list invariants, with `debug_assert!` on every access — 1.30× → 1.50×;
- the re-insertion score is a `Reinsert` trait; every family's formula text and evaluation
  order is verbatim; the only arithmetic touched memoises `powf` on integer-valued
  arguments by exact integer key (same bits).

Per-pass micro-benchmark on the ten slow rows: order_variant 5.939 s → 3.963 s (1.50×),
order_generic 2.640 s → 1.772 s (1.49×). Whole `order()` A/B under equal load: sum 6.09 s →
5.83 s, `1.portfolio` 2.22 s → 2.01 s; rows inside the 130k-nnz light tier gain 6–13 %,
rows above it are pinned by a different candidate and do not move.

**Identity:** a new test-only `PERM` line in `probe_timing_and_score` (FNV-1a over the
permutation) diffed over all 300 rows against the tip: `IDENTITY: OK (300/300)` (PERM and
exact COUNTS). Unit tests 121 passed (3 new).

## Change 2 — `best_flops` write-back (Xo1otl's 6664fb09, hidden 0.842835)

Stage 11 accepted a better `p` without writing its score back, the lt_1k refiners and the
PEO chain likewise, so the terminal transplant compared donors against a stale bound and
could admit a worse one. Reimplemented verbatim from the public archive of submission
`6664fb09` (graded −0.26 bip on hidden, cap-clean). Dev-invisible here: 300/300 rows
bit-identical in flops (score 0.792439 unchanged).

## Result

| | dev | fill | worst local | note |
|---|---|---|---|---|
| tip ab30c0e | 0.792439 | 0.924472 | 0.686 s | |
| this tree | 0.792439 | 0.924472 | ≤ 0.686 s (change 1 only reduces) | 300/300 OK, determinism gate passed |

Submitted as a **probe**: expected hidden ≈ 0.842835 (the write-back's known value); the
`Benchmark` step duration (tip ≈ 9 min) is the only observable of the time effect.

## Reinvestment candidates measured on dev (all thin — dev is exhausted)

| variant | dev delta | movers |
|---|---:|---|
| relabel budget ×1.5 in every bucket | +0.19 bip | 1 better / 3 worse |
| +2 search streams in every phase-3 table | −0.39 bip | 4 / 1 |
| the four `n>=1800 && nnz>=9000` / `(3000..8000)` danger gates removed | −0.37 bip | 2 / 2 |

## Next

Two more output-identical lanes are in flight (rgreedy engine + batch/scoring plumbing;
residual-core / subtree / PEO-alt allocation churn). Then per-row timing on the runner class,
then hidden probes one hunk at a time.
