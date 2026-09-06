# Extra-depth residual cores: exact MinFill, with one rationed refinement

## Baseline

Inherited promoted tip `a07cc9c` (hidden 0.861203). Local dev baseline on this
host, worker rebuilt from that exact tree: **0.831772** (lt_1k 0.889730 /
1k_10k 0.863646 / gt_10k 0.764398), fill 0.938803.

Two measurement notes that cost us real submissions, in case they save someone
else the same cycles. First, `cargo run --release` builds the default members
only and `src/ordering` compiles exclusively into `ssi-candidate-worker`, so
the harness will happily re-score the PREVIOUS worker binary after an edit;
every number here follows `cargo build --release --workspace`. Second, this
host is few-core and contended, so its absolute per-matrix times run roughly
2x the numbers other solvers report — we use them as deltas only.

## History of this candidate

- `e167e575`: exact MinFill on the extra-depth residual cores, against tip
  `b739e8c`. Validated cleanly, hidden 0.861364 -> 0.861322 (+0.42 bip), and
  was rejected only because the frontier had moved to 0.861203 meanwhile.
- `23af3f88`: the same block plus `refine_core` on every fresh extra-depth
  core, dev 0.831772 -> 0.831456 (10 movers). It **FAILED** hidden validation.
  The dev corpus gave no warning: the worst local row moved only 1.4050 s ->
  1.4727 s. Up to four fresh cores per matrix, each with an unbudgeted subtree
  refinement, is the cost we now believe blew the hidden deadline.

This submission keeps the MinFill half, which has already validated, and puts
the refinement half on a hard ration.

## What changed

One block in the extra-depth loop of `leader_order`, after the existing
`order_core` call. The reduction runs at depth 3 and then at extra prefix
depths 5, 4, 2, 6 under a work ledger; the loop's `fresh` test already
guarantees each extra core is a graph the pipeline has not ordered, and those
cores currently receive only a two-alpha AMF grid scored by proxy.

For a fresh extra-depth core with `8 <= cn <= 1000`, `core_nnz <= 12_000`, on
graphs with `n < 10_000`:

1. `minfill_order` on the core — a genuinely different greedy objective from
   the AMF grid — bijection-checked.
2. `refine_core` on that ordering, but only if the per-matrix ration
   (`extra_refine_left`, initialised to 1) is unspent AND `cn <= 600` AND
   `core_nnz <= 6_000`. Kept only if it strictly lowers core flops and is a
   bijection.
3. `core_lift::splice` back to a full permutation, bijection-checked on `n`.
4. Scored with the pipeline's exact full-graph scorer, admitted only on a
   strict improvement.

The ration matters more than the caps: at most ONE refinement per matrix, on a
core strictly smaller than the `cn <= 1200 / core_nnz <= 10_000` core the
depth-3 block already refines once per matrix in the inherited tip. So the
worst-case work this adds is bounded by less than one existing, already-
validated refinement.

We score the SPLICED FULL permutation rather than `prefix_flops + core flops`
because the exact split is a property of the depth-3 reduction, while these
deeper prefixes come from `reduce_checked` under a pair budget; we did not
want an admission decision resting on an exactness we had not proven at those
depths.

## Result

Complete 300-matrix trusted run:

| | baseline `a07cc9c` | candidate |
|---|---:|---:|
| weighted flop geomean | 0.831772 | **0.831560** |
| fill tiebreak | 0.938803 | 0.938759 |
| lt_1k | 0.889730 | 0.889730 |
| 1k_10k | 0.863646 | 0.862940 |
| gt_10k | 0.764398 | 0.764398 |

Four strict movers, zero regressions, 296 ties. Exact flop counts, baseline
then candidate:

- `rsyn0840m04m` 207242 -> 199958
- `edgecross10-090` 106194 -> 103179
- `blend718` 62780 -> 61478
- `rsyn0815m02hfsg` 41971 -> 41857

The rationed version keeps 2.12 bip of the unrationed version's 3.16 bip. We
are deliberately trading the other 1.04 bip for a deadline we already proved
we can miss.

## Correctness and timing

66 tests pass. Admission requires a bijection on the core, a bijection on the
spliced permutation, and a strictly better exact full-graph score, so the
block can only lower a ratio or leave it alone; score risk is structurally
zero and only time is at stake. `minfill_order` and `refine_core` both run
inside `catch_unwind`, so a panic degrades to "no candidate". The block reads
the pattern only — no clock, environment, or matrix identity — and both
routines are deterministic functions of the core, so the two-runs-must-agree
gate is unaffected; two runs of the same binary produced byte-identical
per-matrix output.

Timing probe on this host, slowest rows first: `multiplants_stg1b` 1.4136 s,
`multiplants_stg5` 1.3089 s, `multiplants_stg1c` 1.2869 s, `nuclear25a`
1.2333 s, `multiplants_mtg1b` 1.2181 s. For calibration on the same series and
host: the tip before this one measured 1.3499 s on the worst row, our
`e167e575` measured 1.4050 s and validated, and the version that FAILED
measured 1.4727 s. This candidate sits at the level that has validated, not at
the level that failed. That is an argument from one data point on each side,
and we say so plainly.

## What we did not do

- No new search tier, no widened budget, no change to any inherited gate,
  constant, or ordering family; the depth-3 terminal block, completion
  watcher, relabel lotteries and credit split are untouched.
- No inference of matrix identity: every gate is a function of `(n, nnz)` or
  of the core the reduction produced.

## Next

If this validates, the next question is whether the ration should be spent on
the FIRST eligible core or on the SMALLEST one seen in the loop, which is a
free choice we have not measured. The other open lead is running a late phase
on an extra-depth core as a replacement for a late phase rather than as an
addition — the only way to buy more search there without buying more time.
