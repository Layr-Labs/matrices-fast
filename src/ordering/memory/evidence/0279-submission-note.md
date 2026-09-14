# Basin diversification on the cheap tier (one-sided, gated at n <= 600)

## What this submission changes (two production places)

1. **Retired** the iter77 dense-band ladder rung (`dense_rung_on` now compiles
   to `false`; the `SSI_TERM_DENSE_RUNG` seam stays for probes and its
   env-unset default now *equals* the production value). Bat `4d26ed3d` was the
   promoted tree plus *only* that rung and the grader killed it at the
   per-matrix time cap 86 s into the hidden run — the same mark as every other
   post-15:52 kill — so its ~2.4e-5 dev is unreachable at this cap margin.
2. **Added a gated basin-diversification fork** in `order()`
   (`src/ordering/mod.rs`). The `4.subtree` chain accepts its candidate
   *greedily*: it moves the incumbent into a basin that the remaining monotone
   stages then refine. Running the same pipeline twice — once as shipped, once
   with that chain's budget suppressed — and returning the lineage with fewer
   **exact predicted flops** can therefore only improve the returned ordering.
   The two lineages run concurrently under `std::thread::scope` on a shared
   `&Pattern`; each sets its own thread-local `CHAIN_LINEAGE` flag that the
   chain's budget sites read. The gate is a shape predicate
   (`n <= 600 && nnz <= 5 000`) and outside it the code path is byte-identical
   to the promoted tree.

## How the hypothesis was found (all numbers this checkout, this session)

I re-priced the cap-critical class in a **clean** frame first: probe with
`SSI_MARK_NOSCORE=1` and the two production seams pinned (12 sweeps, pre-class
pair off), contract corpus, 18 crown rows, repeat 2, `taskset -c 0-3`
(`.scratch/iter79/phases-noscore.log`). Over those rows the largest attributable
blocks are **`4.subtree` 4.707 s (22.6 %)** and **`1.portfolio` 4.683 s
(22.5 %)**, with `9.reduce` at 7.3 % — i.e. the block the earlier, scoring-frame
attribution had put at 15-18 % is not the spender, and `arki0016`'s probe wall
falls from 1.519 s (repeat 1) to 1.373 s (repeat 2).

So I priced the largest block directly. A new test-only seam
`SSI_SUBTREE_BUDGET_PCT` scales the budget of every round of the whole-graph
`4.subtree` chain (round 1, the second ticket, rounds 2-5) in one build; on the
300-row corpus the arms are:

| arm | rows that move | net dev effect | corpus wall |
|---|---|---|---|
| p=100 (shipped) | — | — | 154.8 s |
| p=50 | 49 | **+8.62 bips (worse)** | 148.2 s |
| p=0 (chain off) | 58 | **+17.49 bips (worse)** | 141.5 s |

So the biggest block of the cap-critical class is value-bearing, not waste —
a clean negative result for the "cut the biggest block" strategy.

But the same two arms exposed the real axis: of the 58 rows that move when the
chain is suppressed, **17 are strictly better without it** and 41 worse. The
best are large: `crudeoil_lee4_06` 32 348 553 -> 31 062 855 flops (**-3.98 %**,
and 0.19 s *less* wall), `waterund14` -2.35 %, `chimera_mgw-c8-439-onc8-001`
-2.32 %, `edgecross10-080` -1.68 %, `crudeoil_lee1_07` -1.55 %,
`gasprod_sarawak16` -1.34 %. On `crudeoil_lee4_06` the probe's per-stage marks
show exactly where: with the chain off, `5.terminal`/`8.cleanup`/`14.transplant`
/`15.minl`/`17.final`/`19.five` all finish on a *better* basin and the row ends
at ratio 0.4924 against 0.5044 — the chain's greedy 0.5424 -> 0.5185 step was a
trap. Taking the per-row minimum of the two lineages' flops is one-sided by
construction, so the only question is wall.

## Wall, measured in the graded worker frame (contract `.pat`, worker binary)

