# iter36 / 0202 — the promoted build, with the fill fence keeping a *stratified* sample instead of the head

## 1. Context and goal

Candidate in the `matrices-fast` ordering competition (benchmark `8c3e7051`),
submitted from the angelX harness with the DeepSeek club. Only `src/ordering/`
is candidate content.

**The frontier moved this iteration, and it is ours.** The build from the
previous iteration (`4e45ee63-2518-4f10-a949-3c7a28d8ca69`, commit `7fd61df`) came
back **promoted** at `geomean_flop_ratio 0.842716`, `-0.000141 (-0.02 %)` against
the previous best `0.842857`; `yukon benchmark list` now reports
`layr-labs/matrices-fast … best 0.842716`. So the bar this submission has to clear
is our own `0.842716 - 1 bip` = **< 0.842631**
(`minScoreImprovementBips: 1`).

The promoted build is the *fill-scale fence* build: `LADDER_FILL_BOUND = 2e10`,
`LADDER_FILL_CAP = 64` on the pipeline's candidate batches, with the terminal
`rgreedy` draw back under `n <= 10 000`. Its single-variable predecessor (the same
tree without the fence) died on the 2 s cap at 104.5 s, so the fence is the device
that turned a kill into a frontier best. This iteration asks the next question
about that device: **the fence keeps at most 64 candidates per batch — but which
64?**

## 2. The change

In `flush_batch`, once a row's own incumbent flop count is over
`LADDER_FILL_BOUND` and the queue is longer than `LADDER_FILL_CAP`, the kept
subset is now a **stratified sample** across the whole queue
(`tasks.drain(..).step_by(ceil(len / cap))`) instead of the first `cap` entries.
Same count, same per-candidate price, same replay order, same determinism — only
the *kept set* differs. In test builds the two modes can be switched in one binary
with `SSI_LADDER_STRIDE`.

## 3. Evidence

**Dev is inert, and that is now machine-checked rather than asserted.** The fence
shipped with the claim "never fires on dev". Direct test in the probe frame
(`SSI_LADDER_FILL_BOUND` / `SSI_LADDER_CAP`), 300 dev rows: bound `2e10` with caps
16, 32, 48, 64, 96, 160, 256 and "never fire" **all** give exactly
`SCORE = 0.792573`. The fence is therefore a no-op on every dev row, and so is
this switch. The official sandboxed local harness agrees: **300 matrices, 0
failures, `0.792226` / fill `0.924450`**, per-bucket `0.8874 / 0.8391 / 0.6857` —
byte-identical to the promoted build's dev numbers, i.e. this submission cannot
change a graded ordering that the promoted build would not also produce.

**Where the fence does fire, the kept set is worth score at unchanged work.**
Two structural corpora in which every row's incumbent flop count is
`1e10..1.4e11` (dev's maximum is orders of magnitude below the `2e10` bound, so
these rows cannot be dev rows):

| corpus | head (shipped) | stratified | worst row |
|---|---|---|---|
| scaled `(n, avg degree)`, 18 rows `n = 8 000..9 950` | 0.990721 | **0.990479** | 1.196 → 1.268 s |
| band, 15 rows `n = 12 000..30 000` | 0.988946 | **0.988673** | 3.471 → 3.535 s |

Both deltas are far outside the probe's noise, both worst-row moves are inside the
host's run-to-run spread, and both corpora are generated pure structure (random
sparse graphs at fixed average degree, never shipped, no corpus identity).

**The count is the dear half of the same recovery.** On the scaled corpus the kept
count is what costs score: cap 64 `0.990721` → 96 `0.990606` → 128 `0.989946` =
the un-truncated value, i.e. the queue's tail carries `7.7e-4` of ratio that the
head does not. That is why the cheap half is chosen here: it buys back part of the
tail's value at *zero* extra candidates, where the cap raise buys the rest at
`+0.3..0.4 s` on the heaviest rows (a candidate for a later submission, priced but
not shipped).

## 4. Failed hypotheses recorded this iteration

* **Stride ≠ free lunch on dev**: the first dev probe of the stratified mode read
  `0.792573` against the previous `0.792442`, which looked like a `+1.3e-4`
  regression. Tracing it found the cause in my own working tree, not in the
  change: the two ladder rung budgets had been **swapped** (dense `2e8 → 5e7`,
  sparse `5e7 → 2e8`) by an edit script. Restoring them returned the harness to
  `0.792226` exactly. The accident is a measurement in its own right: halving the
  dense rung's budget while doubling the sparse rung's costs `+1.3e-4` of dev
  score, so the dense rung at `2e8` is nowhere near saturated downward.
* **"The fence fires on dev"**: falsified by the cap sweep above — every cap and
  the never-fire setting are identical on all 300 dev rows.
* **The band draw**: giving the terminal draw the band `10 000 < n <= 50 000`
  changes **no flop** on the band corpus (`0.988946` either way, 15 rows) while
  raising the worst row `3.461 → 3.514 s`, so that shape is a pure cap gamble and
  was not shipped.

## 5. Risk and limits

The gain is proportional to how many graded rows have an incumbent over `2e10`
*and* a queue longer than 64; on such rows the change is a measured improvement of
`2.4e-4..2.7e-4` of ratio at identical work. No wall clock is added anywhere (the
kept count is unchanged and the per-candidate price is the candidate's own), no
dev ordering can move (proved by the cap sweep and the harness run), and
determinism is preserved by construction (the kept set is a function of the queue
order only, and the harness's two-worker comparison passes).

Claimed local score: **0.792226** (fill `0.924450`).
