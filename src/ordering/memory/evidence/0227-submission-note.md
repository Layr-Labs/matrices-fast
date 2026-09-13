# Deferred-lift parity: give the held stage-1b lift the same pre-terminal polish, and compare like with like

**Model / effort:** DeepSeek V4 Flash, single-agent loop (angelX harness).
**Base:** the promoted frontier (my own `fe4f40c`, hidden 0.841858) plus the pending
sparse-span schedule extension (9 windows). This change is on top of the current tree.
**Local score:** `yukon run` 300/300 **OK 0.791480 / 0.924039** (`results.tsv:1789261328`);
the same tree with the device switched off reads **0.791586** in-frame
(`probe_timing_and_score`, one binary, one session, `taskset -c 0-3`, 300 dev rows).

## 1. Where the value comes from (measured, not assumed)

The pipeline has one binary decision that reshapes a whole row: at stage 1b the
independent-set "lift" is either

* **adopted immediately** — the force gate (`n >= 20_000`) or a 10 % exact margin
  (`INDEP_IMMEDIATE_MARGIN`), after which *the lift* runs the rest of the pipeline, or
* **held** (`indep_deferred`) — the portfolio incumbent runs the descent / search /
  subtree stages, and at 4b the lift's **1b score** is compared against the
  incumbent's **post-polish** score; if the raw lift wins it is installed **raw**,
  i.e. without any of the polish the incumbent just received.

Those two trajectories score differently on dev (production frame 0.791635 vs the
deferred frame 0.791851 on the same binary), and the per-row minimum of the two is
0.791385 — a 2.5e-4 / 3.1-bip ceiling. Earlier attempts to pick the winner *before*
running the cascade were measured dead (+1.17e-3): the post-4b stages invert the
pre-4b ranking (see `memory/experiments/0219-early-arbitration.md`), and no cheap
observable separates the two classes. So the only sound way to get the value is to
give the held lift the same treatment the adopted one gets, which is what this
submission changes.

## 2. The change

`src/ordering/mod.rs`. The pre-terminal polish (2.descent + 3.search + the 4.subtree
chain, i.e. the ~530 lines between the end of stage 1b and the 4b comparison) is now a
unit — `macro_rules! pre_terminal_polish` — and the 4b site runs it on the held lift
when the raw lift wins:

```rust
if let Some((f, cand)) = indep_deferred.take() {
    if f < best_flops {
        if indep_parity_off() {            // test-only A/B switch; production = false
            best_flops = f; best_perm = cand;      // the old raw rule
        } else {
            let inc_flops = best_flops;            // polished incumbent
            let inc_perm = std::mem::replace(&mut best_perm, cand);
            best_flops = f;                        // raw lift
            pre_terminal_polish!();                // same polish, same gates, same order
            if best_flops >= inc_flops {           // keep the better polished candidate
                best_perm = inc_perm; best_flops = inc_flops;
            }
        }
    }
}
```

The macro is expanded in the ordinary position as well, so the polish applied to the
incumbent is *byte-identical* to the polish applied to the lift (same gates on the
same `n`, `nnz`, `amd_flops`, same budgets, same seeds). `SSI_INDEP_PARITY=0` is a
`#[cfg(test)]`-only restore switch; the graded build compiles `indep_parity_off() ->
false`, so parity is on. No environment, clock, or identity input reaches the
decision: the choice is an exact integer-flop comparison of two deterministic
candidates. Cost is one extra polish pass and only on rows whose lift wins the 4b
comparison (production defers only on `n < 20_000`, so the device is confined to rows
the force gate does not already cover).

## 3. Result (exact, per-row, same binary / session / frame)

`probe_timing_and_score`, `SSI_INDEP_FORCE=1` (production frame), 300 dev rows,
`taskset -c 0-3`; the two arms differ only by `SSI_INDEP_PARITY`:

| arm | score | lt_1k | 1k_10k | gt_10k | worst `order()` |
|---|---:|---:|---:|---:|---:|
| parity OFF (shipped behaviour) | 0.791586 | 0.8873 | 0.8377 | 0.6852 | 1.397 s |
| parity ON (this submission) | **0.791480** | 0.8874 | 0.8377 | **0.6849** | 1.333 s |

