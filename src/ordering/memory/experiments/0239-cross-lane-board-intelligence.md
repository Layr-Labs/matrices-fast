# 0239 — cross-lane intelligence from the public board (PR #674 / #677, submission `039c8e2d` / `65cb40ec`)

Everything here is read off the **public board** (`yukon submissions`, `gh pr view … -R
Layr-Labs/matrices-fast`, `yukon submission-note <id>`), not from the hidden corpus. Treat it as
research data: it is another lane's evidence, quoted with its own frame labels. It is the cheapest
knowledge in this base — the board publishes every receipt anyone has ever paid for.

## 1. The remote cap line, bracketed (the number this lane never had)

| tree | worst local `order()` | remote |
|---|--:|---|
| frontier `43c1ca7d` (2 GiB + `MAX_N` 25 000) | **1.397 s** | **promoted**, hidden 0.840782 |
| `9440dedb` (4 GiB + six sweeps) | **1.536 s** | `RUN FAILED: hidden matrix: order() exceeded the 2.0s per-matrix cap` |
| `edd49e95` (4 GiB + five sweeps) | 1.473 s | same kill |
| `75117ca9` / `393a167c` / `b9549e8f` / `6f117752` (all 4 GiB) | — | all killed |

So a local worst row of 1.397 s passes and 1.536 s fails, and **the 4 GiB allowance is 0-for-7
remotely** (5 in that lane, 2 in this one: `c67ad490`, `52669543`). The 3 GiB rung had never been
submitted until `65cb40ec` (that lane) and `3ace1d4c` (this one, same hour).

That lane's graded-frame instrument (one `target/release/ssi-candidate-worker` run per dev row,
`taskset -c 0-3`, `/usr/bin/time`) gives the frame the grader actually charges:

| allowance | corpus wall | worst row |
|---|--:|---|
| family off | 114.4 s | 0.840 s |
| 2 GiB | 123.2 s | **1.010 s** |
| 3 GiB | 124.2 s | **1.100 s** |
| 4 GiB | 127.2 s | **1.250 s** |

## 2. The frame trap: the probe frame is NOT the graded program

`SSI_MARK_NOSCORE=1` is documented as the "graded-closest" probe frame, but a row-by-row diff against
the production binary on the same constants shows **170 of 300 rows differ** (several by >1e-3,
e.g. `gabriel09` 0.8695 vs 0.8970). Reason: `score()` writes the shared `ScoreWorkspace`, and the
pipeline reads `score_workspace.borrow().nnz_l()` as a *gate* (`nnz_l <= followup_factor`) — so the
extra scoring passes that the `#[cfg(test)]` phase marks pay **change which rows pass the class
block's inner key**. Consequence for this base: a per-row flop value read from a marked probe run is
a *different program* from the shipped one; deltas measured probe-vs-probe are still valid (same
program on both arms), but a probe value should never be compared row-by-row against a production
run, and any "the probe reproduces the shipped `order()` exactly" claim is false on the marked frame.

## 3. What is worth, measured by the other lane (their frames)

| device | Δ dev | note |
|---|--:|---|
| `MAX_N` 25 000 → 45 000 | −1.23e-4 | 4 movers; +0.015 s on the peak row; **passed remotely at hidden 0.840725 (0.57 bip — closed for being under the 1 bip bar)** |
| allowance 2 → 3 GiB | −8.2e-5 … −1.70e-4 | three rows (`crudeoil_lee4_10`, `_09`, `mpbp_48`) |
| allowance 2 → 4 GiB | −1.03e-4 | the two rows closest to the cap — lethal |
| one more exchange sweep (at 4 GiB) | −6.9e-5 per sweep | ≈ −1.15e-3 score per second of worst-row time; the whole tree is on that Pareto line |
| class `nnz` key 200 000 → 300 000 | −1.4e-5 | reaches exactly one dev row (`gams05`) for +0.44 s on it |
| 3 GiB + anchor gate (`best_flops < amd_flops` at the class block) | −1.70e-4 vs promoted | `65cb40ec`, submitted 19 s before `3ace1d4c` |

Their 0260 device is the **same mechanism this lane shipped independently as "one shared `Game` per
`order()`"**: `Game::build_adj` is a pure function of the pattern and the class block enters it 12–14
times per row; they memoize the pristine image across calls keyed on full CSR content, this lane
shares one image within the call. Two lanes converging on "the per-site kernel construction is the
only free time in this class" is the strongest available evidence that the class has no other free
time.

## 4. Consequences for this lane's plan

* Never spend the allowance past 3 GiB unless the extra work is *removed* elsewhere; the 4 GiB rung
  is the most thoroughly falsified device on the board.
* Any new device must be priced against the **1.397 s / 1.536 s bracket**, in a frame that resembles
  the graded child process — not against a marked probe run.
* The promotion bar is `minScoreImprovementBips = 1` on the **hidden** score: a dev device worth
  ~0.6 bip hidden is *closed*, not rejected — it consumes a slot and returns nothing. Stack devices
  until the expected hidden delta clears 1 bip.
