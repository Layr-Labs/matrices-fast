# 0062 — Exact low-degree elimination prefix + residual-core AMF/AMD, terminal (reduce-then-AMF)

- **Date:** 2026-09-05
- **Base:** `784bfe5` (submission `c7d5fe71`, [0061](0061-margin-scaled-leftover-search.md); official hidden **0.86837**; dev re-measured on this box **0.843358**, worst isolated `order()` 1.823 s)
- **Score:** dev 0.843358 → **0.837523** (-0.692% rel, 69.2 bip); independent out-of-sample corpus (591 MINLPLib KKT patterns outside the 300 dev names) 0.863823 → **0.855486** (-0.965% rel, 96.5 bip)
- **Status:** WIN locally on both corpora (dev −69 bip, out-of-sample −97 bip, 0 rows worse); submitted for hidden validation

## Hypothesis

Every vertex whose live degree is <= 3 can be eliminated EXACTLY up front: eliminating it closes its live
neighbourhood into a clique, so the residual graph is the exact fill graph after the prefix, and the objective
splits exactly, `Σ c_j² = (fixed prefix term) + (term computed on the residual core alone)`. The residual core is a
smaller instance of the same problem (median ~n/3 on this corpus) with the same objective. Every ordering routine
that reads the input numbering behaves differently on the core than on the full matrix, so the AMF alpha grid + AMD
on the core reach basins the full-matrix portfolio, the relabel multistarts and the subtree/LNS searches never
visit. Because the prefix is exact, a core ordering can be ranked on the core graph (~100x cheaper than a
full-graph scoring pass) and only the argmin needs the trusted global scorer.

Placed TERMINAL on purpose: as a portfolio candidate the same orderings displace the pool argmin and re-seed the
descent phases (non-monotone: an earlier in-pipeline placement regressed rows that the pipeline had won). After
the finished pipeline, with strict `<`, it is monotone by construction and the AMD anchor still holds.

## What changed

- New `src/ordering/core_lift.rs`: `reduce(sp, max_row_deg, max_core_n, max_core_edges) -> Option<CoreLift>`
  (pendant peel + exact degree-<=K elimination in ascending `(degree, index)` order, clique closure on a
  membership-only hash SET with a fixed-constant hasher — no hash order ever reaches the output), `splice(cl,
  core_perm)`. Fails closed (`None`) on every cap so the incumbent is untouched.
- `mod.rs`: consts `REDUCE_MIN_N = 50`, `REDUCE_MAX_NNZ = 1_500_000` (the AMF envelope), `REDUCE_ROW_DEG = 3`,
  `REDUCE_MAX_CORE_N = 60_000`, `REDUCE_MAX_CORE_EDGES = 3_000_000`, `REDUCE_MAX_CORE_NNZ = 1_500_000`,
  `REDUCE_ALPHAS = [0.5, 2.5, 5.0, 10.0]`; one terminal block after the 5/4-pivot descents: reduce → four AMF
  alphas + AMD on the core on scoped threads (merged by task index) → rank on the core graph → splice the argmin →
  trusted `flops_of` → strict `<`. Gated on `(n, nnz, core size)` only.

## Result

| corpus | crown | candidate | better / worse | worst order() |
|---|---|---|---|---|
| dev | 0.843358 | 0.837523 | 16 / 0 | 1.823 s -> 1.284 s |
| out-of-sample | 0.863823 | 0.855486 | 15 / 0 | 1.574 s -> 1.459 s |

Timing (isolated `probe_timing_and_score`, same box, same session): dev worst crown 1.823 s → candidate
1.284 s; out-of-sample worst crown 1.574 s → candidate 1.459 s. No crown row >= 0.8 s slower by more
than 0.05 s. `cargo test --release -p ssi-candidate-worker`: 50 passed, 0 failed (18 ignored probes).

