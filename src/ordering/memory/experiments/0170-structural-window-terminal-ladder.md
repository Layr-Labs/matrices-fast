# 0170 — Terminal `rgreedy` ladder, structurally windowed after a remote cap kill

- **Date:** 2026-09-12 (iter20)
- **Base:** `ab30c0e` (remote-promoted `a9905f20`, hidden **0.842857**) + the
  terminal ladder of [0166](0166-terminal-engine-ladder.md) (submitted as
  `c13df7a2`, **failed** remotely).
- **Measured result:** dev **0.792199** (`lt_1k 0.8874 / 1k_10k 0.8391 /
  gt_10k 0.6857`, fill tiebreak **0.924450**) = **3.02 bips** better than the
  frontier's dev **0.792436**; probe and official sandboxed harness agree to
  2e-6. Worst in-process `order()` **1.143 s**; worst in-window added time
  ≲0.19 s.
- **Status:** submitted (`iter20`), local official harness **300/300 OK**.

## 1. The predecessor's remote verdict (new evidence)

`c13df7a2-67e4-47e4-b011-93c9cff5d989` (the ungated 2 × 2e8 ladder over every
`n <= 12 000` row) reported `status = failed`, `rejectionReason = workflow run
concluded failure at step "Benchmark"`, and the grader job's own log
(`gh run view 34679317718`) ends with:

```
2026-09-12T06:59:18.5743212Z RUN FAILED: hidden matrix: order() exceeded the 2.0s per-matrix cap and was killed
```

The benchmark step ran 06:57:24 → 06:59:18 (114 s) out of the ~540 s a
*successful* recent submission's step takes on the same workflow, so the kill
came ~21 % into the hidden corpus — the candidate's added time is what killed
it, not a build/purity/determinism defect. This is the first hard datum tying
the ladder's added wall clock to a lost submission, and it supersedes the
"this host's sandbox factor is local" reasoning of 0166 §5: the graded runner
kills too, at ~0.11 s mean / ~0.20 s worst added per in-window row.

## 2. What a draw costs, measured (`0166` CEN2, all 263 in-gate dev rows)

| quantity | value |
|---|---|
| one 2e8 draw, mean / p90 / max added | 0.056 / 0.076 / **0.187 s** |
| same, densest rows (`nnz/n >= 12`) | mean **0.037 s** |
| corr(`nnz`, add) / corr(`n`, add) | **-0.37** / -0.10 |

The budget is a work counter: a draw costs the same on a 400 k-nnz row as on a
1 k-nnz row. So "gate by nnz" (the harness's generic hint) cannot bound the
ladder's cost, and the only thing that can is *how many draws a row gets*.

## 3. The gate that was measured but NOT shipped

`SSI_TERM_LIMITS=0.95,0.90` in the 0172 build: run rung *k* only while the
current call has spent `<= LIMIT[k]` seconds (`t_pipeline`, started one frame
below `order()`). Measured: **all 300 dev ratios byte-identical to the ungated
build**, `SCORE = 0.792188` both ways, 48 of 526 draw decisions skipped, worst
`order()` 1.131 s. Score-neutral *and* it removes the draws from the rows that
are slowest on whatever machine is grading.

Not shipped because the harness runs **every worker twice per matrix** and
fails the run when the two permutations differ: a wall-clock threshold is a
predicate a row can straddle between the two back-to-back runs, which would
trade a cap kill for a nondeterminism kill. The project's own 0063 result
("time margin by structure") is the safe form of the same idea.

## 4. The shipped window (deterministic, monotone in `n`)

```
n <=  7 000        : two draws, 2e8 + 2e8   (seeds 0x9E37…, 0xD1B5…)
7 000 < n <= 12 000: one draw,  5e7         (seed 0x9E37…)
```

Value and risk of the boundary, simulated from the 0151 baseline, 0167
(2 × 2e8) and 0175 (budget ladder) logs, then reproduced exactly by the built
probe (0 rows differ):

| window | dev bips | slowest row *in* the window | rows dropped from 2 × 2e8 |
|---|---|---|---|
| `n <= 12 000`, 2 × 2e8 (killed build) | 3.16 | 0.951 s (`crudeoil_lee4_06`) | 0 |
| **`n <= 7 000`, 2 × 2e8 + 5e7 above** | **3.02** | **0.829 s** (`crudeoil_lee2_06`) | 17 |
| `n <= 6 000` | 1.32 | 0.782 s | 23 |
| `n <= 12 000`, 2 × 5e7 | 1.66 | — | — |

The two 2e8 draws on the 17 widest in-gate rows are worth 0.14 of those bips
(4 % of the ladder) and are exactly the rows the pipeline already spends the
most wall clock on (0.95 s vs a 0.83 s window worst), so the widest tier is
cut to a single 5e7 draw — a 0.015 s-class add. Board: 222 rows take the full
ladder, 17 the light one, and the acceptance rule is unchanged (strictly
better exact flops, `is_bijection` checked), so no row can regress.

## 5. Official sandboxed harness: first full local pass on this host

```
1789198707  OK  0.792199  0.924450  iter20 structural-window ladder …
```

300/300 rows, bijection + twice-per-row determinism gates, 2 s cap enforced,
**293 s** wall, `score.json` = the probe's number. All previous attempts this
session (and 3/3 of the frozen frontier's) were killed at the cap on this host
at 0.2-0.8 s in-process rows; this one was not, on a quiet box (load ~1.4).
That is still a *local* datum — the remote verdict is the only acceptance.

## 6. Open

- The hidden corpus is where the 114 s kill happened; the same window may still
  be too generous there if the graded host is slower than ~1.5× this box.
- A window on `n` is blind to a small dense row's cost: `qspp_0_14_0_1_10_1`
  (n=560, nnz=120274) takes the full ladder and its draws cost ~0.1 s each.
  Adding an `nnz` bound was measured to be value-free (≤0.2 bips) but no
  density effect on draw cost was measurable, so it was left out.
