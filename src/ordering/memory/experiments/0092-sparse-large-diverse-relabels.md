# 0092 — Sparse-large diverse AMD relabels plus ledger-bounded core expansions

- **Date:** 2026-09-07
- **Score:** baseline 214cb89 0.824729 → 0.824578 (−1.51 bips), fill 0.936976 → 0.936927
- **Status:** win (local full 300-matrix run, 69 tests pass, worst order() 0.94–1.07 s vs 0.69 s pre-change on same host)

## Hypothesis

Matrices with `nnz > RELABEL_BUDGET` get zero relabel restarts, yet the highest-leverage
`gt_10k` ties live there (`acopf_case9241pegase_qcqp` n=313068/nnz=1292408 gets literally
nothing but the baseline). On sparse patterns AMD cost tracks `nnz` and sparse-large is
cheap per `nnz`, so a few extra i.i.d. AMD tickets gated to sparse non-hub large rows should
be affordable pure upside via the best-of floor. Diversity of AMD configs (aggressive ×
dense_alpha) should beat same-config repeats at equal cost, mirroring the AMF alpha-sweep
argument in [0005](0005-relabelled-amf-multistart.md) and [0006](0006-cycled-amf-amd-multistart.md).

Secondary ledger-bounded expansions (same worst-case bound, more draws): `PEO_ALT_SEEDS`
8→14 under the unchanged 4M shared ledger (more seeds, same budget), residual-core
`CORE_MINFILL_MAX_CN` 4000→6000 under the unchanged 16M per-row word ledger (slowest dev rows
have `core_nnz`=410700 ≫ 30000, so zero admitted calls there), small-core relabel gate aligned
to the same CN, exact re-ranking of extra-depth proxy picks on small cores, and tiny-graph
MinFill restarts 24/6→40/10 (membership matrix ≤1M, milliseconds, off the tail).

## What changed

`src/ordering/mod.rs` only (production; test-only probes untouched except for constants):

- New sparse-large block after the relabelled-AMF multi-start: if `n>=10000 && nnz>300000 &&
  nnz<=1500000 && nnz<=5*n && max_deg*50<=n`, run 4 AMD relabels with fixed seeds 9000–9003,
  cycling `(aggressive, dense_alpha)` over `(true,10),(false,10),(true,5),(false,5)`,
  composed back through `Q`, routed through `consider` (bijection-checked, strict `<`).
- `PEO_ALT_SEEDS` 8→14 (truncate only; per-round cost still charged `n+nnz+Lnnz` against the
  shared 4M `PEO_ALT_LEDGER`, so worst-case time is unchanged).
- `CORE_MINFILL_MAX_CN` 4000→6000 (comment updated); small-core relabel gate
  `(8..=4000)`→`(8..=CORE_MINFILL_MAX_CN)`; extra-depth proxy picks re-scored exactly on
  `8<=cn<=6000 && core_nnz<=30000` (up to 3 `flops_of`, same envelope as minfill).
- Tiny MinFill restarts 24→40 (`n<=1000 && nnz<=5000`) and 6→10 (`n<2000 && nnz<10000`).
- Small-core relabel alphas `[2.5,10.0,0.5,5.0]`→`[2.5,10.0,0.5,5.0,1.0,16.0]` (measured no
  additional win alone, retained as diversity at ~20–40 ms on gated rows).

All gates are pure functions of `(n,nnz,max_deg,cn,core_nnz)` — never identity, clock, env,
or filesystem — and every acceptance is strict `<` on exact `Σc²`, so score risk is
structurally zero; only time is at stake.

## Result

Full 300-matrix trusted run (`bash scripts/local-candidate-build.sh && cargo run --release`):

| | 214cb89 baseline | candidate 0092 |
|---|---|---|
| weighted flop geomean | 0.824729 | **0.824578** (−1.51 bips) |
| fill tiebreak | 0.936976 | 0.936927 |
| lt_1k (147) | 0.889720 | 0.889718 |
| 1k_10k (108) | 0.857003 | 0.856988 |
| gt_10k (45) | 0.751781 | **0.751416** |

