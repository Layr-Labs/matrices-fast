# 0143 — Independent-set-first lift (the normal-equations family)

- **Date:** 2026-09-08
- **Score:** tip `fb851f6` (hidden 0.848742) dev **0.804851 → 0.798697** (−61.5 bips);
  rebased onto `f7f60dc` (hidden 0.848556, dev 0.804632) → **0.798477**, worst 0.963 → 1.030 s
- **Status:** win (dev); hidden pending
- **Files:** new `indep_first.rs`; `mod.rs` stage `1b.indep` after the portfolio flush plus a
  terminal strict-accept for held-back marginal candidates; `probe.rs` gains
  `probe_indep_first` / `probe_indep_variants`.

## Hypothesis

KKT / saddle-point patterns are (nearly) bipartite: constraint rows are pairwise
non-adjacent, and for LP / separable-QP KKTs so are the variables. Every candidate in the
portfolio — minimum degree, minimum fill, the quotient metrics, the separator codes — picks
pivots one at a time and *interleaves* the two sides. The classic alternative in
interior-point solvers is the normal-equations route: eliminate one whole side first and
factor the Schur complement. Nobody had tried that as an ordering candidate.

The useful generalisation is that for **any independent set `X`** the elimination of `X`
first has an order-free exact cost — no fill edge can ever reach a member of `X`, so

```
Σ_j c_j²  =  Σ_{x∈X} (1 + deg x)²  +  Σ_{w∈core} c_w²
```

with the core being `V \ X` plus one clique per `N(x)`. The first term is fixed and the
second is computed on the core alone, so core candidates rank exactly on the core graph, as
in `core_lift`. Where the Schur complement is grid-like (discretised control / PDE /
collocation families) a minimum-degree or separator ordering of the core should beat any
interleaved order.

## What changed

- `indep_first::greedy_independent_set` walks `(degree, index)` ascending and takes every
  vertex with no neighbour already taken, optionally only vertices of degree ≤ cap. Three
  caps are used: unbounded (the whole-side route), 9 and 3 (mid-degree vertices kept in the
  core). Distinct caps give distinct Schur complements and different rows want different
  ones (glider400 wants cap 3, crudeoil_lee2_06 cap 9, cont6-qq / torsion50 unbounded).
- `budget_trim` drops the highest-degree members until the predicted clique-pair count fits
  the ledger — hubs (dense KKT columns) stay in the core, which is also what normal-equations
  solvers do with dense columns.
- `lift` builds the exact core (hash edge set, membership-tested only) and `run` orders it
  with AMD (always), AMF α10 (sparse cores only: < 20 nnz/node and ≤ 600k nnz) and default
  METIS (cores of ≤ 30k nodes and ≤ 1M nnz — METIS's cost tracks nodes and hub structure,
  not nnz). Ranked exactly on the core, spliced, verified by the trusted full-graph scorer.
- Cost model in edge-touch units, `5·nnz + 9·pairs ≤ 8M` per set (~15 ns/unit measured on
  cont6-qq; giant cores are ~4× worse per unit, hence the AMD-only rule). The three sets run
  on their own scoped threads and merge by index; admission is decided from the pattern
  alone before anything starts, so nothing is ever started and killed.
- **Dual-track acceptance.** A decisive early win (≥ 2 % under the incumbent) replaces the
  incumbent right after the portfolio, so the subtree / PEO / MINL / FINAL_REFINE stages
  polish it. A marginal win is held back and applied LAST with strict accept. Measured
  reason: with unconditional early acceptance three sub-1.5 % wins (pooling_adhya4pq,
  waterund11, pinene200) turned into +0.2 … +9.4 % losses because the exact-search /
  subtree lotteries re-roll on a changed incumbent; every ≥ 2 % early win stayed ahead.
- The candidate is compared directly with `best_flops`, never through the runner-up ledger,
  so non-winning rows keep a bit-identical downstream trajectory (the ledger seeds PEO_ALT).

## Result

Full 300-row dev corpus, same box, `probe_timing_and_score`:

| revision | score | lt_1k | 1k_10k | gt_10k | worst call | rows changed |
|---|---:|---:|---:|---:|---:|---|
| tip `fb851f6` | 0.804851 | 0.8878 | 0.8426 | 0.7144 | 0.978 s | — |
| v1 early-accept always | 0.803615 | 0.8885 | 0.8419 | 0.7112 | 1.010 s | 9 better / 3 worse |
| v2 dual track {AMD, AMF} | 0.803561 | 0.8878 | 0.8420 | 0.7116 | 1.011 s | 7 better / 1 worse |
| v3 + METIS on core (nnz ≤ 300k) | 0.801519 | 0.8878 | 0.8420 | 0.7052 | 1.073 s | 8 better / 1 worse |
| **v4 METIS gate `cn ≤ 30k ∧ cnnz ≤ 1M`, AMF only sparse non-giant** | **0.798697** | 0.8878 | 0.8420 | **0.6994** | 1.057 s | 10 better / 1 worse |
| tip `f7f60dc` (certified reuse + window DP) | 0.804632 | 0.8877 | 0.8413 | 0.7144 | 0.963 s | — |
| **v4 rebased on `f7f60dc` (submitted)** | **0.798477** | 0.8877 | 0.8413 | **0.6994** | 1.030 s | 10 better / 1 worse, same rows |

