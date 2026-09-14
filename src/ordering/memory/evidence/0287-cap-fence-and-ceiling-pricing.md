# iter85 — the component-admission ceiling priced (and rejected); the sweep fence as the registration's wall budget

All numbers below are from this session's own runs, one tree per build, official sandboxed
harness (`cargo run --release`, 300/300 dev rows, two sandboxed `order()` calls per row).

## 1. The bat in flight at the start of this iteration — resolved

Bat `8c3e7051` / submission `1d894ac4` (iter84: basin fork retired, chain-displaced
registration kept, dense rung retired) was **killed on the per-matrix cap**:
benchmark step 01:43:23.550Z → 01:44:48.106Z = **84.56 s**, verdict
`RUN FAILED: hidden matrix: order() exceeded the 2.0s per-matrix cap and was killed`
[.scratch/iter85/ours-1d894ac4.log:1116,1130]. It is the sixth kill at the ~85 s mark.

## 2. The board's own ledger as a control frame (new)

`gh run list --workflow benchmark` over the last 60 runs gives the pass/kill provenance of every
submission, and the score artifact of a completing run is downloadable:

* submission `6864d7bd` (validated 2026-09-14T00:08:47.774 → 00:19:45.037Z = **657.26 s**,
  hidden path) scored **0.840622 / 0.944028** — i.e. the hidden corpus is *passable* today, minutes
  before and after our own kills (.scratch/iter85/succ-job-6864d7bd.log:1105,1116;
  .scratch/iter85/score-6864d7bd/score.json).
* our only completing bat `3587d1b` (fork@600, no registration, 635.36 s) scored
  **0.840545 / 0.944002** (.scratch/iter85/score-3587d1b/score.json) = **−7.7e-5 vs the crown**,
  which is the sub-bar rejection the lane recorded. The gap left to cross the 1e-4 promotion bar
  is therefore only **2.3e-5 hidden**.

So the kills are device-induced, not environmental: same corpus, same day, a different program
completes it in 657 s.

## 3. Production-program provenance of every bat (killed vs completed)

Each submission branch's `src/ordering/mod.rs` was fetched and its production constants read
(.scratch/iter85/bat-shas.txt):

| bat | basin fork band | dense rung (production) | registration | hidden |
|---|---|---|---|---|
| `3587d1b` | 600 / 5 000 | OFF | absent | **COMPLETED** 635.4 s, 0.840545 |
| `f70480c1` | 1 200 / 6 000 | OFF | absent | killed 86.2 s |
| `69bc2fc5` | 600 / 5 000 | **ON** (3 draws) | absent | killed 86.0 s |
| `f02eb0d7` | 600 / 5 000 | OFF | ON | killed 81.5 s |
| `351f3ddb` | 600 / 5 000 | OFF | ON (ungated) | killed 86.5 s |
| `1d894ac4` | **0 (off)** | OFF | ON | killed 84.6 s |

The decisive pair is `3587d1b` vs `69bc2fc5`: their production programs differ by **one** device
(the dense rung) and one completed while the other was killed at 86 s. The rung's measured wall is
+0.033 s on its heaviest mover (`pooling_sppa9tp`), so the crown tree's margin on the hidden killer
row is **≤ ~0.06 s** (pinned-frame scale) — smaller than any wall-adding device the lane owns. That
is why every device since the completion has died, including ones that *remove* wall elsewhere.

## 4. The unpriced dial, priced: `MAX_WIDTH` (component admission of the exact window DP)

`refine_window` admits a live component only when `2 <= |C| <= ceiling` and `solve_component`
enumerates all `2^|C|` subsets, so the cost scales with `2^k` over admitted components. On the
cap-critical rows the `k >= 9` bucket is 8-12 % of the component *calls* and ~88 % of the `2^k·k`
DP steps (source comment), which made it the largest untried wall lever on the killed row class.

Two production builds, same tree (fork@600 + registration), one constant apart:

| component ceiling | score | buckets (lt / mid / gt) | corpus wall |
|---|---|---|---|
| 14 (production) | **0.790017** | 0.8873 / 0.8374 / 0.6818 | 290 s (`f02eb0d7`) |
| 10 | **0.791309** | 0.8870 / 0.8385 / 0.6842 | **253.3 s** |

[.scratch/iter85/w10-run.log, .scratch/iter85/w10-build2.log]

**Verdict: rejected.** The cut buys −0.12 s/row of corpus wall but costs **+1.29e-3 dev**, 12× the
whole 1e-4 promotion bar, concentrated in `gt_10k` (`0.6818 → 0.6842`). The wide components are not
the DP's waste; they are its value core. The dial is now priced, and it cannot fund the cap.

Two implementation notes that matter for anyone re-deriving this:
* `MAX_WIDTH` is *also* the validation bound of `subset_window_descent[_step]`
  (`2..=max_span` must admit the production `exchange_width = 12` call). Lowering `MAX_WIDTH`
  itself makes every production exchange call return `None` — the pre-existing test
  `sparse_spans_match_original_descent_within_exact_width_limit` catches it (it failed; a separate
  `PRODUCTION_XCH_MAX_K` admission const is the correct seam, and the test-only env default must
  equal the production value, not `MAX_WIDTH`).
* The reserve dial is *not* a wall lever either: the source records 50 % reserve as bit-identical in
  score but **+3.1 s of corpus wall**, because waster rows release the reserve and spend it.

## 5. The fence that is admissible: exchange sweep count 12 → 6

Measured in the worker frame (iter81, same-session official harness): 6 sweeps = 0.790237 vs
12 sweeps = 0.790199, i.e. **+3.8e-5 value**; the exchange's own wall over the 41 hot rows drops
10.767 s → 8.523 s (**−0.055 s/row**). The two fences trade roughly 1 s of hot-class wall per
1.4e-5 of value; the ceiling trades 1 s per 1.1e-4 and is out.

## 6. This bat

`src/ordering/mod.rs` + `src/ordering/rgreedy/window_dp.rs`:
* `BASIN_FORK_MAX_N`: 0 → **600** (the only tree that ever completed the hidden corpus carried it).
* `exchange_sweeps`: 12 → **6** (the wall fence above; `exchange_step = 5`, `exchange_width = 12`
  unchanged).
* the chain-displaced registration stays; the dense rung stays retired; the ceiling stays 14.

**Official local sandboxed harness, this session: score 0.790049, tiebreak 0.923141, 300/300 rows,
0 FAIL, corpus wall 277.4 s**; `cargo test --release -p ssi-candidate-worker` 126 passed / 0 failed
[.scratch/iter85/fence-run.log, .scratch/iter85/fence-build.log].

Against the crown's local frame (0.790263, recorded in `0285-submission-note.md`) that is
**−2.14e-4 dev**; against this lane's measured transfer ratio for the fork device (local −8.9e-5 →
hidden −7.8e-5, 0.88) the expected hidden gain is ≈ −1.6e-4 to −1.9e-4, which clears the 1e-4 bar
with ~60-90 % headroom.

**Honest caveat.** The registration has never had a *hidden* receipt — every bat carrying it died on
the cap, so its hidden value is inferred, not measured. What is new here is that its wall is now
budgeted on the class the cap kills: the fence is −0.055 s/row on the ≥0.8 s rows and −13.1 s over
the whole corpus (290.0 s → 277.4 s) versus the same tree at 12 sweeps, while the registration's own
measured deltas are ±0.035 s/row on the 24 heaviest dev rows.
