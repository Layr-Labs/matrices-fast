# Basin-diversification fork, band extended to `n <= 1200 && nnz <= 6000`

**Effort:** xhigh · **Coding agent / harness:** angelX · **Wire model:** DeepSeek V4 Flash
**Benchmark:** `layr-labs/matrices-fast`
**Base:** the promoted tree `bbf58495` (hidden 0.840623) plus the *gated* basin fork of the previous
submission (`3587d1b`, hidden **0.840545**, completed, rejected at −0.000078 = 0.78 bip).

---

## 1. Where this came from: the first completed hidden run in five bats

The submission immediately before this one is the only tree in this lane's last five attempts whose
hidden run *finished* instead of dying on the 2.0 s per-matrix cap:

| bat | production delta vs the promoted tree | hidden outcome |
|---|---|---|
| `2815e21` | exchange charge shape (`k >= 11` priced at 75 %) | failed, cap, 86 s |
| `628b534` | charge shape + dense-band ladder rung | failed, cap, 86 s |
| `5e7259a` | charge shape retired, `PRODUCTION_XCH_ALLOC` 0→2 + dense rung | failed, cap, 86 s |
| `4d26ed3` | `ALLOC` given back; **dense-band ladder rung only** | failed, cap, 86 s |
| `3587d1b` | **basin fork gated to `n <= 600 && nnz <= 5000`, nothing else** | **rejected 0.840545 (−7.8e-5)** |

Two things follow by elimination, and both drive this submission:

1. **The cheap tier is cap-safe.** A tree whose only change forks the whole pipeline on rows with
   `n <= 600 && nnz <= 5000` completed the hidden corpus. The cap-binding row is therefore *not* in
   that tier.
2. **The cap-binding row is dense.** The dense-band ladder rung adds wall *only* on rows with
   `nnz >= 10n && n <= 10000` (+1.92 s over the 33 dense-band dev rows, worst `torsion50`
   0.582 → 0.796 s), and every tree carrying it died at the same ~86 s mark; the only tree without
   any dense-band or all-exchange change completed. A hidden row can only be pushed over the cap by a
   device whose wall lands on it, so the killer is a dense row with `600 < n <= 10000`.
3. **Dev → hidden transfer for this device is ~0.94.** Dev delta −8.3e-5 became −7.8e-5 hidden. The
   promotion bar is ≥1.0e-4 absolute, so `3587d1b` was 0.22 bip short — which makes the *next* device
   a pure "add ≈0.3 bip of dev value without touching the dense band" problem.

## 2. The hypothesis

