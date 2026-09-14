# 0206 — the graded frame is 4 vCPU, and the terminal four-stream fan-out is dev-inert

## 0. Context, baseline, and the question this change answers

This branch owns a promoted submission (hidden `0.842716`) built on a fill-scale fence, and has
lost six of its last seven remote validations to the graded 2.0 s per-matrix cap
(`7bba8604`, `cad51a0f`, `1a93d29e`, `f647df4d` — the last three at 103.8 / 111.6 / ~104 s of
job time). The board's best is `0.842377`. Every local number this branch reasons with — dev
score, per-row `order()` seconds, "worst row 1.565 s, 0.435 s of cap margin" — was measured on the
development box with **no affinity mask**. That is the assumption this submission tests and, where
it is false, removes a device that depended on it.

## 1. What changed

One constant, one line of production logic:

```rust
- const SHIPPED_ENGINE_FANOUT: &[i64] = &[2_000_000_000];
+ const SHIPPED_ENGINE_FANOUT: &[i64] = &[];
```

With an empty budget list the terminal four-stream fan-out stage (`rgreedy::search_par_specs`
over four independent 2e9-op trajectories, `src/ordering/mod.rs:5526`…) becomes unreachable: its
round loop breaks before any stream is spawned. Nothing else in `src/ordering/` is touched; the
stage's structural gate, its four seeds and the primitive itself remain in the tree and remain
exercised by the seam (`SSI_ENGINE_FANOUT`) for anyone who wants to re-price them.

## 2. Environment and setup

* Candidate repo: `/home/frosty/angelX/.tmp/live-performance-20260911/matrices-deepseek`
  (commit before this change: `c001ffd`, "iter39 0205").
* Dev corpus: `corpus/dev/patterns.jsonl`, 300 matrices, buckets `lt_1k` 147 / `1k_10k` 108 /
  `gt_10k` 45, weights 0.30 / 0.30 / 0.40.
* Dev box: Intel i7-13700F, **24 logical / 16 physical** cores (`lscpu`), CPUs 0-1 are SMT
  siblings of *one* physical core, 0-3 span two physical cores.
* Graded runner: GitHub-hosted `ubuntu-latest`, **4 vCPU**, as the repository's own module
  documents — `src/ordering/parallel.rs:3` "The graded worker runs on a GitHub-hosted
  `ubuntu-latest` runner (4 vCPU for a public repository)" and `:59` `PAR_MAX_THREADS = 4`
  ("more threads than this can only add scheduling and memory cost").
* Instruments: the official sandboxed local harness (`./target/release/matrices-fast`, purity
  gate + bubblewrap worker + watchdog, 2 runs per matrix for the determinism gate) and the
  test-only probe (`target/probe-sandbox.sh`, same bubblewrap build boundary), which prints
  per-row `order()` seconds, per-row flops and phase marks that the harness hides behind
  `(capped)`.

## 3. Hypothesis A (frame): the margins this branch quotes are quoted in a wider frame

Every published margin on this branch is a 24-core number. The graded box has 4 vCPUs, and the
worker's pipeline contains several hard-capped 4-thread stages (`PAR_MAX_THREADS = 4` in
`parallel.rs`, further `thread::scope` fan-outs in `rgreedy.rs`, `indep_first.rs`, `mod.rs`).
On a 4-vCPU box those stages have to share two physical cores; on the dev box they get four
idle physical cores. The prediction is a systematic, stage-local inflation of per-row wall time
which no dev score can show, because flops are unaffected.

**Measurement.** The same probe binary, same seams (`SSI_INDEP_FORCE=1 SSI_LADDER_STRIDE=1
SSI_NO_SPARSE_LARGE_TIE=`), three affinity masks:

| frame | worst `order()` | SCORE |
|---|---|---|
| unpinned (16 physical cores) | **1.542 s** | 0.792112 |
| `taskset -c 0-3` = graded frame | **1.822 s** | 0.792112 |
| `taskset -c 0,1` = 2 vCPU | **3.509 s** | 0.792112 |

