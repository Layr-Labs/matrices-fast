# 0146: the stale-incumbent invariant, stage-13 retirement, and identity-window removal

Base: promoted frontier `62654a5` (hidden **0.843173**, `jonathan308`), locally
remeasured dev **0.792300** (lt_1k 0.887390 / 1k_10k 0.839747 /
gt_10k 0.685397), worst `order()` **1.479 s** on a 16-vCPU box.

Result: dev **0.792452** (+1.9 bip), worst **1.303 s**, official 300-pattern
harness run **OK**. Not submitted — the change is deliberately score-negative on
dev; see "Why this was still taken".

## 1. A correctness defect: `best_flops` drifts above the incumbent

`leader_order` carries the incumbent as `(best_flops, best_perm)` and every
terminal stage admits on `f < best_flops`. Three stages improved `best_perm`
without writing the score back:

- stage 11 core-candidate — `if f < best_flops { best_perm = p; }` (the score
  `f` was already computed, just not assigned);
- stage 11 `lt_1k` refiners — `cutoff_paired_swap_refine` /
  `cutoff_plateau_refine` return only a permutation;
- stage 12 PEO chain — true score kept in a round-local `final_flops`;
- stage 13 alt-seed chain — true score kept in a block-local `leader_flops`.

Stage **14 (terminal transplant)** is the one consumer that trusts the raw
value, so once any of those stages won, a donor with
`true_score < f < best_flops` was admitted — a strict regression inside a
pipeline whose safety argument is that regressions are structurally impossible.
Stage 15 recomputes `score(&best_perm)` and takes a `min`, which is why the
damage self-healed and stayed invisible.

Stage 12's round body already carried the comment *"Earlier terminal stages can
change best_perm without updating best_flops, so derive the incumbent's exact
score afresh"* — the hazard was diagnosed previously, defended against locally
inside stage 12, and left in place for stage 14.

**Fix:** restore the invariant at each site using the score already in hand
(hoisting `peo_true_flops` out of the PEO round loop for stage 12). All four
write-backs are monotone non-increasing and cost nothing beyond one `flops_of`
on `n ≤ 1000 && nnz ≤ 8000` rows. A `#[cfg(test)]` reporter at the stage-14
entry prints `STALE_BEST_FLOPS` on any residual violation; it fires on **0 of
300** rows after the fix.

**Score effect: exactly nil** (0.792300, buckets identical). On this corpus
stage 14 never actually exploited the window. Keep it anyway: it is latent, not
absent, and dev is disjoint from a rotating hidden corpus.

### The important consequence is measurement, not score

The phase marker prints `best_flops`, so while it was stale every stage
attribution downstream of stage 11 was wrong. With the invariant repaired,
stage 12's gains become visible for the first time:

| row | after `9.reduce` | after `12.peo` |
|---|---|---|
| `nuclear104` | 0.8997 | **0.7846** |
| `mpbp_35` | 0.4225 → 0.3639 | **0.3342** |
| `arki0013` | 0.4328 | **0.4215** |
| `crudeoil_lee4_10` | 0.6452 | **0.6340** |
| `crudeoil_lee4_09` | 0.6605 | **0.6445** |
| `gabriel09` | 0.9210 | **0.9131** |

**Any inherited claim about which late stage earns its keep should be
re-derived rather than trusted.**

## 2. Stage 13 (alternate-seed PEO chains) retired

First whole-corpus cost/value census taken with a correct `best_flops`
(`SSI_PROBE_PHASES=1`, 299 rows). `fired` = spent >1 ms, `gained` = strictly
improved the incumbent:

| stage | Σ secs | fired | gained |
|---|---|---|---|
| `3.search` | 28.69 | 203 | 67 |
| `9.reduce` | 18.63 | 232 | 29 |
| `4.subtree` | 10.11 | 230 | **111** |
| **`13.alt`** | **8.38** | **176** | **3** |
| `1b.indep` | 8.00 | 210 | 24 |
| `14.transplant` | 1.28 | 159 | **70** |