exact `COUNTS` diff (integer flops, no timing involved): **6 rows better, 1 worse**,
`delta = -1.06e-4` (−1.34 relative bips):

| row | n | nnz | flops before -> after | delta |
|---|---:|---:|---:|---:|
| methanol200 | 11999 | 76128 | 905945 -> 897919 | −0.894 % |
| crudeoil_lee4_10 | 17809 | 120632 | 184173410 -> 182784437 | −0.760 % |
| torsion50 | 2508 | 29808 | 1118955 -> 1116757 | −0.197 % |
| graphpart_3g-0244-0244 | 128 | 672 | 32534 -> 32474 | −0.185 % |
| crudeoil_lee4_09 | 15904 | 101792 | 130655525 -> 130575933 | −0.061 % |
| glider400 | 10017 | 49624 | 327461 -> 327399 | −0.019 % |
| wastewater05m1 | 98 | 536 | 8002 -> 8033 | **+0.39 %** |

Local harness receipt: `yukon run` → `Benchmark complete (score: 0.79148)`, per-bucket
`0.8874 / 0.8377 / 0.6849`, `results.tsv:1789261328 OK 0.791480 0.924039`, and the
crate suite is `123 passed / 0 failed / 55 ignored`.

## 4. The one regression, and why it is worth recording

`wastewater05m1` (n=98, nnz=536) is the honest counterexample to "a better polished
candidate is a better candidate". Instrumented trace of the device on that row:

```
PARITY  n=98  raw=8771  inc=8828  plift=8111  margin_ppm=81218
```

The polish makes the lift 8.1 % better than the polished incumbent, so parity adopts
it — yet the row ends **worse** than the raw rule (8033 vs 8002 flops). The phase dump
(`SSI_PROBE_PHASES`) shows why: under the raw rule the terminal ladder takes the row
from 0.6874 to 0.6845 (its own search, −0.42 %), while from the parity winner it takes
it 0.6872 -> 0.6872 (−0.00 %). The terminal window-descent chain is a *path-dependent
local search*: a better starting permutation can be a worse basin, and the ladder
found 8× more value from the un-polished lift. That is the same inversion the
early-arbitration experiment measured at 4b, now observed one stage later. Every other
row in the corpus is bit-identical between the arms (293/300 unchanged), so this is
the device's only exposure.

## 5. Caveats

* The 1-bip improvement bar is relative, and this device's dev value is 1.34 bips
  before hidden transfer — it is close to the bar, not far above it.
* The probe is an in-process frame; the authoritative per-row receipt here is
  `yukon run` (one child process per matrix), which agrees with it to 4 decimals.
* Cap safety: the device fires only on rows the force gate leaves deferred (all
  `n < 20_000`, the largest being lee4_10 at 1.23 s probe / ~1.28 s with parity) and
  it did not create a new worst row in either arm (worst rows are
  `chimera_selby-c16-02` 1.397 s off / 1.333 s on — host noise, same row).
* The next step this makes cheap: the same "run both, keep the better" rule applied
  to the *terminal* tail (raw lift vs polished lift) would capture the per-row optimum
  on these 7 rows exactly, at one extra tail per firing row instead of one extra
  polish — that is the natural next bat if this one is accepted.

## 6. Reproduction

```bash
# one binary, two arms, production frame, 4 vCPU
SSI_INDEP_FORCE=1 SSI_INDEP_PARITY=0 taskset -c 0-3 bash target/probe-sandbox.sh run   # 0.791586
SSI_INDEP_FORCE=1                    taskset -c 0-3 bash target/probe-sandbox.sh run   # 0.791480
# official local receipt
yukon run
```

Evidence: `memory/evidence/0227-parity-off-4cpu.log`, `0227-parity-on-4cpu.log`,
`0227-yukon-run-parity.log`, `score.json`, `results.tsv:1789261328`.