* 99 of 300 rows inflate by more than 0.05 s at 4 vCPU; median row ratio 1.10; the worst single
  row is `rsyn0820m04m` 1.266 → 1.821 s; the top of the list is rounded out by
  `arki0016` (+0.37), `squfl020-150` (+0.32), `syn40m04hfsg` (+0.43), `ringpack_20_3` (+0.41),
  `chimera_selby-c16-02` (+0.44).
* At 2 vCPU the whole corpus roughly doubles (`emfl100_3_3` 1.42 → 3.21 s), i.e. a 2-vCPU grader
  would kill this tree on dev alone.

Commands:

```
SSI_INDEP_FORCE=1 SSI_LADDER_STRIDE=1 SSI_NO_SPARSE_LARGE_TIE= \
  taskset -c 0-3 bash target/probe-sandbox.sh run      # 0206-pinned4cpu-probe.log
SSI_INDEP_FORCE=1 SSI_LADDER_STRIDE=1 SSI_NO_SPARSE_LARGE_TIE= \
  taskset -c 0,1 bash target/probe-sandbox.sh run      # 0206-pinned2cpu-probe.log
SSI_INDEP_FORCE=1 SSI_LADDER_STRIDE=1 SSI_NO_SPARSE_LARGE_TIE= \
  bash target/probe-sandbox.sh run                     # 0206-unpinned-control.log
```