`13.alt`'s three winners are `maxcsp-ehi-85-297-71` (−0.00149 ln),
`mpbp_15` (−0.00372) and `syn40hfsg` (−0.00176): **−0.00697 ln total ≈ 0.21
bip**, against 8.38 s and 0.14–0.22 s on the rows nearest the cap (`arki0013`
224 ms on a 1.18 s row, `mpbp_48` 184 ms, `crudeoil_pooling_dt3` 183 ms,
`gabriel09` 169 ms).

It cannot be re-gated: affordable chain depth (`PEO_ALT_LEDGER / (n+nnz+Lnnz)`)
does **not** separate outcomes — winners sat at depths **2, 40, 619** while
expensive no-gain rows sat at **4, 11, 19, 23, 34, 64**. A winner at depth 2
and losers at depth 34–64 kill the premise, so no depth or size gate recovers
the value.

The mechanism is also now redundant: with the invariant repaired, `12.peo`
demonstrably reaches those gains first on exactly the rows this stage was once
credited for (`mpbp_34`, `mpbp_35`, `arki0013`, `gabriel09` — see the table
above). A second chain from a discarded runner-up cannot improve on an
incumbent stage 12 has already driven to a minimal triangulation.

Retiring it: worst row **1.348 → 1.200 s**, rows above 1.0 s **17 → 13**,
corpus 149.0 → 140.5 s, for 0.21 bip. `PEO_ALT_SEEDS` stays — the transplant
still draws donors from that pool. The obsolete `probe/alt_lineage.rs` is
removed with it.

## 3. Three identity-keyed `n`-windows removed

`RULES.md` forbids special-casing evaluation instances and states inherited
overfit code is not exempt. Stage 1b force-adopted the lift inside four
predicates named after dev-corpus families:

```rust
let digabel_band = (400..=1000).contains(&n);
let hydro_band   = (1800..=2500).contains(&n);
let gasprod_band = n >= 20_000;
let mid_force    = (8_000..20_000).contains(&n) && nnz >= 50_000; // "skip mpbp_35 class"
```

Removed: the two narrow windows and `mid_force`. **Kept** as
`INDEP_FORCE_MIN_N = 20_000`: `gasprod_band` is an ordinary monotone predicate
on `n` ("on large patterns prefer the lift"), not a window fitted around
particular rows, and it is load-bearing — removing it costs 41 bip of `gt_10k`
geomean. Cost of the removals: **≈1.7 bip**, in all three buckets.

`indep_first.rs:419` still routes `(1800..=2500)` to `run_sequential_180`, a
whole alternate implementation. **Still open** — see "Next".

## Negative results

- **Symmetric subtree polish at 4b: −21 bip, all in `gt_10k` (0.6854 →
  0.6895).** Giving a deferred lift one round of the stage-4 chain before
  comparing it to the polished incumbent looks like the principled fix for the
  windows above. It is not: one round is no proxy for the full 7-round chain
  plus stages 5–12, so it only lets marginal lifts win the comparison and then
  underperform downstream — the failure the stage-1b comment describes (a lift
  leading raw by 8 % ending 22 % behind). Reverting it alone restored `gt_10k`
  to 0.6857. A fair comparison must be made **before** stage 4, or stage 4 must
  run on both candidates. **Do not retry the one-round polish at 4b.**
- **`INDEP_WORK_LEDGER` 8M → 12M: 0.792512 (0.6 bip worse), worst 1.303 →
  1.450 s.** `1b.indep` is the most productive stage in the tree by far
  (−3.607 ln for 8.0 s ≈ 0.45 ln/s, ~8× `9.reduce`), but high average
  productivity does not imply high marginal productivity — the extra ledger
  admits the candidate sets `budget_trim` had already ranked worst.
- **`INDEP_IMMEDIATE_MARGIN` sweep:** (1,1) adopt-on-any-lead **0.793843**
  (+14 bip); (9,10) shipped **0.792452**; (4,5) 20 % lead 0.792482. Flat between
  10 % and 20 %, sharply worse when loosened. The general knob the removed
  windows were patching is already at its optimum; **their 1.7 bip is not
  recoverable here.**
