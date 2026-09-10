# 0148 — Fixed low-width terminal budget after two hidden timeouts

## Outcome and submission target

This is the second runtime reduction of the terminal exact-window experiment.
It starts from promoted source `62654a5`, whose hidden score is **0.843173** and
whose current development score is **0.792300**. The production candidate
scores **0.792241** on all 300 development matrices, a **0.59-bip** reduction.

Submission `113841b9` carried this exact package. Its hidden Benchmark step
again failed because one `order()` call exceeded the two-second per-matrix cap.
That third failure showed that merely removing all new work above 3,000
vertices was insufficient: an inherited path in the promoted pipeline also
needed deterministic headroom. Follow-up [0149](0149-dense-alternate-peo-headroom.md)
profiles and removes one score-neutral inherited loop on large dense rows.

The new work consists of three low-width passes with an 8M operation allowance
each on `n <= 3,000 && nnz <= 80,000`. Rows through 512 vertices receive one
additional width-12 pass with the same allowance. The maximum total new
allowance is therefore 24M on ordinary eligible rows and 32M on tiny rows. All
medium and large rows receive no new work.

This package deliberately keeps less development gain than 0146 or 0147. Both
of those packages timed out remotely. The retained chain is the best measured
combination among the small, low-budget variants and adds only 4.1 ms at its
slowest development row; the separate tiny pass measured 2.4 ms at worst.

## Remote evidence that changed the design

The first package, [0146](0146-multiscale-terminal-window-descent.md), added ten
passes with widths 6 through 14 and allowances from 32M through 96M throughout
the existing `n <= 12,000 && nnz <= 200,000` terminal envelope. It scored
0.791984 locally, but submission `42491bf1` failed when one hidden `order()`
call exceeded two seconds.

The first retry, [0147](0147-two-tier-window-runtime-envelope.md), kept the ten
passes only at `n <= 3,000 && nnz <= 80,000` and reduced sparse medium rows to
two passes. It scored 0.792076 locally. Submission `a1f928e2` nevertheless
failed with the same message:

```text
RUN FAILED: hidden matrix: order() exceeded the 2.0s per-matrix cap and was killed
```

The two benchmark steps both ran for about 106 seconds before the kill. Their
shared added work was the full ten-pass small tier, while the retry also had a
two-pass sparse-medium tier. The remote logs do not reveal the hidden matrix,
so they cannot identify one gate conclusively. They do show that retaining the
high-budget small tier is unsafe. This version removes both suspect branches
rather than guessing which one was encountered at that corpus position.

## Low-budget screen

A temporary test-only switch disabled the added multiscale stage and produced
the promoted parent ordering. Each candidate pass or chain then refined that
same finished incumbent, and exact symbolic scoring accepted only strict flop
reductions. The probe evaluated every development row and aggregated the same
three bucket geometric means and 0.3/0.3/0.4 weights as the benchmark.

Every individual pass below used four sweeps and an 8M operation allowance.
The score column applies the pass through 3,000 vertices and leaves every other
row at the promoted result.

| candidate | stride | strict wins | projected score | gain vs parent | worst added time |
|---|---:|---:|---:|---:|---:|
| width 6 | 1 | 8 | 0.792279376 | 0.203 bip | 1.6 ms |
| width 7 | 3 | 9 | 0.792265138 | 0.345 bip | 1.6 ms |
| width 8 | 3 | 8 | 0.792252316 | 0.473 bip | 1.6 ms |
| width 9 | 4 | 8 | 0.792270696 | 0.289 bip | 1.8 ms |
| width 10 | 3 | 4 | 0.792282660 | 0.170 bip | 2.1 ms |
| width 11 | 5 | 4 | 0.792292547 | 0.071 bip | 2.2 ms |
| width 12, signature charge | 1 | 3 | 0.792282404 | 0.172 bip | 2.4 ms |
| width 12, ordinary charge | 6 | 2 | 0.792290083 | 0.095 bip | 2.3 ms |
| width 14, signature charge | 5 | 1 | 0.792299605 | negligible | 2.3 ms |
| width 14, ordinary charge | 7 | 1 | 0.792299605 | negligible | 2.2 ms |

The low-width sequence 6/7/8, each at 8M, scored **0.792249024** with
11 strict wins and a 4.1 ms maximum combined time. A width-9/11 pair scored
0.792269853, and a six-pass sequence with only 4M per pass scored 0.792270126.
The 6/7/8 sequence therefore provided the best score at a 24M total allowance.

The screen also simulated progressively smaller dimension gates. The 6/7/8
sequence scored 0.792257517 through 2,000 vertices and 0.792249024 through
3,000. It found no low-width win below 1,000. A separate width-12/stride-1 pass
did find one strict win below 512 and scored 0.792291948 by itself. Because it
runs only on tiny rows and has an 8M allowance, it was appended after the
low-width sequence there.

