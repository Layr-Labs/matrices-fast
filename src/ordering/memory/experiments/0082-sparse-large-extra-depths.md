# 0082: the sparse-large band of the extra-depth reductions, bounded

- **Model:** Claude Opus 5 (1M context)
- **Harness:** Claude Code with the agentprivacy dual-agent harness (matrices_mage instance)
- **Date:** 2026-09-06
- **Base:** `a2614af` (submission `1a4183b7`, hidden 0.859834)
- **Status:** measured, all gates green; official pending

## The gap

The extra-depth reductions run behind two bands: small graphs, and dense mid-to-large graphs at
`nnz >= 6 * n`. Between four and six times the dimension sits a class that qualifies on nonzeros and
fails on density, receiving no extra depth at all.

An earlier experiment relaxed the same floor and read as a null result. That predates the terminal
chains. An extra depth leaves a different residual core and the terminal machinery now works on that
core, so the reduction pays only in combination.

## The first attempt failed the hidden cap

Submission `21f19843` admitted the band with no cap on the work an extra depth could do, and failed
the two-second limit after passing every local gate, including an interleaved minimum-of-ten on
exactly the admitted rows showing nothing slower than 0.05 s.

The lesson, consistent across three submissions from this workspace: **local timing is not a
substitute for a bound.** The only one that survived (`be9f0a41`) charged its added work against an
explicit per-matrix allowance. These corpora do not contain the hidden corpus's slow rows.

## The bound

The band carries its own nonzero cap, so the depth is only attempted where the work it can do is
limited, rather than where it happened to measure cheap:

```rust
const REDUCE_SPARSE_DENSITY: usize = 4;
const REDUCE_SPARSE_MAX_NNZ: usize = 350_000;
```

The cap costs no measured gain. Every row that gains is below it (275_106, 331_010, 277_562); every
row above it gained exactly nothing (476_332, 480_012, 557_994, 1_148_210 and larger). No budget is
raised: the reduce-work budget, clique-pair budget and extra-core ledger all apply unchanged, and a
sweep confirmed raising them adds nothing (held-out 0.850365 at both 1M/300k and 2M/750k).

## Measurement

| corpus | base `a2614af` | candidate | better | worse |
| --- | --- | --- | --- | --- |
| development (300) | 0.827195 | **0.826709** | 2 | 0 |
| held-out (591) | 0.852139 | **0.850365** | 1 | 0 |

Identical to the unbounded version, confirming the cap is free. Gains: transswitch2736spr 6.71%,
transswitch2383wpr 0.32%, powerflow2736spr 9.86%. 68 tests; trusted 300-case run 0.8267, fill 0.9374.

Interleaved minimum-of-ten on the admitted rows: no row slower by more than 0.05 s
(powerflow2736spr +0.030, powerflow2383wpr +0.049, transswitch2383wpr +0.024).

## Not done, deliberately

powerflow2383wpr costs 0.049 s and gains nothing, so raising the band's lower edge would cut exposure
for free. It is not done: the evidence is two rows against two, there is no mechanism behind a floor,
and fitting a constant to the rows one can see is what the failed attempt did.
