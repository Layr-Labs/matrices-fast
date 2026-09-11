# 0151 — Subtractive terminal window exchange

## Outcome and submission target

This candidate scores **0.792190638** on the exact development screen and
**0.792191** in `yukon run`, with a **0.924328** fill tiebreak. The current
hidden leader is still submission `3085f82`, score **0.843173**, from promoted
source `62654a5`. That source reproduces at 0.792299626 locally, so the new
schedule improves the weighted flop score by **0.000108988**, or 1.090 absolute
basis points.

The terminal schedule now contains three passes with a summed deterministic
allowance of **120M**. The promoted parent has four passes totaling 136M. The
candidate removes both the inherited width-8/16M and width-12/32M union-charged
passes, retains width-10/24M and width-12/64M, then runs the new
width-8/stride-3/32M signature-charged pass. It is therefore a strictly
subtractive work package: 16M, or 11.8%, below the terminal allowance of the
hidden-proven parent.

Against the parent, 32 development rows improve and two regress by tiny counts.
The gains span pooling, crude-oil, chimera, synthetic, power-flow, and other
families. A direct production timing probe completed all 300 rows at the same
score and measured **0.735 seconds** as the worst complete `order()` call. In
the same-seed terminal screen, this schedule's isolated maximum was 0.02128
seconds versus 0.02851 seconds for the parent.

## Why the equal-budget attempt was insufficient

Five preceding descendants failed the hidden two-second cap:

| experiment | submission | terminal relation to parent | hidden result |
|---|---|---|---|
| 0146 | `42491bf1` | additive broad chain | timeout |
| 0147 | `a1f928e2` | additive two-tier chain | timeout |
| 0148 | `113841b9` | additive low fixed budgets | timeout |
| 0149 | `3a4f3ed0` | same additive chain plus unrelated headroom cut | timeout |
| 0150 | `c8739463` | equal 136M terminal allowance | timeout |

Experiment 0150 was the decisive correction. It removed an inherited 32M
width-12 pass and inserted a 32M width-8 pass, so its allowance sum exactly
matched the promoted parent. It nevertheless failed after 99.20 seconds in the
Benchmark step, at essentially the same hidden corpus position as the other
attempts.

The work counter is deterministic and enforces each pass's own bound, but two
different pass shapes do not translate one charged unit into identical CPU
time. Union-parity and signature-true charging fund different components. A
width-eight pass visits more small windows before exhausting a budget, and a
hidden boundary halo can change extraction cost, memo reuse, and the number of
funded exact solves. Public aggregate timing did not expose the offending
shape. Matching the parent's allowance was therefore insufficient evidence;
the next package needed to be measurably below it.

This candidate removes a complete 16M pass. It does not add a new gate inferred
from public matrix names or attempt another unrelated phase cut. Every row in
the terminal envelope receives a lower allowance and one fewer pass than the
promoted schedule.

## Exact terminal schedules

The promoted source runs:

| order | window | sweeps | stride | charge model | allowance |
|---:|---:|---:|---:|---|---:|
| 1 | 8 | 2 | 4 | union parity | 16M |
| 2 | 12 | 2 | 6 | union parity | 32M |
| 3 | 10 | 2 | 5 | union parity | 24M |
| 4 | 12 | 4 | 5 | signature true | 64M |
|  |  |  |  | **total** | **136M** |

The failed equal-budget exchange ran:

| order | window | sweeps | stride | charge model | allowance |
|---:|---:|---:|---:|---|---:|
| 1 | 8 | 2 | 4 | union parity | 16M |
| 2 | 10 | 2 | 5 | union parity | 24M |
| 3 | 12 | 4 | 5 | signature true | 64M |
| 4 | 8 | 4 | 3 | signature true | 32M |
|  |  |  |  | **total** | **136M** |

The new production schedule runs:

| order | window | sweeps | stride | charge model | allowance |
|---:|---:|---:|---:|---|---:|
| 1 | 10 | 2 | 5 | union parity | 24M |
| 2 | 12 | 4 | 5 | signature true | 64M |
| 3 | 8 | 4 | 3 | signature true | 32M |
|  |  |  |  | **total** | **120M** |

All three schedules use the same structural gate:
`6 <= n <= rgreedy::MAX_N && nnz <= 200,000`. The new bound therefore applies
uniformly to every terminal-eligible hidden row. Each pass also charges graph
construction, refuses components it cannot fund, and returns only a completed
strict improvement. The 120M sum is an upper envelope rather than a promise to
spend the full amount.

## Same-seed schedule screen

The diagnostic captured each row immediately before terminal windows and ran
every schedule from that identical permutation. Earlier candidate generation
was executed once per row, preventing runtime noise or trajectory changes in
the large portfolio from contaminating the comparison.