- **`lt_1k` has no idle budget.** Only 23 of 300 rows finish under 10 ms and
  **174 of 300 already cost ≥ 0.4 s** (mean 0.53 s). Any new spend must be
  funded by removing existing spend. **Do not plan work on "the unused 2 s at
  small n".**

## Why this was still taken, despite +1.9 bip on dev

The removals are mandated (identity-keyed gates) or buy cap safety, which is
the binding constraint: the public submission list is dominated by `failed`,
the frontier has not moved in ~24 h, and this tree reads **1.30–1.48 s worst on
a 16-vCPU box** against a 2 s SIGKILL with ~1.6× repeat noise. A scored run of
the *unmodified* frontier failed locally on `nuclear104` purely from CPU
contention.

Dev and hidden are disjoint and hidden rotates, so windows fitted to dev family
`n`-ranges plausibly contribute ≈0 hidden while costing 1.7 bip dev. That is an
argument for expecting the hidden cost to be far below 1.9 bip, **not** evidence
of it — untested.

## Next

1. `indep_first.rs:419` — replace the `(1800..=2500)` route to
   `run_sequential_180` with the two capabilities that path uniquely has, under
   their own **core-shape** gates: an AMF α=5 pass (`cn ≤ 4_000 && cnnz ≤
   60_000`) and a 2-seed relabelled-AMF pass (`cn ≤ 3_000 && cnnz ≤ 30_000`),
   added to `run`'s Phase 2. The window exists because `COMPETITIVE_MARGIN`
   (1.5×) prunes a core those passes would have won on.
2. `1b.indep` wastes **0.31 s each on `faclay75` and
   `acopf_case9241pegase_qcqp`** (1.02 s and 1.04 s rows) for zero gain. Skip it
   where the ledger cannot admit a set able to restructure the matrix:
   `max_pairs / nnz` was 0.088 and 0.132 there against 0.219 (`gabriel10`) and
   0.44 (`pooling_sppc3pq`), both of which it wins. Score-neutral, ~0.3 s of
   tail safety.
3. `9.reduce` is the remaining large waste block that still carries real
   winners: 15.5 s of waste for 29 gains, worst single waste 0.252 s on
   `arki0016` (a 1.11 s row). Needs a discriminating gate, not retirement.
4. A **per-matrix work ledger** across stages remains the structural law that
   would bound unseen classes; the census here is the calibration data for it.

## Addendum: the `lt_1k` tie residue is a real floor

`probe/ceiling.rs` (new, test-only) re-implements the elimination game
independently of the pipeline — dense bitset, incremental `Σ c_j²` accumulated
as `Σ (1 + d_j)²` where `d_j` is the live pivot degree — and searches with
randomized-greedy restarts under min-degree, min-fill and blended pivot rules.
The incremental identity is asserted against the trusted `flops_of` on the AMD
order for every row probed (verified on 54 rows), so the instrument is checked
rather than assumed.

Over the **54** `lt_1k` rows the pipeline leaves at ratio ≥ 0.9999 it played
**7.0 million full elimination games** (up to 1.14M on one row, thin only on
dense rows where a game is costly — `qapw` n=705 nnz=87496 got 5) and improved
**zero** of them.

This corroborates the earlier "gap ties resist exact search — closed" verdict
with fresh, independent evidence: AMD is at or extremely near the optimum on
that residue. Caveat: the search family is min-degree/min-fill greedy, the same
family as AMD, so it does not formally exclude a structurally different method —
but the portfolio already applies ND, METIS, Scotch and KaHIP there.
**Do not spend further effort on the tie residue.**

Run with:
```sh
CEIL_SECS=1 CEIL_MAX_N=1000 cargo test --release -p ssi-candidate-worker \
  --offline --locked -- --ignored --nocapture --test-threads=1 probe_ceiling
```