Per-row (v2): cont6-qq 0.7983 → **0.6962**, torsion50 0.6762 → 0.6669, glider400 0.8889 →
0.8858, gabriel10 0.9480 → **0.9285**, crudeoil_lee2_06 0.9011 → **0.8446** (lift 0.8969,
then polished downstream), methanol400 −0.8 %, gasprod_sarawak81 −0.7 %; pinene200 +0.2 %.
`probe_indep_variants` (extra core orderers AMD-nd / AMD-na / AMD α5 / AMF α5,2,−1 /
SqDiv / SqPure / DegSqrt, and a recursive lift on the core) found **no** marginal winner
except METIS on the core: arki0013 0.5859 → **0.4393** (AMD/AMF on the same core: 0.62).

v4 adds METIS on the dense pooling cores (the gate is in NODES because METIS's cost
tracks the node count and hub structure, not nnz): pooling_sppc3pq 0.3922 → **0.2822**,
pooling_sppc1pq 0.1902 → 0.1683, arki0013 0.5859 → **0.4237**. All ten v4 wins are at
the exact-core rank; the terminal stages then polish them like any other incumbent.

Stage cost (v4, `probe_indep_timing`): the three sets run on their own threads, so the
wall cost is the slowest set's `lift + AMD + AMF + METIS + 3 scorings`. METIS is 60–90 %
of it: 45–55 ms on the 8–12k-node lee4 cores, 0.13 s on the 12k-node / 950k-nnz pooling
core and the 26k-node nuclear104 cores. Worst stage cost 0.25 s (pooling_sppc3pq, 0.88 s
total), 0.21 s nuclear104 (0.90 s), 77–93 ms on the two slowest rows (crudeoil_lee4_09/10,
1.05 s total; tip 0.98 s on the same box). Cheaper METIS shapes do not help: `niparts` 3
saves ~20 % of the pass and costs pooling's cap-9 core 0.283 → 0.421; `fm_passes` 4 is a
no-op. No structural gate separates the METIS-winning dense cores (pooling, 70–80 nnz per
node) from the METIS-losing ones (lee4 cap-∞, gabriel cap-∞, 30–110 per node) — the
AMD-on-core total is 2–2.7× the incumbent in both groups — so the pass stays as gated.

## Why it won / lost

- The wins are all discretised problems whose Schur complement is a mesh: cont6-qq
  (elliptic control, a 2-D grid), torsion50, glider400 / pinene / methanol (collocation
  chains), gabriel*, arki0013. Minimum degree on the augmented system eliminates grid and
  constraint nodes alternately and builds wide fronts; eliminating the independent side
  first leaves a clean grid where AMD/AMF (or METIS) does what it is good at.
- Dense KKT rows (pooling_*, nnz/n ≥ 20) blow the Schur complement up; `budget_trim` then
  leaves only a small `X` and the candidate loses to the incumbent by a wide margin — the
  best-of floor makes that free, and the ledger keeps it cheap.
- Whole bipartite sides (`bipartite_sides`, probe-only) never beat the greedy set on dev:
  the greedy walk already takes the low-degree side and keeps hubs out.
- Early acceptance is a double-edged sword: it earns the downstream polish (lee2_06 0.8969
  → 0.8446) but changes the lottery seeds' inputs; the 2 % margin separates the two regimes
  on every dev row measured.

## Follow-ups

- Recursive lift (independent set of the core) was null on dev; not shipped.
- A relabelled AMF lottery on the lifted core is the natural next ticket where the core is
  small; untested.
- The stage runs on every row inside `nnz ≤ 1.5M`; if hidden timing ever tightens, the
  first cut is the METIS pass on the cap-∞ set (never won on dev except pooling_sppc3pq,
  where the cap-9 set wins too), then `METIS_CORE_MAX_N` 30k → 20k (drops nuclear104's
  0.13 s pass, which never wins, and keeps arki0013's cap-9 core at 18k nodes).

## Links

- Techniques: [best-of-portfolio](../techniques/best-of-portfolio.md), [nested-dissection](../techniques/nested-dissection.md), [amd](../techniques/amd.md)
- Related: [0062 reduce-then-AMF](0062-reduce-then-amf-terminal.md) (the other fixed-prefix lift; that one eliminates low-degree vertices in min-degree order, this one an independent set in any order)
