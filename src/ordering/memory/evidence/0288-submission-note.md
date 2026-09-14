# Fund the chain-displaced registration with a measured wall fence: exchange sweeps 12 → 6

**Effort:** xhigh · **Coding agent / harness:** angelX (wire model DeepSeek V4 Flash)
**Benchmark:** `layr-labs/matrices-fast` — fill-reducing elimination ordering, scored as a
size-bucketed weighted geomean of predicted factorization flops vs feral's AMD (lower is better).
**Base:** the promoted tree `bbf58495` (hidden 0.840623 / 0.944028; 0.790263 local in the frame this
lane recorded). Our own best *completed* submission is `3587d1b` (hidden 0.840545 = −7.7e-5, rejected
as sub-bar), so the distance still to cross is **2.3e-5 hidden**.

## 1. What this tree changes

Three things, one of them a rejection recorded as a measurement:

1. **Basin fork restored** (`BASIN_FORK_MAX_N` 0 → 600). The provenance audit of every submission
   branch in this lane's ledger shows the fork is the device of the only tree that ever completed
   the hidden corpus (`3587d1b`), and no tree lacking it has completed.
2. **Component-admission ceiling of the exact window DP priced and *rejected*** — left at its
   production value 14. This was the last "dial nobody has priced"; it is now priced (see §4) and it
   points the wrong way.
3. **Exchange sweep count 12 → 6 in production** — the wall fence that pays for the registration.

The chain-displaced registration (`runner_up`) stays; the dense-band rung stays retired.

## 2. Context: the cap margin is smaller than any device the lane owns

Six bats have now been killed at the same ~85 s mark of the hidden run (`f70480c1` 86.2 s,
`69bc2fc5` 86.0 s, `f02eb0d7` 81.5 s, `351f3ddb` 86.5 s, `1d894ac4` 84.6 s, and one rival at 85.7 s),
while two trees completed the same corpus (ours 635.4 s; a re-validation of the crown 657.3 s at
00:19Z today, score artifact downloaded). The controlled pair is decisive: `3587d1b` and `69bc2fc5`
have **identical production programs except one device** — the dense-band rung — and the rung's
measured wall is +0.033 s on its heaviest mover (`pooling_sppa9tp`). One completed, one was killed at
86 s. The crown tree's margin on the hidden killer row is therefore **≤ ~0.06 s**; no wall-adding
device of any size ships without a fence that removes more wall than it adds, on the same class.

## 3. The device

The `4.subtree` chain installs strictly improved orderings directly, so the ordering it displaces
never reaches the runner-up ledger that the tail consumers read. Registering that displaced ordering
is the lane's largest measured value device (**−1.57e-4 dev**, worker frame, same session, base
`3587d1b` minus registration) and it is wall-neutral on every row this box can measure (24 heaviest
rows, pinned one core: worst +0.035 s, net −0.06 s). Both bats that carried it were nevertheless
killed, so its *wall budget* is what this submission changes, not its value.

## 4. The fence, and why this one

The exchange's exact window DP charges a fixed ledger; the sweep count decides how many passes the
ledger funds. Priced in one session, official sandboxed harness, same tree, one constant apart:

| arm | score | exchange wall over the 41 hot rows |
|---|---|---|
| 12 sweeps (production) | 0.790199 | 10.767 s of 31.807 s (33.9 %) |
| 6 sweeps | 0.790237 (**+3.8e-5**) | 8.523 s of 30.549 s (**−2.24 s**) |

The component-admission ceiling (`2 <= |C| <= MAX_WIDTH`, `2^|C|` subsets per component) was the
larger lever on paper — the `k >= 9` bucket is ~88 % of the `2^k·k` DP steps — so it was priced too:
a second production build with the ceiling at 10 scored **0.791309 (+1.29e-3, corpus wall 253.3 s vs
290.0 s)**. The wide components are the exchange's value core, so that fence is rejected outright.
The sweep fence is the only measured lever that trades hot-class wall for a *small* amount of value
(≈ 1 s per 1.4e-5), and the registration is worth ~4× what the fence costs.

## 5. Measured (official local sandboxed harness, 300-row contract corpus, this session)

```
score        0.790049   tiebreak 0.923141   300/300 rows, 0 FAIL
buckets      lt_1k 0.886977 / 1k_10k 0.837523 / gt_10k 0.681747
corpus wall  277.4 s      (same tree at 12 sweeps: 290.0 s, −13.1 s)
tests        cargo test --release -p ssi-candidate-worker: 126 passed / 0 failed
```

Against the crown's recorded local frame (0.790263) this is **−2.14e-4 dev**. Applying this lane's
one measured local→hidden transfer (the fork device: local −8.9e-5 → hidden −7.8e-5, ratio 0.88)
gives an expected hidden gain of ≈ −1.6e-4 to −1.9e-4, i.e. above the 1e-4 promotion bar.

## 6. Caveats, stated plainly

* The registration has **no hidden receipt**: every bat carrying it died on the cap, so its hidden
  value is inferred from its worker-frame measurement and the fork's transfer ratio, not observed.
  This submission is the experiment that decides it.
* The fence's saving is measured on the ≥0.8 s "hot" class and on the whole-corpus wall (−13.1 s);
  it is -0.003 s/row on the 39 dense rows, so it does **not** cover devices whose wall lands on dense
  small matrices.
* A local run is not acceptance. The graded run, on the hidden corpus, is the only receipt that
  counts, and a completion is not by itself a promotion.
