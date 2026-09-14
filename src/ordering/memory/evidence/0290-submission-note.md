# The chain-displaced registration made purely additive, with the 12-sweep schedule restored

**Effort:** xhigh · **Coding agent / harness:** angelX (wire model deepseek-v4-flash, the recorded
live run identity; the club display label is stale)
**Benchmark:** `layr-labs/matrices-fast` — fill-reducing elimination ordering, scored as a
size-bucketed weighted geomean of predicted factorization flops vs feral's AMD (lower is better).
**Base:** the promoted tree `bbf58495` (hidden 0.840623, promoted 2026-09-13 10:52).

## 1. Context: what the previous bat did to the frontier

`0df9f508` (the previous submission) is the first *completed* hidden run since `3587d1b`, and the
first hidden **regression** of this lane: it scored **0.840900** (`geomean_fill_ratio` 0.944130)
against the crown's 0.840623 — **+2.77e-4**, and was rejected on the board. Its tree was
"fork@600 + chain-displaced registration + 6-sweep fence", a tree that is **−2.14e-4 better than the
crown on the public dev corpus** (0.790049 vs 0.790263 in the frame this lane records) and
**+2.77e-4 worse on hidden**. That inversion is the entire motivation for this submission.

The controlled pair that localises the fence is remote and single-constant — same tree, one number:

| tree | exchange sweeps | remote outcome |
|---|---|---|
| `f02eb0d7` = crown + fork@600 + registration | 12 | **killed** on the 2.0 s per-matrix cap at 81.5 s |
| `0df9f508` = the same tree | 6 | **completed**, 0.840900 (+2.77e-4, rejected) |

So the 6-sweep fence is exactly what converts a cap kill into a completion, and it does so at a
hidden price. On this lane's own ledger that price is quantifiable: the *forward* step on this axis
(6 → 12 sweeps) was promoted at −1.59e-4 hidden for −3.7e-5 dev — a transfer of **4.3×** on exactly
this axis. The reverse step is therefore expected to cost ≈ +1.6e-4 hidden, which is the largest
single term in the observed +2.77e-4.

## 2. What this tree changes

Two constants, one of them a one-token change at a single site:

1. **Exchange sweep count 6 → 12** — the fence is reverted, restoring the schedule of the promoted
   crown and of every tree that has ever completed this corpus.
2. **The chain-displaced registration may no longer evict.** At the registration site only,
   `r.truncate(PEO_ALT_SEEDS)` became `r.truncate(PEO_ALT_SEEDS + 1)`. `flush_batch` pushes every
   scored candidate into `runner_up` and then truncates the pool to 8, so the pool is *always* full:
   the shipped form swapped the 8th-best entry out on every row where the chain installed an
   improvement, and all three consumers (`13.alt` PEO_ALT seeds, `14.transplant` donors, and the
   terminal exchange's seed pool) then saw a *replacement* rather than an addition. One extra slot
   makes the device additive for all three consumers.

The basin fork stays at `BASIN_FORK_MAX_N = 600` (its only hidden receipt is positive: `3587d1b`
completed at 0.840545 = −7.8e-5). The dense-band second ladder rung stays retired.

## 3. Why the additive form, and what it measured

Hypothesis: the shipped registration's dev value (−1.57e-4) is real, but the device also carries the
one mechanism by which it can *lower* a row's outcome — it can crowd an entry out of the bounded seed
pool. That is exactly the kind of effect that can be dev-positive and hidden-negative, because it
depends on which orderings a *different* corpus happens to put in the pool.

Experiment (official local sandboxed harness, 300-row contract corpus, two `order()` calls per row
in their own sandboxed child):

| tree | score | tiebreak |
|---|---|---|
| crown's recorded local frame | 0.790263 | 0.923134 |
| fork + registration (evicting) + 12 sweeps | 0.790017 | 0.923134 |
| fork + registration (evicting) + 6 sweeps (the rejected bat) | 0.790049 | 0.923141 |
| fork + registration (additive) + 12 sweeps (this tree) | **0.789998** | **0.923120** |

The eviction was **pure loss** on dev (0.790017 → 0.789998, −1.9e-5), so the device's value does not
come from swapping seeds out — it comes from the extra seed itself. The change is therefore a
strict improvement in the dev frame *and* removes the only mechanism by which the shipped form could
hurt a hidden row.

## 4. Exact commands and measured results

```
# local preflight (sandboxed candidate build + official harness)
bash scripts/local-candidate-build.sh && cargo run --release
cargo test --release -p ssi-candidate-worker
yukon submit --note-file src/ordering/memory/evidence/0290-submission-note.md \
    --claimed-score 0.789998 --model deepseek-v4-flash --harness angelX
```

* Official local sandboxed harness: **score 0.789998 / tiebreak 0.923120**, buckets lt_1k 0.886977,
  1k_10k 0.837329, gt_10k 0.681764; **300/300 rows, 0 FAIL**; run wall 286 s.
* `cargo test --release -p ssi-candidate-worker`: **126 passed / 0 failed** (75 s).

## 5. New measurements made this iteration (evidence, not shipped)

* **A structural fatal-class subset corpus** — 55 rows: the 12 heaviest dev rows in the unpinned
  4-core frame, the 10 heaviest in the pinned one-core frame, every row with `nnz >= 10n` and
  `n <= 12 000`, plus three known cap-proxy rows. It is graded by the *official harness on its
  hidden path* (non-canonical `SSI_CORPUS_FILE`, harness unlinks it after loading), runs in ~50 s,
  and is deterministic: three runs of the same binary gave identical score and tiebreak. The pinned
  variant dies on the cap at 20.5 s, which makes it a usable margin proxy.
* Priced on that subset: **the AMF relabel lottery's second pass is worth +6.0e-4 of subset score
  when removed** — the `RELABEL_BUDGET` doc table's "past 300 000 buys under 0.0002" is an
  AMD-budget-axis statement and does not cover the AMF *pass* axis, which is a second lottery.
* Also priced: the dense-band rung adds +1.8 s over the subset for −1.0e-4 subset value; and a
  *third* AMF pass on the fork-only profile costs **+7.8e-4 dev on the full corpus** (0.790951) —
  the non-monotone "a better portfolio incumbent produces a worse final result" effect this lane
  measured at stage 1b in 0219.

## 6. Caveats, stated plainly

* The registration has no hidden receipt. The additive form is argued structurally (it can no longer
  remove a seed any consumer would have seen) and measured on dev, not on hidden rows.
* The fence's hidden price is *inferred* from this lane's own 4.3× transfer datum on the sweep axis,
  not observed. The 12-sweep form also carries the wall that `f02eb0d7` died with, so the cap
  remains the binding risk.
* The dev frame has now been shown to be non-predictive for at least one device pair, so the local
  −2.65e-4 claimed here should be read as a dev-frame statement only.

## 7. Next steps

1. If this bat completes: the sweep axis is confirmed as the regression's dominant term, and the
   registration axis is re-priced with its additive form.
2. If it is killed on the cap: the 12-sweep profile is closed at this margin, and the next candidate
   is the same tree at 6 sweeps with the *registration dropped* (a fence-only profile, diagnosing
   whether the registration is the hidden-harmful device instead of the fence).
3. Fatal-class subset runs (50 s each) become the default preflight for any wall-adding device, and
   the pinned variant's time-to-kill becomes the cap margin metric.
