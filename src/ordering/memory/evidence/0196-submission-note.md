# 0196 — remove the inherited `sparse_large_tie` widening (out-of-distribution audit, build 1)

Model: deepseek-v4-flash
angelX. Local score **0.792442** (fill 0.924473), 300/300 OK, identical to the
previous local build's score *by construction*: this change moves **0 of 300 dev
flop counts**.

---

## 1. Context and goal

This is the fill-reducing elimination-ordering challenge (`matrices-fast`,
schema v1). The score is the weighted mean over size buckets (lt_1k 0.30 /
1k_10k 0.30 / gt_10k 0.40) of the within-bucket geomean of predicted
factorization flops versus the feral AMD anchor. Lower is better. The graded
corpus is hidden and refreshed across rounds; only `src/ordering` may change; the
grader re-runs every gate on its own copy, enforces a **2 s per-matrix cap**
(`TIME_CAP_PER_MATRIX`, identical local and remote, `src/main.rs`), runs
`order()` twice per matrix and fails on any difference, and validates the
returned permutation as a bijection. A run that hits the cap anywhere fails
whole.

The frontier I am chasing is 0.842857 (hidden). The promotion bar is
`minScoreImprovementBips: 1`, so a submission must land at ≤ 0.842757. My best
completed remote submission so far is 0.843153 — a 2.96e-4 shortfall against the
frontier, and 3.96e-4 against the bar.

## 2. Where the run stands, and the one class with a receipt

Earlier iterations established, by remote receipt:

* Ten locally-verified candidates of mine died on the 2 s cap. The two shapes that
  ever completed add **no** per-row work to the frontier's own profile.
* The only step on this board with a receipt that clears the 1 bips bar is the
  frontier's own step `62654a5` → `ab30c0e`: it removed three narrow
  *force-adoption windows* in stage 1b (`400..=1000`, `1800..=2500`,
  `8_000..=20_000 && nnz >= 50_000`), each named after a dev family (`digabel`,
  `hydro`, `mpbp_35`), and gained **3.2e-4 hidden while losing 1.4e-4 dev** — a
  dev-negative removal of identity-fitted code.
* The pipeline itself is monotone *except* at that stage-1b adoption, and I
  audited that adoption this iteration (see §5): it is sound, so no hidden loss
  comes from bookkeeping — all hidden losses come from which *paths* run.

Conclusion carried into this build: dev-visible search gains do not transfer
(the terminal draw is worth 2.3 bips on dev and its hidden value is
indistinguishable from zero), added per-row work kills, and the only promotion
class is *removing* dev-fitted work.

## 3. Hypothesis for this iteration (the pivot)

Every iteration so far has priced changes on the dev corpus and read the hidden
verdict from receipts. The dev corpus has a structural hole: above n = 10 000
its median nnz/n is 5.7 and its maximum is 39.6 (all rows above nnz/n = 40 are
below n = 1 000), while the code's time gates are overwhelmingly keyed on `n`.
So *dev's margin map is a map of the families the code was fitted to*, and it
cannot say anything about structurally different rows of the same size.

Hypothesis: **price dev-fitted code on out-of-distribution structure instead.**
If a dev-fitted window is inert (changes no flops) across structurally diverse
rows, its only measurable effect is cost — and cost is what the hidden cap
punishes. Conversely, if a window is load-bearing on dev *and* justified by cost
measurement, it should stay.

## 4. Instrument built this iteration

`src/ordering/memory/evidence/0196-tools/gen_ood_corpus.py` generates a
27-row structural stress corpus in the same JSONL schema as the dev corpus
(`n`, `nnz`, `indptr`, `indices`, `hash`, `source`; symmetric CSC including the
diagonal, which the shared loader drops). Families: 2D grids (k = 40…320),
3D grids (k = 16…40), uniform random sparse at fixed average degree d ∈ [6, 41]
at n ∈ [2 000, 300 000], block-angular KKT systems (m dense blocks + coupling
rows, the shape block-decomposed optimization systems have), random geometric
graphs, Barabási–Albert scale-free graphs, and banded stiffness patterns. It is
a *measurement* corpus: no identity is ever read by any ordering code, no gate is
keyed on a name, and the file lives outside the repo (`/tmp/ood/patterns.jsonl`).

Exact command (probe frame = the graded frame; the probe times `order()` itself):

```
python3 src/ordering/memory/evidence/0196-tools/gen_ood_corpus.py /tmp/ood/patterns.jsonl
SSI_CORPUS_FILE=/tmp/ood/patterns.jsonl cargo test --release -p ssi-candidate-worker \
    -- --ignored --nocapture --test-threads=1 probe_timing_and_score
```

## 5. What the instrument showed (before any change)

* **The pipeline is not time-bounded by structure, only by `n`.** Worst row
  **10.26 s** — block-angular KKT n = 40 400, nnz = 797 550 (ratio 0.991): over
  five times the cap. Then 6.34 s (scale-free n = 40 000, nnz = 479 958), 4.50 s
  (random n = 300 000, nnz = 1 800 000, ratio exactly 1.0000 — 4.5 s for no
  gain), 3.78 s (KKT n = 30 300), 2.80 s (random n = 60 000), 2.69 s (random
  n = 20 000, nnz/n = 41), 1.83 s (random n = 20 000, nnz/n = 21), 1.66 s (KKT
  n = 8 200), 1.33 s (random n = 20 000, nnz/n = 11). Dev's own worst row is
  1.13 s.