Ablation (same host, full runs):
- seeds12/cn6k/minfill32-8/exact-rerank alone: 0.824729→0.824724 (+0.05 bips).
- + sparse-large 2×(true,10) nnz>500k: →0.824650 (+0.74 bips total).
- + widen to 4× nnz>300k: →0.824634 (+0.95 bips total).
- + 6× same-config: →0.824584 (+1.45 bips) but worst 1.138 s.
- 4× diverse configs (shipped): →**0.824578** (+1.51 bips), worst 0.94–1.07 s across runs
  (noise band; pre-change worst 0.69 s on same host).

69 tests pass. `probe_timing_and_score` worst `order()` 0.940 s (second run 1.067 s, 1.13× —
within the documented ~1.6× run-to-run variance from machine load). Pre-change worst on the
same host 0.69 s; the +0.25–0.38 s is the 4 large AMD passes. `nnz<=5*n` + non-hub +
`nnz<=1.5M` keeps dense/hub large rows out; the ten slowest dev rows take zero minfill-core
calls (`core_nnz`=410700 ≫ 30000).

Negative controls (reverted, same host):
- `CORE_MINFILL_MAX_CORE_NNZ` 30k→60k alone: 0.824724→0.826397 (**−16.7 bips**). The per-row
  shared ledger is spent in call order, so admitting larger cores first starves a later
  winning core — expanding the gate without enlarging the ledger regresses. Reverted.
- RCM/Sloan/ND/GGGP `MAX_N` 1000→2000: 0.824724→0.824724 (no win, consistent with
  [0020](0020-medium-exact-search.md) and [0075](0075-residual-core-minfill.md) zero-win
  screens). Reverted to preserve margin.
- `relabel_budget_and_cap` +100k per bucket: 0.824724→0.824727 (−0.03 bips). More upstream
  tickets reshuffle `runner_up` and can evict a downstream alternate-seed winner —
  non-monotonic via seed interaction. Reverted.

## Why it won / lost

Large sparse ties get zero tickets from `RELABEL_BUDGET/nnz` (500k/1.29M=0), so they are pure
upside no other family reaches — the same “second lottery” logic as [0005](0005-relabelled-amf-multistart.md),
but for the budget floor rather than the objective. Diversity matters at equal cost:
4× same-config gave +0.95 bips, 4× diverse gives +1.51 bips for the same 4 passes, because
`(aggressive, dense_alpha)` changes the elimination order as much as the relabeling does.
The 6× same-config score (+1.45 bips) shows more tickets also help, but diversity helps more
per unit time.

The ledger-bounded expansions contribute only +0.05 bips combined — the residual-core band is
at its knee (0091: 16M is the knee, 32M scores identically), and the alternate-seed ledger
is shared, so more seeds are more draws on the same budget, not more budget.

## Follow-ups

- Condition the minfill-core pass on portfolio disagreement (0091 §“honest one”): it spends on
  193/300 rows for 6 winners — the same unconditional shape as the failed 0088 bundle. Gate on
  `proxy_best != exact_best` or AMF/AMD disagreement already computed.
- The `CORE_MINFILL_MAX_CORE_NNZ` regression shows per-row ledgers couple gates; any future
  gate widening needs a ledger increase measured together, not alone.
- Sparse-large AMF (not just AMD) on the same gate: AMF per-pass constant is larger, so price
  with `probe_family` in isolation before adding.

## Links

- Techniques: [best-of-portfolio](../techniques/best-of-portfolio.md)
- Experiments: [0005](0005-relabelled-amf-multistart.md), [0006](0006-cycled-amf-amd-multistart.md), [0084](0084-alternate-seed-chains-shipped.md), [0091](0091-residual-core-exact-minimum-fill.md)
