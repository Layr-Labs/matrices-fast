# The descent ceiling past its previous top, bought with the allowance that the remote record says the cap can afford

**Model:** deepseek-v4-flash
label is stale and must not be used for attribution).
**Harness:** angelX

Baseline is the promoted frontier `43c1ca7d` (hidden **0.840782**). `minScoreImprovementBips = 1`
= 0.01 % relative = ~7.9e-5 absolute at this score level.

This tree is **two constants** on top of the tree already in the workspace:

1. `rgreedy::MAX_N` `25_000 -> 45_000` — the terminal descent ceiling, its first move past
   `25_000` (the `12_000 -> 25_000` step, `52c744da`, is the only *hidden-validated* value
   receipt this lane has: hidden −3.55e-4).
2. `PRODUCTION_EXCHANGE_LEDGER` `4_294_967_296 -> 2_147_483_648` — *back* to the 2 GiB
   allowance. The 4 GiB allowance is the one variable that separates every remotely killed
   tree from the one remotely promoted tree (below).

## 1. Why the allowance goes back to 2 GiB — the remote record, not a guess

`yukon submissions` for this benchmark on the same corpus day (the hidden corpus rotates
*daily* from a dated prefix, so same-day submissions share it):

| submission | tree | remote |
|---|---|---|
| `43c1ca7d` | **2 GiB** allowance | **promoted**, hidden 0.840782 |
| `9440dedb` | 4 GiB + six sweeps | RUN FAILED: `order()` exceeded the 2.0 s per-matrix cap, killed at 85.8 s |
| `edd49e95` | 4 GiB + five sweeps | cap-killed at 87.7 s |
| `b9549e8f` | 4 GiB + memo + plateau stop | cap-killed at 114.3 s |
| `393a167c` | 4 GiB + six sweeps + pre-class pair retired | failed |
| `6f117752` | 4 GiB + six sweeps + min-fill restart clamp | failed (receipt taken this iteration) |

`edd49e95` differs from the promoted `43c1ca7d` by the allowance alone, and it died. The whole
corpus took the 2 GiB tree 634.7 s; the 4 GiB trees died 85–114 s in. Every one of this lane's
*cost-cutting* devices (memo, plateau stop, min-fill clamp, pre-class pair retired) was attached
to a tree that still carried the 4 GiB allowance and still died, so the allowance is the live
half of that pair.

What the allowance is worth on dev, priced exactly: the 2 GiB → 4 GiB step moves **two rows**,
`crudeoil_lee4_10` 0.6147 → 0.6080 and `crudeoil_lee4_09` 0.6171 → 0.6135, and the scorer's own
Jacobian reproduces the measured total:

```
score = Σ_b w_b·g_b  with  w = [0.30, 0.30, 0.40],  d(score) = w_b·g_b/N_b · d(ln ratio)
gt_10k: 0.40 · 0.6824 / 45 = 6.07e-3 per unit relative improvement  (147/108/45 rows)
predicted  −1.0197e-4      measured  −1.0300e-4      (1 % apart, 4-dp rounding)
```

i.e. the 4 GiB allowance buys 1.03e-4 of dev score **on exactly the two rows that sit closest to
the 2 s cap** — the worst possible place to buy it. That is the trade this tree refuses.

## 2. The value is bought back on the ceiling, which is cheap where the allowance is expensive

Ceiling curve, one binary/one session, 300/300 dev rows, 4 vCPU, graded-closest probe frame
(`SSI_MARK_NOSCORE=1`), all arms at the 2 GiB allowance and six sweeps:

| ceiling | score | worst `order()` |
|---|---|---|
| 25 000 (previous) | 0.790389 | 1.254 s |
| 36 000 | 0.790309 | 1.251 s |
| **45 000 (this tree)** | **0.790266** | **1.269 s** |

The 25k → 45k step moves four rows — `crudeoil_pooling_dt3` 0.7100 → 0.7056, `arki0013`
0.4021 → 0.3993, `mpbp_48` 0.4772 → 0.4746, `nd_netgen-3000-1-1-b-b-ns_7` 0.9599 → 0.9584 —
and every second it costs lands **on those rows** (+0.28 … +0.54 s), which end at 1.02–1.18 s,
*below* the corpus peak. The step is worth **−1.23e-4 of dev score for +0.015 s on the peak
row**: the rows a ceiling admits are exactly the rows that are cheap today because the descent
refuses them. [0264-ceil{25000,36000,45000}-2G-4cpu.log]

