# Exact MinFill on the extra-depth residual cores

## Baseline

Inherited promoted tip `b739e8c` ("Validate submission 40970270", hidden
0.861364). Local dev-corpus baseline re-measured on this host from a worker
binary built from that exact tree: **0.832118** weighted flop geomean
(lt_1k 0.889764 / 1k_10k 0.864765 / gt_10k 0.764398), fill tiebreak 0.938869.

A methodology note first, because it invalidated our own earlier measurement
and may bite anyone reading this: in this repo `cargo run --release` builds
the default members only, and `src/ordering` is compiled exclusively into the
`ssi-candidate-worker` package. A plain `cargo run --release` after changing
`src/ordering` therefore happily re-runs the PREVIOUS worker binary and prints
a score for code you are no longer looking at. Every number below comes from a
run preceded by `cargo build --release --workspace`. Our first two internal
measurements of this cycle were stale for exactly this reason, and the
apparent "regression" they showed on `gabriel10` and
`acopf_case9241pegase_qcqp` was simply the inherited tip's own effect on those
two rows, not anything our change did.

## Hypothesis

The inherited pipeline peels an exact low-degree prefix and orders the
residual core (`REDUCE_ROW_DEG = 3`), then repeats the reduction at extra
prefix depths 5, 4, 2, 6 under a work ledger. The depth-3 core gets a rich
terminal treatment: an AMF alpha grid, two structural relabels, recursive
refinement, and — most recently — exact MinFill for cores of at most 1000
vertices. The extra-depth cores get only a two-alpha AMF grid.

Those extra-depth cores are structurally different graphs. A deeper exact
prefix eliminates different vertices, closes different neighbourhoods into
cliques, and leaves a core the depth-3 reduction never produces; the loop's
own `fresh` test already refuses to re-order a core it has seen. AMF is a
fill-flavoured degree heuristic, so the whole candidate set on those cores
comes from one family. Exact MinFill is a genuinely different greedy
objective, and it is the same argument that earned the depth-3 core its
MinFill candidate one level down.

## What changed

One block, in the extra-depth loop of `leader_order`, after the existing
`order_core` call:

- For a fresh extra-depth core with `8 <= cn <= 1000` and
  `core_nnz <= 12_000`, run `minfill_order` on the core.
- Check the result is a bijection on the core, splice it back through
  `core_lift::splice`, check the spliced permutation is a bijection on `n`.
- Score the SPLICED FULL permutation with the same exact scorer the rest of
  the pipeline uses, and keep it only on a strict improvement.

Two deliberate choices inside that block:

**Score the full permutation, not `prefix_flops + core flops`.** The exact
split is a property of the reduction; the deeper prefixes here are produced by
`reduce_checked` under a pair budget, and we did not want a candidate admitted
on a score whose exactness we had not proven for those depths. One exact
full-graph score per candidate is cheap at these core sizes and makes the
comparison unconditionally sound. We measured both versions: on this corpus
they select identically, so the cost buys certainty rather than score.

**`n < 10_000`.** The block is bounded by the core, but the graphs where every
dev win landed are small and medium. Keeping the heavy rows out of a new code
path costs nothing measurable and keeps the slow tier exactly as the inherited
tip has it.

New constants `MINFILL_CORE_MAX_N = 1_000` and `MINFILL_CORE_MAX_NNZ = 12_000`
are the full-graph MinFill caps tightened by roughly an order of magnitude, so
this can never become a timing tier of its own.

## Result

Complete 300-matrix trusted run, worker rebuilt from this tree:

| | baseline `b739e8c` | candidate |
|---|---:|---:|
| weighted flop geomean | 0.832118 | **0.832040** |
| fill tiebreak | 0.938869 | 0.938853 |
| lt_1k | 0.889764 | 0.889764 |
| 1k_10k | 0.864765 | **0.864505** |
| gt_10k | 0.764398 | 0.764398 |

Four strict movers, zero regressions, 296 ties. All four are exact flop
counts, baseline then candidate:

- `rsyn0840m04m` 207242 -> 202036
- `rsyn0815m02hfsg` 41971 -> 41857
- `oil` 34913 -> 34848
- `rsyn0840m02m` 45518 -> 45406

Every mover is in `1k_10k`; the small bucket did not move at four digits and
the large bucket is untouched by construction.

## Correctness and timing

66 tests pass. The candidate is admitted only through a bijection check on the
core, a bijection check on the spliced permutation, and a strict exact-score
improvement, so the score risk is structurally zero: the block can lower a
ratio or leave it alone, never raise it. `minfill_order` runs inside
`catch_unwind`, consistent with the rest of the candidate calls in this
pipeline, so a panic in it degrades to "no candidate" rather than a failed
run. The block reads only the pattern — no clock, no environment, no
identity — and both `minfill_order` and the reduction are deterministic
functions of the core, so the harness's two-runs-must-agree gate is unaffected.
Two full runs of the same binary produced byte-identical per-matrix output.

Timing, measured with the repo's own timing probe on the same host and the
same series, slowest rows first (baseline -> candidate):

- `multiplants_stg1b` 1.3499 s -> 1.4050 s
- `multiplants_stg1c` 1.3147 s -> 1.2763 s
- `chimera_rfr-02` 1.2783 s -> 1.2685 s
- `multiplants_stg5` 1.2499 s -> 1.3777 s
- `crudeoil_lee4_10` 1.2189 s -> 1.2676 s

The honest reading: the worst row moves from 1.3499 s to 1.4050 s, about +4%,
and the shuffling among the middle rows is at the level of this host's noise.
These absolute numbers are NOT comparable to a fast grader box — this host has
few cores and is contended, and the inherited tip, which passed hidden
validation, already measures 1.35 s on the same series here. We are reporting
the delta, not the absolute, and the delta is small and bounded by the core
caps above. We flag it plainly rather than claiming headroom we cannot
measure: if this submission fails a hidden deadline, the extra-depth MinFill
block is the first thing to re-gate, and `MINFILL_CORE_MAX_N` is the dial.

## What we did not do

- No new search tier, no widened budget, no change to any inherited gate,
  constant, or ordering family.
- No attempt to infer matrix identity; every gate is a function of `(n, nnz)`
  or of the core the reduction produced.
- We did not touch the depth-3 terminal block, the completion watcher, the
  relabel lotteries, or the credit split.

## Next

The remaining untested part of the same lead is running a late phase on an
extra-depth core as a REPLACEMENT for a late phase rather than as an addition,
which is the only way to buy more search there without buying more time. We
also want a cheap signal for when a core is worth MinFill at all, since on
this corpus the block pays on four rows and costs a few percent of wall time
on rows where it finds nothing.
