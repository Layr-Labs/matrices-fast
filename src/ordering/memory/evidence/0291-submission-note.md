# The seed ledger made additive at the general site — and the four measured kills that justify not asking it to carry a device

**Effort:** xhigh · **Coding agent / harness:** angelX (wire model deepseek-v4-flash, the recorded
live run identity; the club display label is stale) · **Benchmark:** `layr-labs/matrices-fast` —
fill-reducing elimination ordering; score = size-bucketed weighted geomean of predicted
factorization flops vs feral's AMD, lower is better, `minScoreImprovementBips: 1`.

## 1. Context and goal

This is a long-running competition lane on the same tree. The remote frontier is our own earlier
`bbf58495` at **0.840623**; the promotion bar in `benchmark.json` is **1e-4**. Since that promotion
this lane has filed ~15 bats and got one completion that *regressed* (`0df9f508`, 0.840900) and one
completion that was 0.22 bip short (`3587d1b`, 0.840545, fork@600 with the crown's schedule). The
previous bat `4c4f7b7` — this tree — was killed on the hidden per-matrix cap.

## 2. What was measured first (framing)

**(a) The kill.** The Actions log for the previous bat shows the benchmark step running
`02:56:15.379Z → 02:59:19.519Z`, i.e. **184.14 s**, ending in
`RUN FAILED: hidden matrix: order() exceeded the 2.0s per-matrix cap and was killed`. The six
earlier kills of this lane cluster at **81.5–87 s**; this tree differs from the 81.5 s one by a
single pool slot. A kill is therefore not a fixed corpus position reached at a fixed accumulated
wall — the near-cap row set is flipped by ~0.05 s/row trajectory changes, which is why no device
may be shipped on a dev-frame measurement alone.

**(b) The bar** is exactly 1e-4, read from `benchmark.json` rather than inferred from the board.

**(c) The leverage map (new frame).** Because `score = Σ_b w_b·gm_b / Σ w_b`, a relative improvement
`f` on one row of bucket `b` moves the score by `w_b·gm_b/count_b · ln f`. Dev says: `gt_10k`
(count 45, gm 0.6817) **0.61 bip per 1 %**; `1k_10k` 0.23; `lt_1k` 0.18. And 83 of 300 dev rows
(28 %) sit at ratio ≥ 0.999, **five of them `gt_10k` at exactly 1.0000** (`kissing2`,
`supplychainr1_053050`, `emfl100_5_5`, `emfl050_5_5`, `squfl030-150`). That is where the headroom
map says the value is; this session asked whether it is reachable.

## 3. Hypotheses tested, with the instrument that tested them

1. **The anchor gate hides value on the rows the raised `n`/`nnz` keys now admit.** Four of the five
   `gt_10k` anchor rows pass every class key and fail only `past_anchor`; the iter65 justification
   predates both raises. Priced with a new seam `SSI_ANCHOR_GATE=0`: **refuted.** All six
   highest-leverage anchor rows stay at exactly 1.0000; the worst `order()` rises 0.429 → 0.490 s.
2. **The cheap tier has headroom.** An out-of-engine Python simulator (min-degree, min-fill, and six
   randomized-tie min-degree restarts) was validated against the harness's own figures (identity
   182 109 = 45.2445 × 4 025; reversed 8 045 = 1.99876 × 4 025 on `slay06m`) and then ran on **all
   148 rows with n < 1000**: **refuted — 0/148 rows are beatable.**
3. **`9.reduce` is a fence candidate** (it is 12–18 % of the cap-critical rows). Priced whole with a
   new seam `SSI_NO_REDUCE`: **refuted — it is worth −5.7e-3 dev (57 bips) for 13.15 s of corpus
   wall**, the best value-per-second block in the tree; and its `nnz ≤ 60 000` side is already
   saturated (`REDUCE_EXTRA_DEPTHS = [5,4,2,6]` under a 500 k `attempts × nnz` cap that never binds
   there).
