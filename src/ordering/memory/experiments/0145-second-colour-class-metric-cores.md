# 0145 — Second colour class + metric passes on lifted cores; acceptance after subtree

- **Date:** 2026-09-08
- **Score:** tip `691aad4` (hidden 0.844528) official local **0.798268 → 0.795608** (−26.6 bips);
  gt_10k 0.6994 → 0.6935; worst 1.040 → 1.055–1.126 s (lee4_09/10; noise band)
- **Status:** win (dev); hidden pending
- **Files:** `indep_first.rs` (`greedy_independent_set_excluding`, `fill_greedy_independent_set`
  probe-only, two-phase `run` with a flat deterministic task pool), `mod.rs` (immediate
  accept at 1b for ≥ 40 % leads; strict accept at a new point `4b` after the subtree
  stage; old 2 % 1b rule and terminal deferred-accept removed),
  `probe.rs` (`probe_indep_sets`: alternative set constructions × core-orderer menu).

## Hypothesis

[0143](0143-independent-set-first-lift.md) showed the independent-set-first lift saturates on
its core orderers (extra AMD/AMF variants, METIS shapes, recursive lift: nothing) — but the
*set* had only been the greedy low-degree class. Two things had not been tried:

1. **The second colour class.** On a KKT the greedy `(degree, index)` set takes the low-degree
   side; the *other* side (hubs / constraint rows) is the set interior-point solvers actually
   eliminate first (normal equations eliminate the variables, leaving `A D Aᵀ`). Generalised to
   non-bipartite graphs: the greedy independent set among the vertices the first set did not
   take. `bipartite_sides` in 0143 could not see this because the crudeoil KKTs are not
   bipartite (diagonal blocks), so it returned `None` on exactly the rows where it matters.
2. **The quotient-graph metrics on the core.** `probe_census` on crudeoil_lee4_06 shows the
   pipeline's best single orderer on the *full* graph is `DegP075` at 0.751 versus AMD 0.865;
   0143's variant probe had only tried SqDiv / SqPure / DegSqrt on cores.

## Measured (probe_indep_sets, all sets × {AMD, AMF, METIS, 11 metrics}, dev tip finals)

| row | pipe | best lift | set / pass |
|---|---:|---:|---|
| crudeoil_lee4_06 (n 10.4k) | 0.6601 | **0.5424** | x-inf / DegPlusDegme (AMD on the same core 0.6285) |
| crudeoil_lee4_09 | 0.6814 | **0.6512** | x9 / DegSqrt; g3 / DegDivNvSqrtWf 0.6651 |
| crudeoil_lee4_10 | 0.6755 | **0.6489** | x9 / DegDivNvSqrtWf; g6 0.6510, g4 0.6512 |
| crudeoil_lee2_06 | 0.8442 | **0.8082** | x-inf / DegDivNvSqrtWf |
| gabriel09 | 0.9357 | 0.9234 | g2 / DegSqrt (marginal) |

Metric passes cost 5–15 ms on the 8–12k-node lee cores (METIS 45–70 ms) but 0.6 s per pass on
the 80k-node cont6-qq core, hence `METRIC_CORE_MAX_N = 16k`, `MAX_NNZ = 200k`. Three metrics
cover every win: DegDivNvSqrtWf, DegPlusDegme, DegSqrt. Fill-greedy sets (`f*`) equal the
degree-greedy sets on bipartite-like rows (every `N(v)` is independent, so new fill = all
pairs) and never differed where it mattered. Extra caps 2 / 4 / 6 / 16: g16 = g-inf on gabriel09
only. METIS shapes on cores (imb 0.02/0.05/0.10, seed 21, niparts 16, sw 50/800): one row,
−0.0014 ln.

## What changed

- Sets: g-inf, g9, g3, **x-inf, x9** (x-sets only for nnz ≤ 600k — on giants they only add two
  lifts and AMD walks).
- Passes in two phases: (1) lift + AMD on every admitted core, in parallel; (2) on cores whose
  AMD total is within 1.5× of the best (dense whole-side cores lose by 2–100× and never
  recover under any orderer): AMF (sparse, non-giant), METIS on the **two** lowest-AMD cores
  (every dev METIS win is on one of those; the third never won), the three metrics on the
  **three** lowest-AMD cores inside the metric envelope. One flat task list on
  `PAR_MAX_THREADS` threads, results by index.
- Acceptance, two live tiers (the old 2 % 1b rule and the terminal deferred-accept are gone):
  - **1b immediate** if the lift is ≥ 40 % under the portfolio best (`INDEP_IMMEDIATE_MARGIN`
    3/5). The largest *pure* downstream gain on any dev row is 32 % (mpbp_34/35, nuclear10a,
    gams05, ringpack_20_2), so this lead cannot be overtaken; it also keeps the subtree stage
    off a poor incumbent (pooling_sppc3pq: 0.88 → 1.03 s when the lift waited for 4b).
  - **4b strict** (new point right after the subtree stage) if the lift beats the
    subtree-polished incumbent at all. v6 (the old 1b ≥ 2 % rule) accepted the lift on
    mpbp_35 at a 7.8 % lead and the row went 0.3226 → **0.3951**: the old incumbent's
    downstream polish was 31 % (subtree −10 %, completion −14 %, transplant −8 %) and the
    lift's only 8 %. The subtree stage is the strongest polish and the best available
    predictor of the rest: with the check after it, mpbp_35 stays 0.3226 and the lee rows
    still take the lift (their subtree gain is < 1.5 %). Every later stage is monotone, so
    a lift that loses at 4b cannot win later.

## Result (full dev corpus, same box)

| revision | score | gt_10k | worst | vs tip |
|---|---:|---:|---:|---|
| tip `691aad4` | 0.798268 | 0.7144 | 1.040 s | — |
| v6 five sets + metrics, 1b ≥ 2 % accept | 0.796753 | 0.6962 | 1.086 s | lee4_06 0.507, lee4_09 0.6425, lee4_10 0.6347, lee2_06 0.7591; **mpbp_35 0.3226 → 0.3951** |
| v7 accept after subtree, METIS top-2 | 0.795682 | 0.6935 | 1.126 s | mpbp_35 back; pooling_sppc3pq 0.88 → 1.03 s |
| v8 + immediate tier ≥ 40 %, metrics top-3 | 0.795666 | 0.6935 | 1.095 s | lee4_06 **0.5044**, lee4_09 0.6473, lee4_10 0.6374, lee2_06 **0.7677**, mpbp_34 0.3031; methanol400 +0.8 %, gasprod_sarawak81 +0.7 % (their 1b lifts were < 2 % ahead once the incumbent had been subtree-polished) |
| **v9 strict 4b (shipped); official `yukon run`** | **0.795608** | **0.6935** | 1.055 s | extra: glider400 0.886 → 0.879, torsion50 −0.2 %; graphpart +0.08 % |

Stage cost (1b, v8): lee4_09 0.077 → 0.101 s, lee4_10 0.094 → 0.126 s, nuclear104 0.24 s,
pooling_sppc3pq 0.24 s, gams05 0.18 s (was 0.20: g-inf's 986k-nnz core no longer gets METIS).

## Follow-ups

- The lee rows' polish (0.5424 → 0.5044) comes from subtree + transplant/minl on the lift; with
  acceptance after subtree the lift misses the subtree pass. Running subtree on an accepted
  lift would cost 0.13–0.16 s on the slowest rows — not affordable there; could be gated to
  rows under ~0.6 s.
- x-sets at further caps (x3, x16) and metric α variants untested.