**Consequence.** The margin this branch has been quoting ("worst row 1.565 s, 0.435 s under the
cap") is **0.18 s** in the frame that enforces the cap. The official sandboxed harness pinned to
the graded frame still passes — 300 matrices, 0 failures, score 0.7919 / tiebreak 0.9244 in
5m07.2 s (user 11m33 s) — so the tree is legal, but there is much less room on it than the
ledger assumed.

**Negative control (recorded because it could have invalidated everything above).** Core width
does **not** change any flop ratio: the probe returns SCORE 0.792112 at 2, 4 and 24 logical CPUs,
row for row, from the same binary. The "output-identical parallelism" claim of the threaded
stages survives a 12x change in available parallelism — measured, not assumed. (An earlier
apparent 4-row discrepancy turned out to be a *build* difference against a pre-`c001ffd` log, not
a core-count effect; it is the reason this control was run.)

## 4. Hypothesis B (device): the newest terminal stage is now pure price

The four-stream fan-out was shipped one iteration ago on the claim that "four streams of `budget`
ops each cost the same wall time as one, and buy 4x the search", with a measured dev delta of
−2.70e-4 against a build that did *not* have the ratio-gated rung ladder. If the ladder added
later now covers the same rows, the fan-out's remaining contribution is wall time on the rows it
fires on — which is exactly the quantity the graded cap punishes.

**Measurement (production A/B, one constant).**

```
taskset -c 0-3 ./target/release/matrices-fast --note "graded-frame control"        # fan-out ON
taskset -c 0-3 ./target/release/matrices-fast --note "fan-out removed"            # FANOUT = &[]
```

| build | score | tiebreak | buckets (lt_1k / 1k_10k / gt_10k) | wall, pinned |
|---|---|---|---|---|
| shipped (fan-out ON) | **0.791896** | 0.924419 | 0.887431 / 0.838452 / 0.685328 | 5m07.2 s |
| this submission (OFF) | **0.791896** | 0.924419 | 0.887431 / 0.838452 / 0.685328 | 5m05.8 s |

Byte-identical on both sides (`results.tsv:1789241019` with the stage, `results.tsv:1789241463`
without). The probe confirms at row granularity: with `SSI_ENGINE_FANOUT=0`, **0 of 300** dev rows
change flops; corpus wall drops 0.85 s; the worst single-row saving is 0.137 s
(`p_ball_30b_5p_2d_h`). `rsyn0830m04m` — the fan-out's flagship mover when it was introduced
(0.8100 → 0.7854, 166607 flops) — reaches the *same* 166607 flops with the stage absent, because
the rung ladder added in iter38 covers that row.

**Correction recorded.** The 0205 ledger line "one terminal four-stream round at 2e9/stream costs
+18.7 s of corpus time, ~+0.53 s on each of the 35 firing rows" is not reproducible against the
current tree. The price that remains is ~5 ms/row on average. The stage is therefore *dominated on
the measured axis*: identical search value on all 300 rows, strictly less work on each of the 35
rows it fires on and on any hidden row inside its gate.

## 5. Trade-offs considered

* *Keep the fan-out for hidden-corpus value.* The stage's dev value is zero **because** another
  device now covers it; on a hidden row outside the ladder's ratio gate its four 2e9-op walks
  could still find something the ladder cannot. That is unmeasurable here. What is measurable is
  that on the class the cap has actually killed us on — rows that already spend near the wall —
  the fan-out is unspent budget with no demonstrated return anywhere in the dev corpus.
* *Shrink the fan-out instead of removing it* (2e9 → 1e9/stream): the earlier dose–response
  measured −1.01e-4 for 4x1e9 against a pre-ladder build, i.e. value that no longer exists in
  this tree while ~half the wall remains. Removal dominates shrinking here.
* *Spend the freed wall on value* (a fifth rung, a wider ladder): deliberately not done in the
  same change. This submission is the survival instrument; a value add on top would make the
  receipt uninterpretable, and the branch's last four receipts show the cap, not the score, is
  what is currently rejecting this line of work.

## 6. What is *not* claimed

* No dev score improvement: score and all three buckets are identical to the shipped build's own
  measured row. The claimed `0.791896` is this build's measured local score.
* The removal is not proven to fix the hidden kills. It is proven to be wall-negative on every
  dev row and value-neutral on every dev row.
* The 4-vCPU grading claim rests on the repository's documented runner type and the graded job's
  public workflow log (`runs-on: ubuntu-latest`), not on a direct CPU reading of the grading host.
  The 2-vCPU row of the table is a *counterfactual* frame, not a claim about the grader.

## 7. Learning and next steps

1. Per-row margins must be quoted with the affinity mask that produced them; on this box the
   graded frame is `taskset -c 0-3` and it costs 10–18 % on the rows that matter.
2. A device can be *dev-inert yet fully budgeted*: the fan-out still spends its 2e9 ops × 4
   streams on every row inside its gate, and on the hidden corpus a row inside that gate is the
   one place that budget can turn into a kill. Devices should be re-priced whenever a newer
   device changes the pipeline's coverage — shipping a stage is not a permanent justification.
3. Next: re-price the remaining sequential spends under the same graded frame (the rung ladder's
   four rungs are 5e8 ops paid four times in wall clock; the terminal draw is still budgeted at
   the wide band) and take the same production-A/B treatment — a stage that no longer moves any
   dev row does not belong in a build that is graded against a 2 s per-matrix cap.

## 8. Reproduction

```
# graded-frame control (shipped row)
time taskset -c 0-3 ./target/release/matrices-fast --note "graded-frame control"
# this submission
grep -n SHIPPED_ENGINE_FANOUT src/ordering/mod.rs    # -> &[]
time taskset -c 0-3 ./target/release/matrices-fast --note "fan-out removed"
# row-level confirmation of the removal
SSI_ENGINE_FANOUT=0 taskset -c 0-3 bash target/probe-sandbox.sh run
```

Artifacts in `src/ordering/memory/evidence/`: `0206-pinned2cpu-probe.log`,
`0206-pinned4cpu-probe.log`, `0206-unpinned-control.log`, `0206-noenv-probe.log`,
`0206-pinned4cpu-harness.log`, `0206-prod-fanout-off-pinned4.log`,
`0206-pinned4cpu-fanout-off.log`, `0206-pinned2cpu-fanout-off.log`, `0206-phases-4cpu.log`, plus
the iter40 ledger entry in `0162-remote-submission-ledger.txt`.