The temporary switch and screen were removed before the production build.

## Production policy

The existing promoted terminal passes remain unchanged. After them, rows under
this gate:

```text
n <= 3,000
nnz <= 80,000
```

receive the following fixed sequence:

| width | sweeps | stride | operation allowance | charge model |
|---:|---:|---:|---:|---|
| 6 | 4 | 1 | 8M | signature |
| 7 | 4 | 3 | 8M | signature |
| 8 | 4 | 3 | 8M | signature |

Rows satisfying the same density ceiling and `n <= 512` then receive:

| width | sweeps | stride | operation allowance | charge model |
|---:|---:|---:|---:|---|
| 12 | 4 | 1 | 8M | signature |

There is no sparse-medium branch. A row above 3,000 vertices pays zero new
setup cost. A small row can spend at most 24M charged operations, and a tiny
row at most 32M. Budget exhaustion preserves completed strict gains and returns
without attempting the remaining windows.

## Why low width reduces runtime risk

Each exact window is split into connected components, and a component of size
`k` uses subset dynamic programming with state count exponential in `k`. The
previous versions included width 14 and allowed up to 96M charged operations
for a single pass. They also chained as many as ten passes, so a hidden row that
reached those allowances could consume hundreds of millions of charged units.

This version caps the ordinary eligible path at three calls, never exceeds
width 8 there, and lowers every per-call allowance to 8M. The tiny width-12
call runs on at most 512 vertices. These are structural and operation-count
bounds; they do not inspect elapsed time and behave identically on every run.

The direct screen measured candidate-stage time rather than total `order()`
time. Its 4.1 ms maximum is about 27 times smaller than the 112M sparse-medium
allowance in 0147 and about 24 times smaller in charged work than just the two
largest passes of the original ten-pass chain. More significantly, rows above
3,000—the class containing every slow development matrix—receive no added
work at all.

## Score evidence

The final sandboxed `yukon run` result is:

| metric | promoted parent | fixed-budget candidate | change |
|---|---:|---:|---:|
| weighted flop score | 0.792300 | **0.792241** | **-0.000059** |
| weighted fill tiebreak | 0.924373 | **0.924360** | lower |

Representative retained strict improvements include:

- `pooling_sppa0pq`: 1,234,887 to 1,226,861 flops;
- `crudeoil_pooling_ct3`: 869,821 to 868,337 flops; and
- `hydroenergy2`: 55,946 to 55,877 flops.

The improvement is smaller than the failed candidates because all gains on
medium rows and all high-budget small-row refinements were removed. It is still
a strict aggregate improvement over the source of the current hidden best.

## Correctness and determinism

The implementation calls the existing `subset_window_descent_step` routine.
That routine validates its sparse pattern and seed, maintains the exact live
elimination graph, solves bounded windows, and returns a deterministic
permutation. Each returned candidate is scored by the production symbolic
score workspace. It replaces the incumbent only when its flop count is
strictly lower.

All control flow depends only on `n`, `nnz`, fixed constants, and exact score
comparisons. It uses no matrix name, corpus position, wall clock, external
state, hashing, or randomness. The implementation remains within
`src/ordering/` and uses the Rust standard library only.

## Validation

The exact production tree was checked with:

```sh
yukon run
SSI_ALLOW_UNSANDBOXED_WORKER=1 cargo test --release -p ssi-candidate-worker
SSI_ALLOW_UNSANDBOXED_WORKER=1 cargo test --release \
  -p ssi-candidate-worker probe_timing_and_score \
  -- --ignored --nocapture --test-threads=1
```

The Yukon run completed all 300 development matrices at **0.792241** with a
**0.924360** fill tiebreak. The direct production probe reproduced the score
and measured a **0.815-second** worst complete `order()` call. The production
worker suite passed **118 tests** with 39 measurement tests ignored. The
temporary low-budget screen completed the full corpus and was removed before
these production checks.

## Remote result and next step

The hidden corpus and runner differ from development, and the promoted parent
already has substantial per-row work. Submission `113841b9` confirmed that
the fixed terminal budget alone did not provide enough remote headroom. The
failure is especially informative because every row above 3,000 vertices paid
zero new terminal-window work in this package. Follow-up 0149 therefore keeps
this entire score-positive fixed budget and profiles the inherited production
pipeline on the slow public analogues. It skips an alternate-seed PEO chain
only where that chain changed no public final ordering, reducing the measured
worst complete call from the prior 0.8-second band to 0.712 seconds without
changing the 0.792241 development score.

Effort: high.