| schedule | allowance | score | wins / losses vs parent | isolated worst |
|---|---:|---:|---:|---:|
| promoted `A+B+C+D` | 136M | 0.792299626 | 0 / 0 | 0.02851 s |
| remove B, no replacement `A+C+D` | 104M | 0.792298937 | 1 / 4 | 0.02037 s |
| failed equal exchange `A+C+D+E32` | 136M | **0.792188201** | 32 / 1 | 0.02303 s |
| **selected `C+D+E32`** | **120M** | **0.792190638** | **32 / 2** | **0.02128 s** |
| reduce E to 24M `A+C+D+E24` | 128M | 0.792207263 | 28 / 2 | 0.02282 s |
| reduce D to 48M `A+C+D48+E32` | 120M | 0.792222544 | 28 / 9 | 0.01985 s |
| shift work D48/E40 | 128M | 0.792200734 | 34 / 8 | 0.02073 s |
| remove C instead `A+D+E32` | 112M | 0.792246063 | 27 / 12 | 0.01833 s |

`A`, the removed width-8/16M pass, contributes almost nothing once the later
width-8 signature pass is present. Deleting it changes the failed equal-budget
score by only **0.000002437**, about 0.024 basis point, while buying the entire
16M safety margin. Reducing the replacement itself to 24M loses about eight
times as much aggregate score. Reducing the retained width-12 pass creates more
losses and falls below the observed promotion scale. The selected schedule is
the cleanest point on the measured score/work frontier.

## Per-row behavior

Representative exact improvements over promoted source `62654a5` are:

| matrix | promoted flops | candidate flops | change |
|---|---:|---:|---:|
| `pooling_sppa0pq` | 1,234,887 | 1,217,666 | -17,221 |
| `crudeoil_lee2_06` | 17,666,549 | 17,659,239 | -7,310 |
| `chimera_mgw-c16-2031-01` | 2,600,520 | 2,598,618 | -1,902 |
| `chimera_k64ising-02` | 372,954 | 371,241 | -1,713 |
| `crudeoil_pooling_ct3` | 869,821 | 868,337 | -1,484 |
| `chimera_lga-01` | 497,542 | 496,328 | -1,214 |
| `crudeoil_lee1_07` | 3,553,352 | 3,552,322 | -1,030 |
| `chimera_selby-c16-01` | 2,431,621 | 2,430,714 | -907 |
| `edgecross10-030` | 100,608 | 100,139 | -469 |
| `transswitch0300p` | 378,674 | 378,205 | -469 |

The two regressions are tiny trajectory effects: removing an early strict pass
changes the seed seen by later strict passes. `chimera_mgw-c8-439-onc8-002`
increases by 30 flops, and `syn30m03m` increases by four. Each pass remains
strictly monotone against its own current incumbent, every final permutation is
valid, and the exact weighted aggregate improves by more than one basis point.

## Correctness, determinism, and scope

The production decision uses only the supplied pattern, fixed integer
parameters, and exact symbolic scores. It reads no clock, matrix label, corpus
position, environment state, randomness, or unordered iteration. A pattern
therefore returns the same permutation in both grader invocations.

Window refinement preserves the vertices outside each window and returns the
inside vertices once each. The outer selector accepts a candidate only when its
trusted symbolic flop count is strictly below the current best. Exhaustion is
fail-closed: an unfunded graph build or component cannot introduce a partial
unverified ordering. Existing exhaustive tests compare the window dynamic
program with independent small-graph oracles and cover malformed inputs,
budget boundaries, determinism, and bijection.

Only `src/ordering/` is changed, and the implementation uses the Rust standard
library already present in the module. Temporary pre-window capture and
multi-schedule screen code was removed; `src/ordering/probe.rs` has no diff.
The dense alternate-PEO guard from 0149 remains because it is public-score
neutral and removes inherited work on large dense rows.

## Validation

The production tree was checked with:

```sh
yukon run
SSI_ALLOW_UNSANDBOXED_WORKER=1 cargo test --release \
  -p ssi-candidate-worker probe_timing_and_score \
  -- --ignored --nocapture --test-threads=1
SSI_ALLOW_UNSANDBOXED_WORKER=1 cargo test --release \
  -p ssi-candidate-worker
git diff --check -- src/ordering
```

`yukon run` completed all 300 matrices at **0.792191**, with a **0.924328**
fill tiebreak. The direct probe reproduced the exact aggregate score and
reported **0.735 seconds** as the worst complete `order()` call. The release
worker suite passed **118 tests**, with 39 measurement probes ignored and no
failures. The final source diff check is clean.

The candidate retains 97.8% of the failed equal-budget experiment's public
score gain while lowering the terminal allowance by 11.8% versus the promoted
parent. Its 32 development wins provide breadth, and the aggregate delta clears
one absolute basis point. Hidden translation remains corpus-dependent, but the
runtime argument now rests on strictly less bounded work rather than equal
allowance or an unrelated public timing cut.

This work was performed with GPT 6 through Codex at high reasoning effort. It
used exact same-seed schedule screens, deterministic work accounting, direct
production timing, and the full release worker suite.

Effort: high.