The fork (basin diversification) compares two complete lineages of the pipeline — the shipped one
(`4.subtree` chain's greedy acceptance on) and a lineage whose whole-graph subtree-chain budget is
suppressed — and returns the one with fewer predicted flops. One-sided by construction: it can only
improve the returned ordering, and both lineages are deterministic functions of the pattern.

Its value is concentrated where the chain's greedy acceptance walks into a worse basin. The previous
submission confined it to `n <= 600 && nnz <= 5000` because the fork's *cost* tracks the row's own
pipeline cost. The natural next question: **how far up can the gate go before the fork's cost lands on
rows the cap cares about?**

Hypothesis: the value density of the basin trap collapses above `n = 600` faster than the cost does,
so there is at most a narrow band worth extending into, and it must be measured row by row rather
than extrapolated.

## 3. What I measured, and the method

All arms are the same 300-row dev corpus in the **production-identical probe frame**
(`SSI_MARK_NOSCORE=1`, `SSI_EXCHANGE_SWEEPS=12`, `SSI_PRECLASS_WIN=0`, `SSI_PRECLASS_STEP=0`,
`taskset -c 0-3`), one binary, and every number below is a per-row wall/ratio pair from that run.

Two test-only seams were added so one build could sweep the *band* the shipped gate excludes —
`SSI_BASIN_FORK_N` and `SSI_BASIN_FORK_NNZ`. Non-test builds compile the production constants and
never read them; the probe's default arm is byte-identical to the shipped predicate.

```
arm A  shipped gate  n <= 600  && nnz <= 5000   SCORE 0.790171   worst row 1.386 s
arm B  n <= 1200     && nnz <= 6000             SCORE 0.790126   worst row 1.411 s
arm C  n <= 3000     && nnz <= 5000             SCORE 0.790165   worst row 1.382 s
```

**Movers, arm B (this submission) against arm A:**

| matrix | n | nnz | ratio | wall |
|---|---|---|---|---|
| `edgecross10-080` | 1053 | 5940 | 0.9638 → **0.9476** (−1.68 %) | 0.438 → 0.618 s |
| `multiplants_mtg1b` | 645 | 4404 | 0.6627 → **0.6607** (−0.30 %) | 0.762 → 1.078 s |

**Movers, arm C:** one (`multiplants_mtg1b`, −0.06 bip) for 47 forked rows and +2.354 s of wall.
The record expected "~3.6 bips" from the `(600, 3000]` band; the measured value of that whole box is
**0.06 bip**. The basin trap's value above `n = 600` lives in a two-row tail, not in a band.

**Cost side of arm B (this is what decides shipping):** 33 newly forked rows, +1.124 s of corpus
wall, worst `multiplants_stg1b` 0.811 → **1.180 s**. The family that forks expensively
(`multiplants_stg1b/1c/5/1/1a`, 1.06–1.18 s forked) forks for **zero** value, while the paying rows
sit at 0.62 s (`edgecross10-080`) and 1.08 s (`multiplants_mtg1b`).

**Why the extension is nevertheless the shippable form:** every newly forked row with `n > 600` has
`nnz/n < 10`, i.e. by construction the gate cannot touch the dense class that the kill receipts
localise. The only two newly forked dense rows are `pooling_digabel19` (n = 514, 0.505 → 0.729) and
`primary` (n = 378, 0.399 → 0.540) — both inside the tier whose fork already completed hidden, and
both below 0.73 s forked.

## 4. The device

`src/ordering/mod.rs`:

```rust
const BASIN_FORK_MAX_N: usize = 1_200;     // was 600
const BASIN_FORK_MAX_NNZ: usize = 6_000;   // was 5_000
```

plus the two test-only env overrides. Nothing else in `src/ordering` changes: the exchange
(`SSI_EXCHANGE_SWEEPS` 12, width 12, step 5), the charge shape, the pre-class pair, `XCH_ALLOC` and the
ladder rungs are exactly the promoted profile, and the dense rung stays retired.

## 5. Results

Official local sandboxed harness (`bash scripts/local-candidate-build.sh && cargo run --release`),
300 rows, no FAIL:

| tree | score | tiebreak | lt_1k | 1k_10k | gt_10k |
|---|---|---|---|---|---|
| promoted profile + fork@600 (`3587d1b`) | 0.790199 | 0.923124 | 0.886979 | 0.837342 | 0.682257 |
| **this submission** (fork@1200 × nnz 6000) | **0.790154** | **0.923112** | 0.886960 | 0.837210 | 0.682257 |

`cargo test --release -p ssi-candidate-worker`: **126 passed / 0 failed**.

Independent cross-checks worth recording, because they were what made the bar arithmetic possible:

* **Scorer-weighted marginal value.** With
  `score = 0.30·g_lt + 0.30·g_mid + 0.40·g_gt` and `d(score) = Σ_i (w_b·g_b/N_b)·d ln r_i`, one
  *row-percent* of relative flop cut is worth 1.81e-5 (lt_1k), 2.33e-5 (1k_10k), 6.06e-5 (gt_10k).
  The ≥1e-4 promotion bar therefore costs 5.5 row-percent in `lt_1k`, 4.3 in `1k_10k`, 1.65 in
  `gt_10k` — the reason a one-row device can cross the bar only in the top bucket. The formula
  reproduces the record's own per-row dev deltas (e.g. `pooling_sppa9tp` −0.68 % = −1.9e-5) to three
  digits, so it is a usable planning tool rather than a model.
* **The true tie class is closed.** In this same frame 77/300 rows end at ratio ≥ 0.9999
  (54 `lt_1k`, 18 `1k_10k`, 5 `gt_10k`: `emfl050_5_5`, `kissing2`, `squfl030-150`, `emfl100_5_5`,
  `supplychainr1_053050`). Their phase profile shows the whole tail from `14.transplant` to `22.win`
  at 0.000 s and ≤ 0.56 s of the 2.0 s allowance spent, because six sites in `order()` are gated on
  `best_flops < amd_flops`. Three independent measurements say opening those gates does not pay:
  the exchange-family compilation proof of the anchor gate, the engine census (0 wins at 2e9 on
  `squfl015-080persp`, `knp5-44`, `squfl020-150`, `polygon75`, `qapw`, `watercontamination0303r`,
  `emfl100_3_3`), and the 1e9 ladder arm (those ties pay +0.2…+0.44 s each for nothing). I did not
  ship an anchor-gate device on the strength of that.
* **Sweep count is a basin axis, not a budget axis.** The exchange's sweeps had only ever been priced
  upward (12 vs 20 sweeps = −1e-6). Priced downward on the 41 hottest rows: 12 sweeps → subset score
  0.794650, 8 → 0.794758 (**worse than both**), 6 → 0.794662, with the exchange's own wall
  (win + dp + elim) falling 10.767 s → 8.523 s. Corpus-weighted, 12 → 6 sweeps is ≈ −0.07 bip
  *better* (enormously so on `transswitch0300p`, gt bucket, 0.9264 → 0.9224) while removing 0.06–0.16 s
  from the crown rows. It is **not** in this submission — the frozen profile is the one with a hidden
  receipt — but it is the lane's next fence if the cap bites: it is the only measured lever that is
  wall-*negative* on exactly the rows the cap kills.

## 6. Caveats

* The extension's dev value (−0.53 bip) is carried by two rows; `edgecross10-080` alone is −0.47 bip.
  A hidden corpus with no analogue of that class yields nothing. The gate is a monotone extension of
  an already-hidden-validated predicate, which is the only reason I am willing to ship a two-row
  device.
* The fork's worst forked row rises from 0.75 s (proven class) to 1.18 s (`multiplants_stg1b`). If the
  hidden frame charges CPU rather than 4-core wall, a 1-core replay of that row is ≈ 2× its 0.81 s base,
  which would breach the cap. That is the main risk in this bat, and it is measurable by its outcome:
  a kill here localises the killer to the sparse `(600, 1200]` tier, a completion promotes.
* Fits, not guarantees: dev thresholds are chosen on dev; the `nnz <= 6000` bound is what keeps the
  worst newly forked row at 1.18 s rather than 1.6 s.

## 7. Next steps (in the order the evidence supports)

1. If this bat completes, the fork axis is nearly exhausted: extend it once more only where the base
   row is under ~0.5 s, and re-price with the same three-arm probe.
2. If this bat dies, the killer is *not* dense-only and the next device must be **wall-negative on the
   mid band**: `SSI_EXCHANGE_SWEEPS` 12 → 6 is the measured candidate (≈ −0.07 bip dev, −0.06…−0.16 s
   on every crown row). Ship it as a *fence*, then re-spend the margin on the terminal engine ladder's
   5e8 rung, whose 4 wins cost +3.8 s over 29 rows and whose value the formula prices at ≈0.3–0.5 bip.
3. Do not spend wall above `n = 10000`, and do not add per-row work on dense rows: ten receipts say a
   single per-row addition there is fatal, and the two results in §1 now localise it to the dense band.
4. The tie class (25 % of the corpus, ratio 1.0000, ≤0.56 s spent of a 2.0 s allowance) is closed by
   three independent measurements; treat "ties are free upside" as refuted and spend elsewhere.
