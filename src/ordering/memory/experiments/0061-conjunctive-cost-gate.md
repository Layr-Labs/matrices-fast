# 0061 — A conjunctive cost gate for the terminal escalation

**Base commit:** `ea67ff80041e8e7717be32decdf95c1c1e80eb90` (`ea67ff8`), still
the promoted tip (official hidden **0.869826**, submission `56ee74d`) — checked
against `submissions --all` immediately before submitting, not assumed.

**Base re-measured on this box, in this session, before any edit:**

| run | score | lt_1k | 1k_10k | gt_10k | worst `order()` |
|-----|-------|-------|--------|--------|-----------------|
| this session | 0.843978 | 0.8903 (147) | 0.8670 (108) | 0.7920 (45) | **1.485 s** (`crudeoil_lee4_10`) |
| earlier session (0060) | 0.843978 | 0.8903 | 0.8670 | 0.7920 | 1.375 / 1.378 s |

Same score to the last digit, as it must be. The worst time moved 1.375 → 1.485 s
on identical code, which is the useful number here: **this box's timing noise is
about 0.1 s, ~8%**, and no timing conclusion below is stated to finer precision
than that.

## Why there was a second attempt

`0060` shipped this mechanism and **failed** on the hidden corpus. Its gate
bounded expensive search via the `work_spent` ledger and bounded nothing else,
while the escalation it admitted requested 4.1e9 word-ops against a 2.0e9
ceiling. Full post-mortem at the end of `0060-*.md`.

This experiment keeps the mechanism — unconditional re-postordered rounds after
the conditional chain stalls — and replaces the cost model.

## Hypothesis

If wall time is bounded on **both** independent cost axes, and the escalation's
own cost is small in absolute terms rather than merely small on the dev corpus,
the same mechanism ships safely and still clears the 3 bip bar.

## Measurement 1 — does `budget` actually bind? (`probe_budget_saturation`, new)

The question that decides whether requested work is a legitimate safety bound.
One `subtree_refine` pass at `max_s=768, max_blocks=32`, on the same postordered
tree, over the 270 eligible dev matrices, sweeping only the budget:

| budget | total wall over corpus | max single pass |
|---|---|---|
| 4M | 1.497 s | 0.0190 s |
| 8M | 2.937 s | 0.0380 s |
| 16M | 5.759 s | 0.0686 s |
| 32M | 11.531 s | 0.1688 s |
| 64M | 22.859 s | 0.2762 s |
| 128M | 45.990 s | 0.5406 s |

Almost perfectly linear — each doubling of the budget doubles the time, and the
**median per-matrix growth for a 4x budget rise is 3.976x**. The budget is not a
loose upper bound that blocks rarely reach; it is a direct time knob, and a
matrix whose blocks saturate it pays all of it.

This cuts both ways, and both directions matter. It means requested work *is* a
sound bound (good), and it means `0060`'s 4.1e9-op request was a real ~0.5-0.7 s
commitment on any matrix that saturates it (bad).

## Measurement 2 — what the ledger cannot see (`probe_work_ledger`)

Base `order()` time against the ledger, escalation disabled, 300 matrices.
Bucketing by an `est = ledger + 1200*nnz` fit (the best single linear estimator,
r=0.771) gives a **non-monotone** envelope:

| decile of est | max secs |
|---|---|
| d0 (lowest) | **0.714** |
| d1 | 0.650 |
| d2 | 0.141 |
| d3 | 0.134 |
| d4 | 0.165 |
| ... | ... |
| d8 | 1.374 |

The *cheapest* decile contains the 0.714 s matrix. `ringpack_30_2`: ledger 48M
— a fortieth of `0060`'s ceiling — n=17999, nnz=121458, **0.714 s**, essentially
all of it in the O(nnz) candidate families the ledger never counts. A
ledger-only gate calls this matrix free. That is the `0060` failure in one row.

## Measurement 3 — escalation cost rises with nnz (`probe_margin_cascade`)

Cost of four escalation rounds, by nnz band, at 8M:

| nnz band | count | max escalation cost |
|---|---|---|
| < 60k | 221 | 0.135 s |
| 60k–150k | 20 | 0.172 s |
| 150k–400k | 11 | 0.218 s |
| 400k–1.5M | 9 | **0.467 s** (`gabriel10`, nnz 1.15M) |

Each round re-postorders — permute, elimination tree, column counts — which is
O(nnz) and charged per round. So `nnz` bounds exactly the cost the ledger
misses, and the two bounds are complementary rather than redundant.

## The gate

Conjunctive, both bounds required:

```
work_spent < 500_000_000  &&  nnz <= 150_000  &&  n in SUBTREE_MIN_N..=SUBTREE_MAX_N
```

with the per-round budget cut 32M → 16M.

What that buys, on dev: **all 31 matrices whose `order()` exceeds 0.9 s are
excluded**, every one of them. The slowest matrix that still escalates is
`ringpack_30_2` at 0.714 s base, and the eligible set tops out at n=26778,
nnz=132380.

Tightening the ledger ceiling 2e9 → 5e8 drops 33 matrices from the eligible set
and costs **0.1 bip** (−6.42 → −6.32 in simulation). Safety that cheap is not a
trade-off.

## Variant table, including the losers

Gates simulated offline by joining per-matrix ledger data with a depth-4 cascade
at three budgets, so every row below is measured, not modelled. `worst esc` is
the worst `order()` among matrices that **actually escalate** — the number that
carries the risk. Rows are dropped if they raise the corpus worst case at all.

