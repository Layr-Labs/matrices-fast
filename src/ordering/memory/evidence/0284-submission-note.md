# The chain's displaced incumbent, registered in the terminal donor/seed ledger

**Effort:** xhigh · **Coding agent / harness:** angelX (wire model DeepSeek V4 Flash)
**Benchmark:** `layr-labs/matrices-fast` — fill-reducing elimination ordering, scored as a
size-bucketed weighted geomean of predicted factorization flops vs feral's AMD (lower is better).
**Base:** the promoted tree `bbf58495` (hidden 0.840623). The previous bat (`351f3ddb`) was killed
on the per-matrix cap after 86.5 s.

## 1. What this tree changes

One new value device, plus one retirement:

1. **The ordering the subtree chain displaces is now registered in `runner_up`.** The `4.subtree`
   chain installs strictly improved orderings *directly*, so the ordering it displaces never reaches
   the runner-up ledger — the single ledger that three tail consumers read: the `13.alt` PEO_ALT
   seeds, the `14.transplant` donors, and the terminal exchange's seed pool (`xchg_pool`). The
   snapshot is taken just before the chain and pushed just after it, only when the chain actually
   installed something. Every consumer accepts only a strict exact decrease, so a registration can
   add value but cannot lower a row's own flops; the bounded seed pool can crowd one entry out, which
   is where its (small) regressions come from.
2. **The dense-band second ladder rung is retired** (production compiles it OFF). It is the one
   device the completed profile (`3587d1b`) did not carry, and the two bats that carried it were
   killed on the cap.

Removed relative to the previous bat: the work-band extension (`nnz <= 6 000`) and the head-light
second lineage. Both are measured value-positive in the probe frame (together −1.3e-4) and are
*not* part of this submission.

## 2. Measurements (this session, same box, same corpus)

**Official local sandboxed harness** (`bash scripts/local-candidate-build.sh && cargo run --release`,
300-row contract corpus, two `order()` calls per row in their own sandboxed child, 0 FAIL):

| tree | score.json | lt_1k | 1k_10k | gt_10k |
|---|---|---|---|---|
| promoted-profile baseline | 0.790263 | 0.887006 | 0.837339 | 0.682176 |
| previous bat `351f3ddb`'s tree (also had the band + head-light) | 0.789974 | 0.886977 | 0.837241 | 0.681771 |
| **this tree** | **0.790017** | 0.886977 | 0.837385 | **0.681771** |

Run wall 4 m 50 s; 300/300 rows scored; fill tiebreak 0.923134.
`cargo test --release -p ssi-candidate-worker --offline --locked`: **126 passed, 0 failed**.

**Probe frame (4-core, production seams, one session), decomposed arm by arm:**

| arm | score |
|---|---|
| A path, dense rung ON (tree before this pass) | 0.790234 |
| A path, rung retired, no registration | 0.790260 |
| A path, rung retired, **registration ON** | **0.789521** |
| full fork + band + head-light + registration | 0.789389 |

So the registration alone is **−7.39e-4** in the probe frame — the largest single device this lane
has measured (the basin fork was −8.9e-5). Its probe-frame wall is small: corpus wall 151.7 → 152.8 s
(+1.1 s) and the worst per-row delta is +0.146 s (`popdynm200`, 1.075 → 1.221 s); it moved the ratios
of 19–29 rows and the movers are large rows (`crudeoil_pooling_dt3` n = 30 660 0.7056 → 0.6449,
`arki0013` 0.3993 → 0.3919, `crudeoil_lee4_09` 0.6171 → 0.6100, plus `chp_partload`, `mpbp_15`,
`rsyn0815m04m`, `rsyn0840m04m`).

The registration is a **null on the cheap band**: on the 15 rows where the chain-off lineage beats
the shipped pipeline it changes no ratio at all (one row +0.06 %). Its value lives on rows the fork
never touches, so it is additive to the fork rather than a substitute for it.

## 3. Frame facts measured this pass

* The harness runs `order()` **twice** per matrix (`run_once("a")` / `run_once("b")`, byte-identical
  required) and the 2.0 s cap is the **child process's wall clock**, polled every 10 ms
  (`src/watchdog.rs::run_capped`, `src/main.rs`), with the *sandbox spawn* inside the same child
  budget — so the ordering budget per call is 2.0 s minus the sandbox/exec cost.
* `target/probe-sandbox.sh` omitted `SSI_SUBTREE_BUDGET_PCT` and (as written this pass)
  `SSI_NO_CHAIN_DONOR` from its seam whitelist; both A/Bs came back byte-identical until the seams
  were forwarded. Seam forwarding has to be verified before a seam-priced device is believed.
* `gasprod_sarawak16` (n = 4596, nnz = 15 316) sits **at** the 2 s line in the local harness frame:
  one harness run of this tree died on it at ≥ 2.0 s and an immediate re-run of the identical binary
  scored 300/300. Its work is identical in both runs, so the heavy-row margin in that frame is
  ≈ 0 and a local cap failure on it is (at least partly) load, not code.

## 4. Safety

All changes are structural (the pipeline's own incumbent, and a monotone density/`n`/`nnz`
predicate); no matrix identity, no clock, no environment reads; the ordering stays deterministic.
The registration is bounded by the same ledgers the consumers already use, and its measured wall is
+1.1 s corpus-wide in the probe frame with a worst row delta of +0.146 s. The tree carries strictly
less work than the profile whose hidden run completed, minus the retired rung.
