# Terminal engine ladder, structurally windowed — after the previous ladder was killed by the grader

## 1. Context and goal

The objective is a full-pipeline fill-reducing elimination ordering for sparse
symmetric indefinite matrices, scored as a size-bucketed weighted geomean of
predicted factorization flops against the `feral-AMD` baseline (lower is
better; buckets `lt_1k` / `1k_10k` / `gt_10k` at weights 0.30 / 0.30 / 0.40).
Only `src/ordering` is editable. The board's current best is **0.842857**
(submission `a9905f20`, commit `ab30c0e`, which is the tree this work starts
from) and `minScoreImprovementBips = 1`, so a submission must land at least one
basis point under that on the *hidden* corpus to be promoted. The hidden corpus
is not visible; every number below is measured either on the public 300-matrix
dev corpus or read back from the platform's own CI logs.

## 2. Environment and setup

- Host: shared 24-vCPU / 15 GB desktop ("Apollo"), `CARGO_BUILD_JOBS=2` by the
  workspace rules, load ~1.4 during the runs reported here.
- Measurement instruments, both in the repo: the test-only in-process probe
  (`cargo test --release -p ssi-candidate-worker -- --ignored --nocapture
  --test-threads=1 probe_timing_and_score`) which reports per-row `order()`
  wall time and the exact harness score, and the official sandboxed harness
  (`bash scripts/local-candidate-build.sh && cargo run --release`), which is
  the graded command: per-matrix 2 s SIGKILL cap, worker in a sandbox, worker
  run **twice** per matrix with bijection and determinism gates.
- Baseline re-measured here: frontier `ab30c0e` = dev **0.792436** (probe),
  worst in-process `order()` 1.085-1.131 s.

## 3. Baseline and the immediate predecessor

The frontier pipeline has a large stage chain (`1.portfolio` … `22.win`); its
worst dev rows are the big ones (`crudeoil_lee4_09` n=15904, `acopf_…_qcqp`
n=313068). The previous iteration (0166) appended a **terminal engine ladder**:
two 200M-operation `rgreedy::search` draws with distinct seeds, seeded from the
*finished* incumbent, gated on `n <= 12000`, accepting only a strictly better
exact flops count. That measured dev 0.792188 (-2.5 to -3.2 bips by instrument)
with 19 movers across all three buckets.

## 4. Failure and course correction (this iteration's starting point)

That submission (`c13df7a2-67e4-47e4-b011-93c9cff5d989`) came back
**`failed`**. The submission record's `rejectionReason` points at the grader
workflow run, and its job log ends with:

```
2026-09-12T06:57:24 … Benchmark step starts
2026-09-12T06:59:18.5743212Z RUN FAILED: hidden matrix: order() exceeded the 2.0s per-matrix cap and was killed
```

The step ran 114 s, while a recent *successful* submission's step on the same
workflow takes ~540 s — so the kill landed about a fifth of the way into the
hidden corpus and the added wall clock, not the score, is what lost the run.
The frontier is promoted, so its own worst hidden row is inside the cap; a
ladder that adds time to 263 rows can push such a row over. Two hypotheses
followed: (a) the added time is matrix-dependent (dense rows are expensive) and
can be gated by `nnz` as the harness's own error text suggests, or (b) the add
is a property of the *budget* alone, and only the number of draws per row can
bound it.

## 5. Measurements that settled the mechanism

From the 0166 CEN2 census (one 2e8 draw priced on all 263 dev rows with
`n <= 12000`):

| quantity | value |
|---|---|
| one 2e8 draw, mean / p90 / max added | 0.056 / 0.076 / **0.187 s** |
| the same on the densest rows (`nnz/n >= 12`) | mean **0.037 s** |
| corr(add, `nnz`) / corr(add, `n`) | **-0.37** / -0.10 |

So (b): the budget is a work counter and a draw costs what the budget says,
nearly independently of the matrix — an `nnz` gate cannot bound the ladder.
Combined with the per-row budget ladder (2e8 / 5e8 / 2e9 -> mean adds
0.056 / 0.198 / 0.763 s, 15 / 24 / 28 movers) the only lever that bounds the
cost is *how many draws a row is allowed*.

## 6. The mechanism I built, measured, and rejected: a wall-clock gate

`SSI_TERM_LIMITS=0.95,0.90`: run rung *k* only while the current call has spent
<= `LIMIT[k]` seconds, measured from an `Instant` taken at the top of
`leader_order` (one frame below `order()`, per call). Measured on the probe:

- **all 300 dev ratios byte-identical to the ungated build**, `SCORE = 0.792188`
  both ways, 48 of 526 draw decisions skipped, worst `order()` 1.131 s.

It is score-neutral *and* it removes the draws exactly from the rows that are
slowest on whatever machine is grading (self-calibrating, because it reads the
same clock the cap reads). It is **not shipped**: the harness runs every worker
twice per matrix and fails the run on any output difference, so a wall-clock
threshold is a predicate one run can straddle, trading a cap kill for a
nondeterminism kill. The project's own 0063 result — "time margin by structure"
— is the safe form of the same idea, so the shipped version windows the work by
`n` alone, which no clock can flip.

## 7. Shipped change

In `src/ordering/mod.rs`, the terminal ladder keeps its shape but its window
and budget become two deterministic, monotone tiers:

```
n <=  7 000        : two draws, 2e8 + 2e8   (seeds 0x9E37…, 0xD1B5…)
7 000 < n <= 12 000: one draw,  5e7         (seed 0x9E37…)
```

Value and risk of the boundary, simulated from the 0151 baseline / 0167
(2 x 2e8) / 0175 (budget ladder) logs, then reproduced **exactly** by the built
probe (0 of 300 rows differ):

| window | dev score | dev bips | slowest row *in* the window |
|---|---|---|---|
| `n <= 12 000`, 2 x 2e8 (the killed build) | 0.792186 | 3.16 | 0.951 s |
| **`n <= 7 000`, 2 x 2e8 + one 5e7 above** | **0.792199** | **3.02** | **0.829 s** |
| `n <= 6 000`, 2 x 2e8 | 0.792332 | 1.32 | 0.782 s |
| `n <= 12 000`, 2 x 5e7 | 0.792306 | 1.66 | — |
| `n <= 12 000`, one 2e8 draw | 0.792215 | 2.83 | — |

The two 2e8 draws on the widest 17 in-window rows are worth 0.14 bips — 4 % of
the ladder — and those are exactly the rows the pipeline already spends the most
time on (0.95 s vs the 0.83 s window worst), so they are cut to one 5e7 draw, a
0.015 s-class add. Acceptance stays strict against the exact flops count and
every candidate permutation is bijection-checked, so no row can regress.
222 rows take the full ladder, 17 the light one. The predicate uses `n` only:
no matrix identity, no fingerprinting, no clock, no environment read in the
shipped path.

## 8. Exact commands and measured results

```
cargo test --release -p ssi-candidate-worker -- --ignored --nocapture --test-threads=1 probe_timing_and_score
  -> SCORE = 0.792199 (lt_1k 0.8874 / 1k_10k 0.8391 / gt_10k 0.6857)
     WORST order() = 1.143 s, 300 rows
bash scripts/local-candidate-build.sh && cargo run --release -- --note "…"
  -> 300/300 rows OK, score 0.792199, fill tiebreak 0.924450, 293 s wall
     (results.tsv row 1789198707), score.json identical to the probe's number
```

The official sandboxed harness completing is itself evidence: every previous
local attempt this session — including 3/3 runs of the *unchanged frontier* —
was killed at the cap on this host at rows whose in-process time is 0.2-0.8 s,
so the local host produces random cap kills; this build passed all 300 rows
with the same 2 s cap and the same twice-per-row determinism gates.

## 9. Caveats

- The kill happened on the hidden corpus, which is not visible here. The window
  reduces the added time on the rows this box measures as slowest, but a hidden
  row can still surprise, and the graded host may be slower than this box.
- A window on `n` is blind to a small *dense* row's cost (`qspp_0_14_0_1_10_1`,
  n=560, nnz=120274, takes the full ladder and its draws cost ~0.1 s each). An
  `nnz` bound was measured to be nearly value-free (<=0.2 bips) but no density
  effect on draw cost was measurable, so it was left out to keep the predicate
  single and monotone.
- Draw-cost numbers are this box's; the graded runner's per-unit cost is not
  measurable from here.

## 10. Learning and next steps

On this pipeline, anything appended after the last improvement is priced by
*budget*, not by matrix shape, and the binding constraint at the top of the
board is the 2 s cap rather than search quality: the grader killed a 3-bip
score gain outright. Next steps, in order: (1) read the remote verdict on this
submission; (2) if it survives but the hidden delta is under one bip, raise the
budget of the wide tier before widening the window; (3) if it is killed again,
shrink the window to `n <= 6000` class and re-measure, since that is the only
dial with a measured score/time curve.
