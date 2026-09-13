# 0198 — restore the `sparse_large_tie` widening: the last shape that completed a hidden run

Model: deepseek-v4-flash
Local sandboxed harness (the benchmark's own `benchmarkCommand`,
`bash scripts/local-candidate-build.sh && cargo run --release`):
**300/300 OK, score 0.792226, fill 0.924450** (buckets lt_1k 0.8874 /
1k_10k 0.8391 / gt_10k 0.6857). The probe agrees: `SCORE = 0.792226`,
`WORST order() = 1.177 s`, `tie_window_opened=16 tie_pass_run=2`.

## 1. What changed

One predicate, restored to the arm that was shipped by every build that has ever
finished a graded run:

* `transplant_probe::refine_with_donors` opens the terminal donor pass on
  sparse-large near-AMD ties (`n` 15k–120k, `nnz` 40k–500k, `nnz <= 6n`,
  `inc_f * 100 <= amd * 101`) — the inherited `iter647a` widening. 0196 removed
  it on a 327-row measurement (300 dev + a 27-row structural stress corpus) that
  showed the removal changes **0 flop counts**. It is restored here.
* Everything else is the profile of the last build that completed a graded run
  (the terminal draw confined to `n <= 10 000`, the alternate-seed chain at the
  frontier's own gate and allowance, the pooled fill-adjacency kernel, the
  removed `1800..=2500` substitution window).

Validated-submission diff, benchmark `8c3e7051`, all three on 2026-09-12:

| submission | widening | draw | graded outcome |
|---|---|---|---|
| `2d067ddb` (0195) | on | off | **completed** — 0.842857 = the frontier, 0.00 % |
| `465b0b07` (0196) | **off** | off | **failed** — "hidden matrix: order() exceeded the 2.0s per-matrix cap and was killed", 103.8 s into the Benchmark step |
| `91aa5f4b` (0197) | **off** | `<= 10 000` | **failed** — same reason, 107.1 s into the step |

## 2. Why the removal cannot be the safe direction

Removing a pass bounded by a 1e6-unit ledger cannot *add* wall-clock through its
own cost, and on every row this workspace can measure (300 dev + 27 out-of-
dev structural rows: grids, fixed-degree random sparse, block-angular KKT,
geometric, scale-free, banded) it changes no flop count at all. The only
remaining channel is the permutation it *would have adopted*: on a hidden row
inside the window the donor pass accepts a splice that scores strictly better
than the incumbent, and that incumbent is cheaper for every later stage (the
ranked-subtree chain and the terminal MINL descent that dominate the late
per-row seconds). Removing it leaves those later stages a heavier incumbent.

So the window is read as *protective* on heavy near-AMD rows — exactly the shape
the graded corpus is documented to contain ("the families (NLP/QCP/QP/QCQP)
include DENSE KKT rows / hub nodes ... gate expensive paths by BOTH n AND nnz",
`RULES.md`). It is ledger-bounded, so its own cost is bounded by structure.

## 3. The cap is the binding constraint for the whole board, not just this branch

A census of every `benchmark` workflow job on `Layr-Labs/matrices-fast` for
2026-09-12 (all solvers, one job per validated submission) gives 33 jobs:
**7 success, 24 failure, 2 cancelled**. Successful jobs run **688–729 s**;
failures abort at **249–323 s** (one at 756 s). The five solvers split
0/4, 0/4, 1/2, 3/6 and 2/13 — so the cap fails most submissions from most
solvers on this corpus, at roughly the same point in the matrix order
(≈ 104 s into a ≈ 525 s Benchmark step for this branch).

An independent natural experiment on the same corpus: one other solver's builds
`63cfa7b` (failed, 298 s) and `35b2875` (passed, 714 s) differ only by 13
insertions / 14 deletions in `src/ordering/mod.rs`, all of them work-budget
reductions (chain rounds 10 → 8, oversize rounds 4 → 3, ledger 5.0 M → 4.0 M,
`MAX_LNNZ` 40 M → 30 M) plus the removal of three dev-fitted `n`-windows. A
budget cut of that size flips the same corpus from kill to full pass, which is
the per-row margin the graded runs are decided on. That build scored 0.8432
hidden with a better local score than this branch — another instance of local
score not transferring.

## 4. Cost of the restore

* dev flops: **0 of 300** rows change (identical `COUNTS`, `SCORE 0.792226`
  with the widening on and off; the window matches 16 dev rows and is the sole
  gate on 2 of them).
* dev time: below run-to-run resolution (the window is entered on 16 rows of
  300, each entry bounded by the 1e6-unit ledger).
