# Oversize in-gate PEO rounds: charge them, do not exempt them

## Baseline

Inherited promoted tip `0503cf6` (submission `be9f0a4`, hidden 0.859925). Local
dev baseline on this host, worker rebuilt from that exact tree: **0.827794**,
fill 0.937666 (lt_1k 0.889730 / 1k_10k 0.863415 / gt_10k 0.754627).

Host caveats: `src/ordering` compiles only into `ssi-candidate-worker`, so
`cargo run --release` alone re-scores a stale worker after an edit and every
number here follows `cargo build --release --workspace`; and this host is
few-core and contended, so per-matrix times run roughly 2x the runner's and are
used only as deltas.

## The gap

The inherited tree now has two terminal PEO chains. Rows inside
`16 <= n <= 30_000 && nnz <= 180_000` run `candidates`, and rows above that gate
run the same chain under a work ledger. What neither reaches is a row that is
inside the gate but whose factor is too large for the reconstruction: the
in-gate path still refuses anything above `MAX_LNNZ = 300_000` outright, and the
above-gate branch is an `else`, so those rows fall through both.

That set is not hypothetical. We instrumented `reconstruct` to log every accept
and refusal and ran the full dev corpus:

- 630 accepted reconstructions, largest accepted factor **289,121**.
- **6** refusals, all `MAX_LNNZ`, being three matrices seen twice: Lnnz
  **381,126** (n=17,493), **618,374** (n=15,904), **714,536** (n=17,809).
- Zero refusals from `MAX_N`, zero from `MAX_INPUT_NNZ`.

So the constant sits just above the largest factor it admits, and the rows it
turns away are all in `gt_10k`, the bucket carrying weight 0.40. The
instrumentation was reverted before the runs below.

## Why a ledger and not a bigger constant

We already tried the constant. Raising `MAX_LNNZ` to `650_000` scored 0.828460
on dev against the previous tip, two movers, no regressions, worst probe row
1.3654 s — and it FAILED hidden validation (submission `9efba180`). Two earlier
submissions failed the same way, and all three shared one shape: they widened a
limit so that larger inputs entered an expensive path, with nothing bounding
what the largest such input could cost. A flat cap says "anything under this
size is free", which is only true if the dev corpus contains the worst case. It
does not; the full corpus reaches n around 340k.

The above-gate branch in this tree solves that honestly, so we reuse its
mechanism rather than invent one. An oversize round pays before it runs:

```
cost(round) = 5 * (n + nnz) + Lnnz
ledger      = 2_500_000 units per matrix
```

A round whose factor is within the existing `MAX_LNNZ` is not charged at all,
so the promoted in-gate chain keeps its eight free rounds and none of its
behaviour changes. Only a round above the old limit touches the ledger, and when
the ledger cannot cover the next one the chain ends. The ceiling for a charged
round is `PEO_OVERSIZE_MAX_LNNZ = 1_000_000`, but the binding constraint is the
ledger, not that number: on the three matrices above, one round costs 1.07M to
1.45M units, so each gets one or two rounds and then stops. The cost of this
change is bounded by structure the runner can be counted on to have, rather than
by a size we happened not to see.

## Result

Complete 300-matrix trusted run, baseline then candidate:

| | `0503cf6` | candidate |
|---|---:|---:|
| weighted flop geomean | 0.827794 | **0.827195** |
| fill tiebreak | 0.937666 | 0.937411 |
| lt_1k | 0.889730 | 0.889730 |
| 1k_10k | 0.863415 | 0.863415 |
| gt_10k | 0.754627 | 0.753129 |

Three strict movers, zero regressions, 297 ties. Exact flop counts:

- `nuclear10a` 62648236 -> 58262504 (7.0%)
- `crudeoil_lee4_09` 164703084 -> 162105989 (1.6%)
- `crudeoil_lee4_10` 199204080 -> 199013967 (0.1%)

All three are the refused rows, all land in `gt_10k`, and the other two buckets
are bit-identical — which is what a change confined to oversize factors should
look like.

## Correctness and timing

68 tests pass. No new code path was added: the in-gate call moves from
`candidates` to `candidates_bounded` with the same `MAX_N` and `MAX_INPUT_NNZ`
and a per-round `max_lnnz`, and the round body, tie policy and acceptance are
untouched. Acceptance still requires a bijection and a strictly lower exact
score from the independent scorer, so this can only lower a ratio or leave it
alone; the reconstruction's per-column checks, `checked_add` totals and final
`total + n == lnnz` equality are unchanged, so a larger admitted factor is
validated exactly as a small one is. The block reads only the pattern, uses no
clock or environment, and the ledger is a function of `n`, `nnz` and `Lnnz`, so
the result stays deterministic and the two-runs-must-agree gate is unaffected.

Timing probe on this host, slowest rows first: `multiplants_stg1b` 1.3674 s,
`multiplants_stg1c` 1.2859 s, `chimera_rfr-02` 1.2468 s, `multiplants_stg5`
1.2428 s, `crudeoil_lee4_10` 1.2295 s. Under the failed flat-cap version
`crudeoil_lee4_10` was the slowest row on this host at 1.4484 s; under the
ledger it is 1.2295 s and the tail is where the baseline left it.

## Also tried, and rejected

- Retrying a stalled chain under a reversed zero-weight bucket seed: **zero**
  movers across 300 matrices. The two traversal directions already exhaust what
  the tie policy reaches.
- Raising the in-gate round cap from 8 to 32: only 0.39 bip on dev, three
  `mpbp_*` movers. Chain-length instrumentation shows 132 winning rounds with
  eight runs reaching the old cap, so the cap does bind — it just is not worth a
  submission.

## Next

`crudeoil_lee4_10` now takes its one charged round and stops with a 0.1% gain,
so the question there is whether a cheaper round exists rather than whether it
deserves a larger allowance. The other open measurement is the cost law itself:
the 5:1 weighting between `(n + nnz)` and `Lnnz` is inherited from the
above-gate experiment's regression, and we have not re-fitted it for in-gate
rows, where `Lnnz` dominates by a wider margin.