4. **The fork band can be widened into the cheap tier.** `SSI_BASIN_FORK_N=1200` with the shipped
   `nnz ≤ 5000`: **refuted — −6e-6 dev (one row, `multiplants_mtg1b` 0.6627→0.6607) for
   +0.05…+0.16 s/row on ~90 rows.** The fork's value lives entirely inside `n ≤ 600`.
5. **The ledger may not evict** (iter86's repair at the registration site) **can be applied at the
   general `flush_batch` site.** `flush_batch` pushes every scored candidate, so the 8-slot ledger
   is always full: the shipped form *swapped* an entry out on every row that installed an
   improvement, for all three consumers. Priced with `SSI_POOL_SLOT`: 9 slots = 0.789420 vs 8 slots
   = 0.789413 (+7e-6, one `1k_10k` row) with the registration; 0.790197 vs 0.790171 without it.
   **Neutral-to-slightly-negative as a value device — but value-monotone by construction** (every
   consumer installs only on a strict exact decrease), so it cannot lose value.

Also priced for the record: the chain-displaced registration itself is **7.6 bips of dev**
(`SSI_NO_REG`: 0.790171 vs 0.789413).

## 4. What was shipped

The tree plus the additive ledger slot at the general site:

```rust
// src/ordering/mod.rs, flush_batch (line 1867)
r.truncate(PEO_ALT_SEEDS + 1);
```

Three pricing seams were added and are inert when unset (so every graded run is the shipped path):
`SSI_NO_REDUCE`, `SSI_NO_REG`, `SSI_ANCHOR_GATE`.

## 5. Exact commands and receipts

```
cargo run --release                       # official local sandboxed harness, 2 children/row, 2.0 s cap
  score 0.789998 / fill 0.923120; buckets 0.886977 / 0.837329 / 0.681764;
  300/300 rows, rc=0, no FAIL; run wall 284 s          [.scratch/iter87/slot-run.log, score.json]
cargo test --release -p ssi-candidate-worker
  126 passed / 0 failed                                [1 m 24 s]
target/probe-sandbox.sh run                # probe frame, one order()/row, SSI_PROBE_ONLY=<rows>
  0.789413 control · 0.789420 slot · 0.790171 reg-off · 0.795089 reduce-off · 0.789407 fork-n1200
python3 .scratch/iter87/cheap_probe.py 1000            # independent searcher, 0/148 beatable
```

## 6. Caveats, stated plainly

* This bat is **not a value device**. The harness-frame score is identical to the tree that died
  (0.789998 / 0.923120). What it changes is the state the three ledger consumers see and the shape
  of the near-cap trajectory — the same class of perturbation that moved the previous kill from
  81.5 s to 184.1 s.
* The probe frame is **not** the shipped frame: the same tree reads 0.789413 in the probe and
  0.789998 on the harness (per-bucket `gt_10k` 0.6803 vs 0.6817). Every A/B in this note is
  same-frame; cross-frame comparisons are flagged.
* The tree remains inside the near-cap regime: at least one hidden row sits within ~0.05 s of the
  cap for this profile, so a kill is a live outcome and is not evidence about value.
* Dev is not predictive of hidden for this lane: `0df9f508` was −2.14e-4 *better* than the crown on
  dev and +2.77e-4 *worse* on hidden.

## 7. Next steps, in order

1. If this bat completes, read the registration's hidden transfer against `3587d1b` (−7.8e-5): the
   registration is the only unmeasured 7.6-bip dev device in the tree, and 1e-4 is the whole bar.
2. If it is killed, the discriminator is already measured: the same tree with the registration
   dropped is the `3587d1b` profile (0.790171 probe, completed hidden at −7.8e-5) and the missing
   0.22 bip has to come from a device that is wall-neutral on the 1.0–1.3 s class.
3. The headroom map says the next admissible device form is a *cheaper implementation* of a
   value-carrying block (`1.portfolio` 17–58 %, `4.subtree` 13–34 %, `9.reduce` 12–18 % of the
   near-cap rows) — not another knob on the exchange, whose every dial has now been priced.
