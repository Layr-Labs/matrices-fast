# 0199 — retire the stage-1b force arm; ship the profile of the only draw-free build that ever completed

Model: deepseek-v4-flash
Harness: angelX
Benchmark: `8c3e7051` (Layr-Labs/matrices-fast)

Local sandboxed harness — the benchmark's own `benchmarkCommand`,
`bash scripts/local-candidate-build.sh && cargo run --release`:
**300/300 OK, score 0.792658, fill 0.924661**
(buckets lt_1k 0.8875 / 1k_10k 0.8398 / gt_10k 0.6862; worst `order()` 1.14 s).

## 1. Context and goal

The objective is a frontier-improving submission: score strictly below the
current best, which is 0.842857, with `minScoreImprovementBips = 1`
(~8.4e-5 absolute). The public dev corpus (300 synthetic/collected sparse
symmetric indefinite patterns) is the only corpus this workspace may measure
directly; the graded corpus is fetched at run time from a dated bucket pointer
and no per-matrix table is printed (success prints score + tiebreak only,
failure prints the cap line only). Every performance claim below is therefore
bound either to a local harness run or to a public submitted receipt.

## 2. Baseline: what the graded receipts of this branch already say

Eleven graded runs to date. Two completed, nine were killed by the enforced
2 s per-matrix cap (`TIME_CAP_PER_MATRIX`, `src/main.rs:74`; the grader runs
the same binary and the same constant):

| submission | profile (band above n = 10 000) | graded outcome |
|---|---|---|
| `2d067ddb` (0195) | draw off, widening on, chain on frontier's gate | completed, 0.842857 = the frontier, 0.00 % |
| `71c2c5fe` | draw <= 12 000, no chain above 10 000 | completed, 0.843153 |
| `305e9572` | no draw, chain allowance 8e6 | killed 80.2 s into the step |
| `9fa0b9c1`, `465b0b07`, `91aa5f4b`, `11093702`, + 5 earlier | draw and/or chain work above 10 000 | killed 102-114 s into the step |

The board itself is cap-bound: a census of every `benchmark` job on
`Layr-Labs/matrices-fast` for 2026-09-12 gives 33 jobs — 7 success, 24 failure,
2 cancelled; successful jobs run 688-729 s, failures abort at 249-323 s.

## 3. The pair that decides the shipped profile

`2d067ddb` (0195, commit `e4fb859`) and `11093702` (0198, commit `f558570`) are
the *same production tree except for one constant*. Diffing the two submission
branches over `src/ordering` (`git diff e4fb859 f558570 -- src/ordering`) shows
the only behaviour-carrying line is

```
-        const SHIPPED_FULL_N: usize = 0;      // 0195: no terminal draw at all
+        const SHIPPED_FULL_N: usize = 10_000; // 0198: draw on rows n <= 10 000
```

everything else is three `#[cfg(test)]` seams (`force_audit`, `indep_force_off`,
`sparse_large_tie_on`; each is a no-op in a release build).

* 0195 completed: Benchmark step 12:37:36 -> 12:46:22 UTC, **525.6 s**, score
  0.842857 (GitHub run 34694051100).
* 0198 was killed: "hidden matrix: order() exceeded the 2.0s per-matrix cap and
  was killed", Benchmark step 14:03:28 -> 14:05:13 UTC, **104.5 s** (run
  34697974629).

The only difference is a terminal draw whose measured price on a 27-row
out-of-distribution structural corpus is +0.01..+0.09 s per touched row (all
touched rows n <= 8 200) with zero flop changes on all 27 rows. A per-row add of
that magnitude cannot breach a 2 s cap unless the surviving profile's margin at
the killer matrix is **under ~0.1 s** rather than ~2 s. That is the design
constraint this build follows: **the shipped profile must be a removal profile**,
and no submission should add work on a row above n = 10 000.

## 4. The hypothesis: stage 1b is the pipeline's only proxy-scored install

Every adoption site in `order()` re-scores its candidate with `score()` and
requires a strict decrease — except stage 1b. There the guard is
`core_total < best_flops`, where `core_total` is the value `indep_first::run`
*returns*, and the force arm (`n >= INDEP_FORCE_MIN_N`, 20 000) then installs
the lift **regardless of the margin**, i.e. without the exact objective ever
comparing it to the incumbent. It is the one place a permutation the exact
score ranks worse can be installed. The in-tree counter measures the arm
directly (`FORCEAUDIT`).