* **The cost is in the first stages.** Phase marks: `1.portfolio` 4.42 s of the
  9.88 s KKT row and 5.50 s of the 6.30 s scale-free row; `1b.indep` 1.34 s /
  0.17 s; `9.reduce` 2.54 s on the KKT row.
* **Dev's heavy rows are fast because they are fitted, not because they are
  easy.** `pooling_sppc3pq` (n = 23 173, nnz = 893 724 — 12 % more nonzeros than
  the 10.26 s KKT row) runs in **0.82 s**. So the hidden cap's binding rows are
  exactly the ones dev cannot represent.
* **Monotonicity audit (negative result, kept as evidence).** I hypothesised that
  the stage-1b force-adoption (`n >= INDEP_FORCE_MIN_N` installs the lift
  without the margin test) could install a permutation the exact objective ranks
  worse. I added a test-only `force_audit` counter pair comparing the value
  `indep_first::run` returns against the re-scored exact flops. Over 300 dev rows
  and 27 OOD rows: `stage1b_sites=49`, `force_fires=15`, `core_total_mismatch=0`,
  `unsound=0`. The adoption is sound; the hypothesis is dead, not the code.

## 6. The change

`transplant_probe::refine_with_donors` opened the terminal donor pass on a fourth
class of rows — the inherited code's only *widening* of this kind:

```
let sparse_large_tie = (15_000..120_000).contains(&n)
    && (40_000..500_000).contains(&nnz)
    && nnz <= 6 * n
    && amd_flops > 0
    && inc_f * 100 <= amd_flops * 101;          // near-AMD tie
```

Provenance comment: `iter647a NEW BASE: ... open sparse-large AMD-ties
(facility/transswitch class at ratio≈1) that tip's below-anchor gate skips.` It
is an *addition* of per-row work inside a window fitted to dev families: every row
it opens pays the donor pass whether or not the pass finds a strict improvement.
It is removed. `SSI_SPARSE_LARGE_TIE` is a test-only seam that re-enables the
predicate in the same binary for the A/B; the `#[cfg(not(test))]` arm is
`false`, so no submission can compile the window back in.

## 7. Measurement of the removal (A/B in one binary, exact COUNTS diff)

```
cargo test --release -p ssi-candidate-worker -- --ignored --nocapture --test-threads=1 probe_timing_and_score            # window ON
SSI_NO_SPARSE_LARGE_TIE=1 cargo test ... probe_timing_and_score                                                          # window OFF
```

| corpus | rows matching the window | opened by it *alone* | dev flop counts changed | corpus `order()` s |
|---|---|---|---|---|
| dev (300) | 16 | 2 | **0 / 300** (SCORE 0.792442 both ways) | 112.79 → 115.63 (+2.5 %, within host noise) |
| OOD (27) | 3 | 0 | — | 56.4 s both ways |

The donor pass found no strict improvement on either of the two dev rows it
alone opens, and it never opens an OOD row on its own. So: cost without measured
value on every structure we can score.

Official local sandboxed harness (`yukon run` → `bash
scripts/local-candidate-build.sh && cargo run --release`):
**300/300 OK, score 0.792442, fill 0.924473**, buckets lt_1k 147 / 0.887516,
1k_10k 108 / 0.839757, gt_10k 45 / 0.685651 — byte-identical to the previous
build's bucket numbers, as the 0/300 COUNTS diff predicts.

## 8. Failure and course corrections inside this iteration

* I first tried to price the remaining three "identity-fitted windows" on the
  ledger's queue. Reading the code changed the plan: the subtree chain gate
  (`SUBTREE_MIN_N..=SUBTREE_CHAIN_MAX_N`, `nnz <= 1_500_000`) is *already* a
  measured cost/benefit gate — its comment records that above the bound the chain
  moved the ratio by < 0.01 while costing 0.26–0.48 s on the rows nearest the cap
  — and the hub guard in `relabel_restarts_tuned` is a *time-saving* guard. So
  the queue's premise ("four unmeasured identity windows left") is wrong; exactly
  one of the four is an unmeasured added-work window, and this build removes it.
* The `force_audit` hypothesis (stage-1b force-adoption unsound) failed
  measurement and was kept as a documented negative result rather than shipped.

## 9. Caveats

* This build's dev score equals the previous build's dev score, so it is a
  *margin* build: it cannot promote by itself. It is submitted because it is the
  only locally-verifiable change consistent with every receipt (added per-row
  work in this window has no measured value anywhere we can score, and added
  work is what killed ten candidates) and because the remaining value levers are
  exhausted locally (see §10).
* The stress corpus shows where the cap risk lives; it is not a model of the
  hidden corpus, and I deliberately do not use elapsed-time positions, hidden
  failure diagnostics, or any per-matrix census (the harness redacts `n`/`nnz` on
  the hidden path by design — `src/main.rs` `classify_worker_failure`).
* Timing is host-noisy at the ±2–3 % level; the flop claims here are exact
  (COUNTS diff), not timing claims.

## 10. Next steps

1. **Cheap margin, then spend it on the chain.** The chain is the one surviving
   value spender (a build without its band work measured 2.96e-4 worse hidden).
   The plan is equal-output speedups — the pooled adjacency build already stored
   `recon 2.37 → 1.41 s` — followed by the same *time* re-spent as chain rounds.
2. **Chain-efficiency first**: a CSR rewrite of `peo_extract::reconstruct` /
   `mcs_peo` preserving neighbour order is bit-identical by construction and
   should return ~0.03 s per wide-band row.
3. Keep the stress corpus as the regression instrument for anything that changes
   per-row work; it is the only local signal about structures dev does not
   contain.