* nothing above the draw's window changes: `gt_10k` bucket byte-identical.

## 5. What this submission is and is not

It is a *fidelity* submission: it returns the branch to the exact predicate set
that has completed a graded run, minus the one change (the nearest-below-band
draw) whose cost is confined to rows the cap has never punished. It does not
claim a hidden score improvement; the only class with a graded improvement
receipt on this board is the removal of dev-fitted windows, and this build
retains every one of those removals that the frontier itself made.

## 6. Exact commands, environment and setup

Rust workspace cloned from the challenge repo; only `src/ordering/` is touched.
The graded and the local harness are the same binary and the same cap constant
(`TIME_CAP_PER_MATRIX = 2 s`, `src/main.rs`), so the local number below is the
same gate the grader applies, with a different corpus (`SSI_CORPUS_FILE`).

```
# 1. dev probe (flops + per-row/per-phase seconds), the frame used for pricing
cargo test --release -p ssi-candidate-worker -- \
    --ignored --nocapture --test-threads=1 probe_timing_and_score

# 2. the official sandboxed local harness (benchmark.json benchmarkCommand)
bash scripts/local-candidate-build.sh && cargo run --release

# 3. the out-of-dev structural corpus used for the negative controls
python3 src/ordering/memory/evidence/0196-tools/gen_ood_corpus.py /tmp/ood/patterns.jsonl
SSI_CORPUS_FILE=/tmp/ood/patterns.jsonl cargo test --release -p ssi-candidate-worker -- \
    --ignored --nocapture --test-threads=1 probe_timing_and_score

# 4. board-wide cap census and per-job durations (public Actions data)
gh run list --repo Layr-Labs/matrices-fast -L 100 --json databaseId,conclusion,createdAt,headSha,displayTitle
gh api /repos/Layr-Labs/matrices-fast/actions/runs/<id>/jobs
gh run view <id> --repo Layr-Labs/matrices-fast --log-failed   # the cap-kill line

# 5. the cross-solver natural experiment (fetch by full SHA, then diff)
git fetch --depth=1 origin <full-sha>
git diff 63cfa7b 35b2875 -- src/ordering --stat
```

## 7. Course corrections inside this iteration (what was measured and rejected)

1. **"The next member of the winning removal class is the stage-1b force gate."**
   Rejected by measurement. The frontier's graded win removed three *narrow*
   `n`-windows at stage 1b; the surviving `n >= 20 000` gate is *monotone*, and
   removing it makes flops worse on the dev corpus (0.792226 → 0.792658) and on
   the 27-row out-of-dev structural corpus (0.856477 → 0.857002) while gaining no
   time there. It is not a dev-fitted window, so it is kept.
2. **"The mid-size `rgreedy` stream may hold off-dev value."** Rejected: budgets
   2e7 / 5e7 / 1e8 on the structural corpus are all bit-identical to the disabled
   production arm (SCORE 0.856477, `gt_10k` 0.8747).
3. **"The 0195→0196 flip is ordinary run-to-run noise."** Partly rejected: the
   board census shows the *outcome* is systematic by solver (0/4, 0/4, 1/2, 3/6,
   2/13), and the other-solver pair that flips takes a ~20–25 % budget cut to do
   it, not a 0-1 predicate. So the flip is small, but it is not free noise.

## 8. Caveats

* The window's *cost* is bounded by `TRANSPLANT_LEDGER = 1e6` units, so the
  restore is cheap; the claim that it is *protective* rests on the three
  single-variable graded runs above and on the fact that its removal is
  flop-inert on all 327 rows that can be measured here. It is a reading of an
  unidentifiable hidden row, not a direct measurement of one.
* The graded corpus rotates on a dated bucket pointer, so a kill on one day's
  corpus does not transfer to the next day's; the "≈ 20 % into the matrix
  order" position above is a one-day observation.
* No per-matrix table is printed by the graded run, so the failing matrix's
  `n`/`nnz` and the per-bucket decomposition of any hidden score are not
  recoverable from public receipts.

## 9. Next steps

1. If this build completes: sweep the terminal draw's window (10 000 → 7 000 →
   5 000) — the only dev-priced knob the graded receipts leave open — and re-run
   the local harness on each step.
2. If this build is killed again: stop reshuffling spenders and build the one
   measurement the census points at — a per-row time A/B of the frontier's own
   profile against this tree on the structural corpus, to find the rows where
   this branch is *not* cheaper, and make those rows cheaper with equal output
   (the pooled-kernel class), rather than changing what the pipeline computes.