| budget | rounds | ledger ceiling | nnz cap | Δ bip | max esc cost | worst escalated total |
|---|---|---|---|---|---|---|
| **16M** | **4** | **5e8** | **150k** | **−6.32** | **0.229 s** | **0.878 s** | 
| 16M | 4 | 2e9 | 150k | −6.42 | 0.229 s | 0.878 s |
| 16M | 4 | none | 60k | −8.24 | 0.216 s | — (admits 1.15 s base matrices) |
| 8M | 4 | 5e8 | 150k | −4.81 | 0.131 s | 0.801 s |
| 16M | 3 | 5e8 | 150k | −3.45 | 0.163 s | 0.834 s |
| 16M | 2 | 5e8 | 150k | −3.06 | 0.130 s | 0.805 s |
| 8M | 3 | 5e8 | 150k | −2.58 | 0.096 s | under the bar |
| 16M | 4 | 2e9 | 60k | −1.60 | 0.215 s | over-tight, both gates bite |
| 32M | 4 | 2e9 | 1.5M | −7.47 | 0.467 s | 1.179 s — **this is `0060`, which failed** |

The `none / 60k` row scores best of all and is **rejected on principle**: with no
ledger bound it escalates `crudeoil_lee1_07` (nnz 19322, base 1.155 s) and other
expensive-search matrices. It only looks safe because dev's worst such matrix is
1.155 s. That is precisely the reasoning that failed in `0060`, so it is not
available, whatever it scores.

## Result

Dev **0.843978 → 0.843356**, **−6.22 bip**. Fill 0.943300.
**22 better / 0 worse / 278 unchanged.** All three buckets improve:

| bucket | base | candidate |
|---|---|---|
| lt_1k | 0.8903 | 0.8901 |
| 1k_10k | 0.8670 | 0.8661 |
| gt_10k | 0.7920 | 0.7912 |

Largest movers: `gasprod_sarawak81` 1.0000→0.9604 (gt_10k), `pooling_sppa9tp`
0.4464→0.4307, `pooling_sppa9pq` 0.6492→0.6302, `graphpart_clique-70`
0.8103→0.7888, `arki0002` 0.9953→0.9764, `popdynm200` 1.0000→0.9979 (gt_10k).

## Timing

| revision | worst `order()` | matrix |
|---|---|---|
| base, this session | 1.485 s | `crudeoil_lee4_10` |
| base, previous session | 1.375 / 1.378 s | `crudeoil_lee4_10` |
| candidate | 1.365 s | `crudeoil_lee4_09` |
| **candidate, worst matrix that ACTUALLY escalates** | **0.918 s** | `ringpack_30_2` |

Every one of the ten slowest matrices in the candidate run is skipped by the
gate and runs byte-identical code to the base; the only added instructions on
their path are the ledger's integer adds. The number that characterises this
change is **0.918 s** — a 2.2x margin to the 2 s cap, and 0.45 s below the base
worst. `0060` shipped 1.179 s here and failed.

## Validation

`bash scripts/local-candidate-build.sh && cargo run --release --offline --locked`
run twice: `results.tsv` status **OK** both times, `score.json` byte-identical
at **0.843356** / fill 0.943300, 300 matrices, no worker failures, no capped
matrices. 44 unit tests pass including `order_is_deterministic`. The harness
itself runs `order()` twice per matrix and compares, so determinism is checked
by the harness and not only by the unit test.

## Ties, again

`0060` found that ties are a bad *target* (3.6% conversion versus 20–42% below
the anchor) but must not be *excluded*, since the few that move are `gt_10k`.
That reproduces here at the tighter gate: exactly two ties move, both `gt_10k` —
`gasprod_sarawak81` 1.0000→0.9604 and `popdynm200` 1.0000→0.9979 — and
`gasprod_sarawak81` alone is worth roughly 2.6 bip of the 6.22.

## Caveats

- Dev gain is concentrated in ~6 matrices. `0060`'s bootstrap over the dev
  corpus put a −7.47 bip dev result in a 95% interval of [−20.92, −2.10] bip on
  a resampled corpus; a −6.22 bip result should be read the same way. The
  interval avoids zero only because the mechanism is monotone and structurally
  cannot lose score.
- The gate constants are still dev-calibrated. What changed since `0060` is that
  they now bound both cost axes rather than one, and that the admitted work is
  small in absolute terms (0.23 s worst) rather than merely small in the dev
  sample (0.47 s worst). Both bounds fail safe: more prior work, or more nnz,
  means no escalation.
- The ledger still counts work **requested**, not **issued**. Measurement 1
  shows the two are close when blocks saturate, which is the conservative
  direction for a safety bound, but it does make the ledger a poor *estimator*.

## Next

- The oracle bound (deepest cascade per matrix that still fits under the base
  worst) was −19.11 bip in `0060`, so most of the time-feasible gain is still
  held by the matrices this gate excludes. The way in is a **cheaper round** for
  them — 4M x 8 rather than 16M x 32 — not a looser gate.
- Make the ledger count work issued. That turns it from an envelope into an
  estimator and is the only thing that would justify a ceiling near the cap.
- `probe_budget_saturation` should be re-run on any future change to
  `subtree_refine`: the linearity it establishes is what makes budget a
  legitimate time knob, and it is an assumption, not a theorem.