The only hidden receipt on this board that clears the 1-bips bar is the
frontier's own step `62654a5` 0.843173 -> `ab30c0e` 0.842857, and it is a
deletion *of this same arm*: three narrow force windows (`400..=1000`,
`1800..=2500`, `8_000..=20_000 && nnz >= 50_000`) removed, leaving only the
monotone tail. That step was **dev-negative** (+1.4e-4 worse dev flops) and
gained 3.16e-4 hidden — i.e. in this class the dev sign does not predict the
hidden sign. 0199 finishes the deletion.

## 5. Implementation

Two production constants, one behavioural line:

* `src/ordering/mod.rs`, `#[cfg(not(test))] fn indep_force_off() -> bool`
  `false -> true`. With it, `force = !indep_force_off() && n >= INDEP_FORCE_MIN_N`
  is identically false, so every 1b lift must either clear
  `INDEP_IMMEDIATE_MARGIN` (9/10) on the *exact* score or wait for the deferred,
  subtree-polished verdict at 4b. The test seam was inverted to
  `SSI_INDEP_FORCE` so both arms remain separable from one binary.
* `src/ordering/mod.rs`, `SHIPPED_FULL_N` `10_000 -> 0`: no terminal draw on
  any row, i.e. the profile of 0195 — the only draw-free build that has ever
  run the whole hidden corpus.

Exact commands:

```
bash scripts/local-candidate-build.sh
cargo run --release -- --note "iter33 0199: ..."      # appends results.tsv row 1789223560
```

## 6. Measured result and price

* Official sandboxed local harness: **300/300 OK, SCORE 0.792658, fill
  0.924661** (results.tsv row `1789223560`).
* Dev price of retiring the arm, on this profile: **0.792442 -> 0.792658**
  (+2.16e-4). The arm's own counterfactual seam agrees exactly
  (`0197-probe-no-force-dev.log`: `force_fires=0`, same 0.792658 on the
  draw-carrying profile).
* Arm exposure on dev: `FORCEAUDIT stage1b_sites=39 force_fires=10 unsound=0`
  (0198 roll) — ten rows, all sound adoptions, which is precisely why dev
  cannot see the class the hidden receipt paid for.

## 7. Failures and course corrections recorded honestly

* The cap was first read as a monotone work law ("added band time kills"). It
  survives as a *margin* statement, not as a law: 0195 and 0198 differ by one
  constant, one completes and one dies.
* The instant reading of the 0195/0198 pair was "identical code, two outcomes".
  That was wrong and was corrected before acting: the draw window differs; the
  tree diff is quoted in section 3.
* The previous build (0198) restored a donor-pass widening on the theory that
  its removal caused the 0196 kill. That theory is not falsified here but it is
  also not supported by a receipt — 0196 and 0197 each differ from 0195 by more
  than the widening, so no single-variable conclusion was available.

## 8. Caveats

This submission is deliberately **dev-negative** (4.3 bips worse on the public
corpus than the previous build). The bet is that the arm's dev value does not
transfer, which is what the frontier's own receipt shows for this arm. If it
does transfer, the build lands at roughly the frontier's own score and is
rejected. Nothing measured locally can decide that: the 327 measurable rows
(300 dev + 27 structural) contain no row where the force install was unsound,
and the graded corpus is not observable per row.

## 9. Next steps

1. Read this receipt. If it completes at or above 0.842857, the arm's value
   transfers and the removal class is exhausted at this site; the next lever is
   equal-output speed: a bit-identical cut in the pipeline's own kernels (the
   portfolio/MCS and subtree stages dominate per-row seconds), which cannot
   perturb any trajectory and only buys cap margin.
2. If it is killed again, the cap-margin reading is confirmed: run the
   frontier-vs-this-tree per-row wall-clock A/B on the 27-row structural corpus
   to locate the profile's residual slack before shipping any further change.
3. Keep every submission a single-variable change over the last *completed*
   build; the cap censors exactly the builds whose changes touch rows the dev
   corpus cannot see.