**dev movers (16 rows better, 0 worse):** nuclear10a (gt_10k, n=17493, nnz=163816): 0.9912 -> 0.7602; crudeoil_pooling_dt2 (gt_10k, n=18742, nnz=75910): 0.8372 -> 0.7418; nuclear104 (gt_10k, n=39098, nnz=257806): 0.9925 -> 0.8998; pinene200 (gt_10k, n=19995, nnz=97990): 0.9407 -> 0.8567; gasprod_sarawak81 (gt_10k, n=22536, nnz=75636): 1.0000 -> 0.9400; crudeoil_pooling_dt3 (gt_10k, n=30660, nnz=152210): 0.9488 -> 0.9002; faclay75 (gt_10k, n=272878, nnz=1379706): 1.0000 -> 0.9667; hydroenergy2 (1k_10k, n=2092, nnz=6236): 0.8543 -> 0.8263; faclay35 (gt_10k, n=26778, nnz=132380): 1.0000 -> 0.9687; faclay30 (gt_10k, n=16678, nnz=82242): 0.9988 -> 0.9729; sonet21v6 (1k_10k, n=8232, nnz=40744): 1.0000 -> 0.9839; methanol400 (gt_10k, n=23999, nnz=151728): 0.7367 -> 0.7289; methanol200 (gt_10k, n=11999, nnz=76128): 0.7364 -> 0.7296; unitcommit_200_100_1_mod_8 (gt_10k, n=146830, nnz=476332): 0.9839 -> 0.9764.

**out-of-sample movers (15 rows better, 0 worse):** pooling_sppb5stp (gt_10k, n=27255, nnz=738608): 0.9329 -> 0.6098; mpbp_04 (1k_10k, n=4734, nnz=20800): 0.9799 -> 0.9162; edgecross20-040 (1k_10k, n=9503, nnz=38518): 0.8021 -> 0.7538; faclay30h (gt_10k, n=16678, nnz=82242): 0.9988 -> 0.9729; sonet20v6 (1k_10k, n=7070, nnz=34964): 1.0000 -> 0.9863; faclay20h (1k_10k, n=4753, nnz=23700): 0.9981 -> 0.9863; sonet19v5 (1k_10k, n=6023, nnz=29758): 0.9982 -> 0.9877; sonet18v6 (1k_10k, n=5085, nnz=25096): 0.9984 -> 0.9893; sonet17v4 (1k_10k, n=4250, nnz=20948): 0.9981 -> 0.9899; pooling_sppc3stp (gt_10k, n=33387, nnz=967510): 0.9937 -> 0.9880; pooling_sppc1stp (gt_10k, n=19673, nnz=516464): 0.9837 -> 0.9786; rsyn0805m04hfsg (1k_10k, n=5400, nnz=14640): 0.9536 -> 0.9504; portfol_shortfall200_05 (1k_10k, n=1615, nnz=164428): 0.5408 -> 0.5400; portfol_robust100_09 (lt_1k, n=813, nnz=42020): 0.4834 -> 0.4833.

## Why it won / lost

The win is a basin change, not a better heuristic: on the core, AMF/AMD see a graph whose low-degree fringe is
gone, so their dense-row deferral and pivot choice differ from the full-matrix run. Attribution from the previous
challenge round (same dev corpus): the whole gain came from one AMF pass at the right alpha on the core; relabel
tickets and terminal refinement on the core added nothing there. Rows the crown already ties at AMD with
structured exact-floor families (squfl / sssd / kall / autocorr) do not move — those ties are at the exact
elimination floor.

## Follow-ups

- Relabelled AMF/AMD multistart ON THE CORE (tickets are ~3x cheaper than full-matrix tickets).
- Multi-depth prefixes (K in {2, 4, 5}, distinct cores only) — different K reach different basins.
- Core RECURSION (the late phases on the core, spliced) under a per-call WORK budget in operations, as a
  replacement of a late phase on the classes where it pays — never additive on the slow class.

## Links

- Techniques: [best-of-portfolio](../techniques/best-of-portfolio.md), [amd](../techniques/amd.md)
- Validation: dual-agent verification harness (propose / hold-apart / assay) with an independent out-of-sample
  corpus as the judge; timing is a hard gate that is never overridden.
