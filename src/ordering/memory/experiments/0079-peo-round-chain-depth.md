# Terminal PEO re-extraction: let the strict-gain chain run

## Baseline

Inherited promoted tip `33ae43a` (hidden 0.860209). Local dev baseline on this
host, worker rebuilt from that exact tree: **0.830047**, fill 0.938301
(lt_1k 0.889730 / 1k_10k 0.863467 / gt_10k 0.760220). That reproduces the
inherited experiment note's own figure exactly, which is our evidence that we
are measuring the tip we think we are.

Two host caveats we state up front. `src/ordering` compiles only into
`ssi-candidate-worker`, so `cargo run --release` will re-score a stale worker
after an edit; every number here follows `cargo build --release --workspace`.
And this host is few-core and contended, so absolute per-matrix times run
roughly 2x what the runner sees — we use them as deltas only.

## The change

One number. The inherited terminal PEO re-extraction loop runs `for _ in 0..2`;
this runs `for _ in 0..8`. Nothing else in the block, the extractor, the gates,
the tie policy, or the acceptance test is touched.

The inherited note argues carefully for the extractor and then stops the loop
at two rounds without arguing for two specifically. We think two is leaving
money on the floor, because the loop is already self-limiting in the exact way
that makes a higher cap safe:

- A round that produces no strict exact gain breaks immediately. So the second
  round is only ever paid for on a matrix where the first round won, the third
  only where the second won, and so on. The cost of a deeper cap is charged
  only to matrices that have already proven the work pays.
- Each round reconstructs the induced completion of a strictly better
  permutation. The graph a round works on is non-increasing, so later rounds
  are cheaper than earlier ones, not more expensive.
- Acceptance is unchanged: a candidate is kept only if it is a bijection and
  strictly lowers the exact score, so a deeper chain cannot make any matrix
  worse. The only thing at risk is time.

The cap of 8 exists so the loop cannot run unbounded if some structure admits a
long chain of tiny strict gains. In practice the dev corpus never needed it.

## Result

Complete 300-matrix trusted run, baseline then candidate:

| | `33ae43a` | candidate |
|---|---:|---:|
| weighted flop geomean | 0.830047 | **0.829057** |
| fill tiebreak | 0.938301 | 0.938063 |
| lt_1k | 0.889730 | 0.889730 |
| 1k_10k | 0.863467 | 0.863434 |
| gt_10k | 0.760220 | 0.757769 |

Nine strict movers, zero regressions, 291 ties. Exact flop counts:

- `gabriel09` 28335084 -> 27462015
- `pooling_sppa9tp` 46402955 -> 46209948
- `mpbp_34` 1040202 -> 977042
- `mpbp_35` 1062835 -> 1015868
- `mpbp_48` 16338243 -> 16276831
- `edgecross24-115` 32011580 -> 31965269
- `chp_shorttermplan2d` 2120242 -> 2118201
- `crudeoil_lee4_06` 51990726 -> 51990536
- `crudeoil_lee2_06` 22660579 -> 22660574

The gain is concentrated in `gt_10k`, the heaviest-weighted bucket, and
`lt_1k` is bit-identical. That shape is what we would expect if the extra
rounds matter where the completion is large enough for a second MCS pass to
find a different tie structure, and we read the two `crudeoil` rows — five and
190 flops respectively — as the loop correctly taking a microscopic strict gain
and then stopping, rather than as noise.

## Correctness and timing

68 tests pass, including the inherited extractor tests that enumerate all
labeled simple graphs and all incumbent permutations for n=1..5. We added no
test because we added no code path; the round body under a deeper cap is the
same body the existing tests already cover, and its acceptance gate is
unchanged.

The block reads only the pattern and its own reconstruction, uses no clock,
environment or matrix identity, and both MCS extractions are deterministic
functions of the incumbent, so the two-runs-must-agree gate is unaffected.

Timing probe on this host, slowest rows first: `multiplants_stg1b` 1.3558 s,
`chimera_rfr-02` 1.2624 s, `multiplants_stg5` 1.2350 s, `multiplants_stg1c`
1.2284 s. For calibration on the same host and probe, the previous tip measured
1.3499 s on its worst row. The deeper cap does not move the tail, which is the
result the self-limiting argument predicts.

We are being deliberate about timing because our last two submissions,
`23af3f88` and `d713f925`, both FAILED hidden validation while scoring fine
locally. Those added exact MinFill, and then MinFill plus a rationed
refinement, to extra-depth residual cores — unconditional work on up to four
cores per matrix. We have abandoned that line rather than tune it a third
time, and we note that this candidate is the opposite shape: it adds no
unconditional work at all.

## What we did not do

- No change to the extractor, its `MAX_N` / `MAX_INPUT_NNZ` / `MAX_LNNZ` caps,
  the `n >= 16 && n <= 30_000 && nnz <= 180_000` gate, the bucket tie policy,
  or the fresh-symbolic-counts scoring.
- No new search tier, ordering family, budget, or seed; no change to the
  watcher, the relabel lotteries, the credit split, or `completion.rs`.

## Next

The open question this raises is whether the round should also try candidates
seeded from something other than the incumbent order once the chain stalls,
since a stalled chain currently ends the block entirely. The other lead we have
not measured is whether the extractor's `MAX_LNNZ` cap is refusing
reconstruction on exactly the large `gt_10k` matrices where these rounds pay
best — worth a count of refusals before anyone raises a cap that would also
raise worst-case time.
