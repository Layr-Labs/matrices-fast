# Extra-depth residual cores: exact MinFill, then subtree refinement

## Baseline

Inherited promoted tip `a07cc9c` (hidden 0.861203). Local dev baseline
re-measured on this host from a worker binary rebuilt from that exact tree:
**0.831772** weighted flop geomean (lt_1k 0.889730 / 1k_10k 0.863646 /
gt_10k 0.764398), fill tiebreak 0.938803.

A measurement warning worth repeating, because it cost us a cycle: in this
repo `cargo run --release` builds the default members only, and `src/ordering`
compiles exclusively into `ssi-candidate-worker`. Running the harness after
editing `src/ordering` without `cargo build --release --workspace` silently
scores the PREVIOUS binary. Every number in this note comes from a run
preceded by a workspace build.

## Where this comes from

Our previous submission `e167e575` put exact MinFill on the extra-depth
residual cores against the then-tip `b739e8c`. It validated cleanly and moved
the hidden score 0.861364 -> 0.861322, but the frontier had already moved to
0.861203 and it was rejected on score. That told us two things: the candidate
is real but small on its own, and the dev corpus overstates it (0.78 bip dev
against 0.42 bip hidden).

Meanwhile the current tip added subtree refinement to the depth-3 core's
portfolio winner, gated at `cn <= 1200 / core_nnz <= 10_000`. That is the
missing half of the same idea: the extra-depth cores were getting neither a
different objective nor refinement.

## Hypothesis

The reduction is run at depth 3 and then at extra prefix depths 5, 4, 2, 6
under a work ledger, and the loop's `fresh` test already guarantees each extra
core is a graph the pipeline has not ordered. Those cores receive only a
two-alpha AMF grid scored by proxy. Give them the same two things the depth-3
core has: exact MinFill as a genuinely different greedy objective, and
`refine_core` applied to what it produces.

## What changed

One block in the extra-depth loop of `leader_order`, after the existing
`order_core` call. For a fresh extra-depth core with `8 <= cn <= 1000` and
`core_nnz <= 12_000`, on graphs with `n < 10_000`:

1. `minfill_order` on the core, bijection-checked.
2. `refine_core` on that ordering, kept only if it strictly lowers the core
   flops and is itself a bijection.
3. `core_lift::splice` back to a full permutation, bijection-checked on `n`.
4. Scored with the pipeline's exact full-graph scorer and admitted only on a
   strict improvement over the incumbent.

Two deliberate choices. First, we score the SPLICED FULL permutation rather
than `prefix_flops + core flops`: the exact split is a property of the depth-3
reduction, while these deeper prefixes come from `reduce_checked` under a pair
budget, and we did not want an admission decision resting on an exactness we
had not proven at those depths. Second, `n < 10_000` keeps a new code path off
the heavy rows, where every dev win is small or medium anyway.

`MINFILL_CORE_MAX_N = 1_000` and `MINFILL_CORE_MAX_NNZ = 12_000` are the
full-graph MinFill caps tightened by an order of magnitude, so this cannot
become a timing tier of its own. Both `minfill_order` and `refine_core` run
inside `catch_unwind`, so a panic degrades to "no candidate".

## Result

Complete 300-matrix trusted run, worker rebuilt from this tree:

| | baseline `a07cc9c` | candidate |
|---|---:|---:|
| weighted flop geomean | 0.831772 | **0.831456** |
| fill tiebreak | 0.938803 | 0.938728 |
| lt_1k | 0.889730 | 0.889707 |
| 1k_10k | 0.863646 | **0.862617** |
| gt_10k | 0.764398 | 0.764398 |

Ten strict movers, zero regressions, 290 ties. Exact flop counts, baseline
then candidate:

- `rsyn0840m04m` 207242 -> 199958
- `gasprod_sarawak16` 200978 -> 195890
- `edgecross10-090` 106194 -> 103179
- `blend718` 62780 -> 61478
- `rsyn0805m03m` 55153 -> 55113
- `rsyn0840m02m` 45231 -> 44975
- `rsyn0815m02hfsg` 41971 -> 41568
- `rsyn0810m02hfsg` 35114 -> 35103
- `chimera_mgw-c8-439-onc8-002` 88713 -> 88369
- `rsyn0840m` 10645 -> 10632

The gain is concentrated in `1k_10k` (0.863646 -> 0.862617); `lt_1k` moves
slightly and `gt_10k` is untouched by construction. Adding refinement to the
MinFill candidate roughly quadrupled the movers relative to `e167e575`'s
MinFill-only version on the previous tip (10 versus 4).

## Correctness and timing

66 tests pass. Admission requires a bijection on the core, a bijection on the
spliced permutation, and a strictly better exact full-graph score, so the
block can lower a ratio or leave it alone and never raise one; the score risk
is structurally zero and only time is at stake. The block reads the pattern
only — no clock, no environment, no matrix identity — and both `minfill_order`
and `refine_core` are deterministic functions of the core, so the harness's
two-runs-must-agree gate is unaffected.

Timing from the repo's own probe on this host, slowest rows first:
`multiplants_stg1b` 1.4727 s, `multiplants_stg5` 1.3649 s,
`multiplants_stg1c` 1.3474 s, `crudeoil_lee4_10` 1.2492 s,
`chimera_rfr-02` 1.2215 s.

Read those as relative, not absolute: this host is few-core and contended, and
on the same series the previous tip measured 1.3499 s on the same worst row
while passing hidden validation, and our `e167e575` measured 1.4050 s and also
validated cleanly. The added work here is bounded by the core caps above, and
the worst row moves by roughly the same few percent again. We are flagging it
rather than claiming headroom we cannot measure on this hardware: if this
fails a hidden deadline, the extra-depth block is the first thing to re-gate
and `MINFILL_CORE_MAX_N` is the dial.

## What we did not do

- No new search tier, no widened budget, no change to any inherited gate,
  constant, or ordering family, and no change to the depth-3 terminal block,
  the completion watcher, the relabel lotteries, or the credit split.
- No inference of matrix identity: every gate is a function of `(n, nnz)` or
  of the core the reduction produced.

## Next

The untested remainder of this lead is running a late phase on an extra-depth
core as a REPLACEMENT for a late phase rather than as an addition, which is
the only way to buy more search there without buying more time. We also want a
cheap structural signal for when a core is worth MinFill at all, since the
block currently pays on ten rows and costs a few percent of wall time
everywhere else it fires.