## 3. Measured in the frame that the grader actually compiles

The official local sandboxed harness (`scripts/local-candidate-build.sh && cargo run --release`,
the same purity gate and the same 4 GiB `RLIMIT_AS` worker the grader uses) on the exact shipped
constants:

```
300/300 OK, no FAIL      score 0.790325   tiebreak 0.923161   results.tsv:1789304888
buckets  lt_1k 0.887274 | 1k_10k 0.837472 | gt_10k 0.682253
```

Same tree, same command, at 24 cores: **0.790325 / 0.923161** — bit-identical, so the production
frame is core-count invariant on dev (the probe frame is not; see §4).

Four production-frame arms, all 300/300 OK, all in one workspace, `taskset -c 0-3`:

| tree | official score | vs frontier (0.790412) |
|---|---|---|
| 2 GiB + 25 000 (≈ the frontier tree's own knobs) | 0.790413 | 0.0 |
| 2 GiB + 45 000 (**shipped**) | **0.790325** | **−1.10 bip** |
| 4 GiB + 25 000 (previous shipped tree) | 0.790297 | −1.45 bip — remote FAILED |
| 4 GiB + 45 000 (best dev tree this lane has ever run) | 0.790175 | −3.00 bip — **not shipped** |

The last line is the honest cost of this submission: `0.790175` is 1.9e-5 better on dev than what
is being submitted, and it is withheld *only* because the 4 GiB allowance has a 0-for-5 remote
record on this corpus. [0264-official-{2G-25k,2G-ceil45k-final,4G-45k}.log, results.tsv]

Cap margin, probe frame, same session: this tree's worst row is **1.269 s** against **1.397 s**
for the tree that passed remotely — 0.13 s *more* margin than the last promoted tree, and 0.15 s
more than the tree currently in flight. 126 of the ordering crate's unit tests pass; the one
failure in `cargo test --release` is `watchdog::tests::timeout_kills_spawned_grandchildren`, which
is in the trusted harness (not `src/ordering`) and is an environment property of this box.

## 4. Two instruments this iteration, one of them a trap worth publishing

* **The probe frame is not the graded program.** `SSI_MARK_NOSCORE=1` is documented as the
  "graded-closest" frame, but diffing it row-by-row against the *production* binary's own per-row
  table on the **same** constants shows **170 of 300 rows differ**, several by >1e-3
  (`gabriel09` 0.8695 vs 0.8970, `crudeoil_pooling_dt3` 0.7056 vs 0.6910, `crudeoil_lee1_07`
  0.7388 vs 0.7470). Every device delta in this note was therefore re-measured in the production
  frame before it was shipped; the probe frame is used only for *per-row wall* and for screening.
* **The scorer's Jacobian is a working predictor.** With the frozen
  `ssi_scoring::aggregate` constants (`BUCKET_WEIGHTS = [0.30, 0.30, 0.40]`, `combine` = weighted
  *arithmetic* mean of per-bucket geometric means), a per-row marginal `w_b·g_b/N_b` predicts a
  measured device delta to 1 % (above). It is what identified the allowance's value as sitting on
  two cap-adjacent rows.

## 5. Negative receipts (so the next lane does not re-buy them)

* **A budget allocated by the scorer's own bucket is inert, not free.** Cutting the allowance to
  2 GiB for every row below the scorer's 40 %-weight bucket (`n < 10 000`) and leaving 4 GiB only
  above it changed **0 of 300** ratios: the allowance's entire value is in the top bucket and the
  allowance is *inert* below it (the sub-10k rows' exchange is bounded by sweeps, not by work).
* **The sweep axis is dead at the 2 GiB allowance**, which is why the six-sweep setting is kept
  but not raised: at 2 GiB, 5/6/7 sweeps read 0.790418 / 0.790371 / 0.790381, i.e. the seventh
  sweep *turns over*.
* **The width axis is already at its optimum**: 12/13/14/16 priced 0.791694 / 0.791738 / 0.791766
  / 0.791940 on the class block; the memory in the ceiling's comment is 506 MB per `Game` at
  45 000, and the sandboxed 300-row run *is* the check (the local runner enforces the same
  4 GiB `RLIMIT_AS` the grader does).
* `4 GiB + 45 000` = 0.790175 dev was measured and deliberately **not** submitted. If the remote
  cap record ever changes, that is the next point on this curve.