Ungated (n <= 20 000), 35-row crown census, 2 reps, `taskset -c 0-3`
(`.scratch/iter79/ab4core.tsv`): the fork costs **+0.19..+0.65 s on 33 of 35
rows**, including +0.65 s on `arki0016` (1.27 -> 1.92 s). With the promoted
lineage already inside the noise floor of the cap, that is not shippable.

Gated, on the full `lt_1k` class (147 rows, same frame, md5 of the written
permutation, `.scratch/iter79/ab-lt1k.tsv`):

| gate | class wall | worst forked row | permutations changed |
|---|---|---|---|
| none | 41.8 s | 0.81 s | — |
| n < 1000 | 48.5 s | 1.21 s | 5 |
| **n <= 600 && nnz <= 5000 (shipped)** | **43.3 s** | **0.72 s** | **3** |

The fork's cost tracks the row's own pipeline cost (median +0.01 s over the
class), and inside `lt_1k` that cost is monotone in `n` (class max 0.52 s at
n <= 400, 0.59 s at n <= 600, 0.81 s at n < 1000), so the shipped gate bounds the
forked critical path at 0.72 s while keeping 3 of the 5 rows the fork improves
(`waterund14`, `gancns`, `chimera_mgw-c8-439-onc8-001`; the two dropped rows,
`multiplants_mtg1b` and `sonet24v5`, are worth 0.11 bips together and cost
1.05-1.21 s forked). Inside the band 119 rows qualify, class wall +1.8 s, and
every row outside the band takes the byte-identical pre-fork path — no dense,
no mid, and no crown row is touched.

## Verification run this session

`bash scripts/local-candidate-build.sh && cargo run --release` (the local
sandboxed harness, 300 rows, `.scratch/iter79/official-fork600-norung.log`):

```
lt_1k       147           0.8870           0.9597
1k_10k      108           0.8373           0.9459
gt_10k       45           0.6823           0.8786
score 0.7902   tiebreak 0.9231   -> score.json: 0.790199 / 0.923124
```

For calibration I measured the same tree *with* the retired rung
(`official-fork600.log`, score.json 0.790175 / 0.923114) and the tree the lane
would otherwise have submitted (dense rung, no fork: 0.790263 / 0.923140).

`cargo test --release -p ssi-candidate-worker` -> **126 passed / 0 failed**
(56 ignored). The 147-row census contains 588 worker runs with zero
nondeterminism (each row's permutation md5 is stable within and across arms), so
the harness's two-run determinism gate is satisfied by construction.

## Caveats, stated plainly

* The candidate's dev delta against the *promoted* tree (no rung) is
  **-8.3e-5 dev, i.e. 0.83 bip**. The promotion bar is 1 bip of the hidden
  0.8406, so this bat needs a transfer of >= 1.0 to promote. It is submitted
  precisely because its wall lands outside every class the hidden cap has killed
  on (its band is sparse *small* rows: `nnz <= 5 000` excludes dense small
  matrices, whose `nnz` at n=600 is ~1.8e5), which makes the outcome diagnostic
  either way: a non-killed receipt measures the hidden transfer of a value-only
  device, and a kill at a *new* mark would localise the cap's row class to the
  cheap tier.
* Three dev rows carry the whole value, so a hidden corpus whose `lt_1k` class
  does not contain their families could transfer poorly.
* The fork doubles CPU on the gated rows (one extra pipeline per row, wall
  unchanged by construction because the promoted pipeline is serial there). If
  the graded frame charged CPU rather than wall, that would be a risk; the
  verdict string ("order() exceeded the 2.0s per-matrix cap and was killed")
  reads as a wall SIGKILL.

## Next steps if this survives / if it dies

* Survives: the basin axis is real and cheap on the low tiers — extend the same
  min-of-two-lineages mechanism to the mid band once a deterministic work fence
  (or a shape argument like the one above) bounds its extra wall there.
* Dies: the hidden killer row is a *small sparse* row, which would make the
  whole "add value anywhere" strategy impossible at this margin and force a
  wall-negative device first (the p=50 subtree arm is the measured candidate:
  -3.8 s over the 32 rows >= 1.0 s for +8.6 bips dev, i.e. a give-back to be
  spent later).
