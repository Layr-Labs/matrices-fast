# 0147 — Two-tier runtime envelope for terminal exact windows

## Outcome and submission target

This is the runtime-reduced successor to
[0146](0146-multiscale-terminal-window-descent.md). It starts from promoted
source `62654a5`, whose hidden score is **0.843173** and whose freshly measured
development score is **0.792300**. The reduced package scores **0.792076** on
all 300 development matrices, a **2.24-bip** reduction, while limiting the full
multiscale search to small, bounded-density rows and using only two short passes
on sparse medium rows.

The direct timing probe reports **0.798 s** at the slowest local call. The two
slowest rows, `crudeoil_lee4_09` and `crudeoil_lee4_10`, do not enter either new
gate. The implementation remains a terminal, deterministic, strict-accept
refinement; a candidate cannot worsen the flop score of any matrix.

## Why this retry exists

The first multiscale package ran ten exact window passes wherever the promoted
terminal window stage ran: `n <= 12,000 && nnz <= 200,000`. It achieved a
strong local score of **0.791984** (3.16 bips better than the promoted parent)
and a local worst call of 0.803 s. Submission `42491bf1`, however, failed the
hidden benchmark with:

```text
RUN FAILED: hidden matrix: order() exceeded the 2.0s per-matrix cap and was killed
```

The failure occurred in the benchmark step after setup, dependency validation,
sandbox verification, and hidden corpus fetch had all passed. This isolates the
problem to the additive runtime of the broad ten-pass chain. The exact-score
logic and local correctness tests were not implicated.

Simply reducing every operation budget would be difficult to reason about: a
budget-cut pass can stop before reaching the useful windows, and different
widths have different setup and dynamic-program costs. Instead, this retry
keeps complete passes and reduces how many passes each structural tier receives.

## Paired gate measurement

A test-only switch ran the promoted parent and the full ten-pass candidate
back-to-back on every development matrix. Because both versions were evaluated
in one process, the per-row flop deltas could be aggregated under alternative
structural gates without repeatedly editing production code. The relevant
results were:

| simulated gate for the ten-pass chain | strict wins | projected score | gain vs parent |
|---|---:|---:|---:|
| all original eligible rows | 48 | 0.791984 | 3.159 bip |
| `n <= 3,000` | 21 | 0.792138 | 1.624 bip |
| `n <= 6,000 && nnz <= 20,000` | 31 | 0.792139 | 1.610 bip |
| `n <= 6,000 && nnz <= 50,000` | 33 | 0.792099 | 2.008 bip |
| `n <= 3,000` or `nnz <= 20,000` | 35 | 0.792089 | 2.109 bip |
| `n <= 3,000` or sparse through 12k | 44 | 0.792031 | 2.691 bip |

The last row defined sparse as `nnz <= 50,000 && nnz <= 5*n`. It retained most
of the full package's score, but it still assigned all ten passes to medium
matrices. Since hidden timing had already rejected that amount of work, the
production retry uses the structural split but gives the sparse-medium tier
only two passes.

The small tier also has an `nnz <= 80,000` ceiling. The denser small development
rows did not contribute useful wins, and excluding them removes a class where
the surrounding promoted pipeline may already be expensive.

## Selected two-tier policy

The existing promoted width-12/stride-5 pass is unchanged. The added chain is
selected as follows inside the parent's outer `n <= 12,000 && nnz <= 200,000`
envelope:

### Tier 1: small bounded-density rows

Gate:

```text
n <= 3,000 && nnz <= 80,000
```

These rows receive the complete ten-pass chain:

| width | sweeps | stride | budget | charge path |
|---:|---:|---:|---:|---|
| 12 | 4 | 1 | 64M | signature |
| 12 | 4 | 6 | 64M | ordinary half-width |
| 14 | 4 | 5 | 96M | signature |
| 14 | 4 | 7 | 96M | ordinary half-width |
| 9 | 4 | 4 | 48M | signature |
| 11 | 4 | 5 | 64M | signature |
| 8 | 4 | 3 | 32M | signature |
| 10 | 4 | 3 | 48M | signature |
| 7 | 4 | 3 | 32M | signature |
| 6 | 4 | 1 | 32M | signature |

An exhaustive follow-up screen of all stride choices for widths 6 through 11
on the finished small-tier incumbent found the search essentially converged.
The largest remaining independent configuration was width 11/stride 10 at only
0.067 development bip. No extra pass from that screen was added.

### Tier 2: sparse medium rows

Gate:

```text
n > 3,000
nnz <= 50,000
nnz <= 5*n
```

These rows receive only:

```text
width 9,  four sweeps, stride 4, 48M operation budget
width 11, four sweeps, stride 5, 64M operation budget
```

Those were the two highest-yield configurations in the independent screen on
the wide-window incumbent. Their worst isolated development costs were 7.6 ms
and 13.0 ms. This cuts the medium tier from ten added calls to two while
retaining improvements such as `mpbp_15`, `mpbp_35`, `transswitch0300p`, and
`arki0016`.

All other rows receive no added work.

## Correctness and determinism

The implementation calls the existing `subset_window_descent` and
`subset_window_descent_step` routines. Those routines maintain the exact live
elimination graph, factor a window into connected components, and solve each
component by subset dynamic programming. Width never exceeds 14, and every call
has a deterministic operation allowance.

The returned candidate is re-scored with the production symbolic score
workspace. It is accepted only when its exact flop count is strictly below
`best_flops`; on acceptance, both `best_perm` and `best_flops` are updated.
Because the chain is the final stage before return, no later heuristic can turn
a local improvement into a pipeline regression.

The parent width-12/stride-5 block previously updated `best_perm` without
updating `best_flops`. This was observationally harmless while it was the final
statement. The new chain requires a current threshold, so the accepted score is
now stored there as well.

No wall clock, matrix name, corpus position, hash, external state, or source of
randomness affects the gate or ordering. The only new inputs to control flow are
`n`, `nnz`, the fixed average-degree inequality, and exact score comparisons.

## Measured score

| metric | promoted parent | reduced retry | change |
|---|---:|---:|---:|
| weighted flop score | 0.792300 | **0.792076** | **-0.000224** |
| `lt_1k` flop geomean | 0.8874 | **0.8873** | lower |
| `1k_10k` flop geomean | 0.8397 | **0.8391** | lower |
| `gt_10k` flop geomean | 0.6854 | **0.6854** | slightly lower |

Representative retained improvements include:

- `mpbp_15`: `1,197,372 -> 1,190,884` flops;
- `mpbp_35`: `977,650 -> 977,067`;
- `transswitch0300p`: `378,674 -> 378,477`;
- `arki0016`: `836,006 -> 835,812`;
- `pooling_sppa0pq`: `1,234,887 -> 1,214,182`;
- `crudeoil_pooling_ct3`: `869,821 -> 868,311`; and
- `hydroenergy2`: `55,946 -> 55,877`.

Every changed row is a strict improvement because the stage performs exact
acceptance at the end of the pipeline.

## Runtime evidence

The full direct probe on the exact retry tree reported:

| slow row | n | nnz | seconds | enters new gate? |
|---|---:|---:|---:|---|
| `crudeoil_lee4_09` | 15,904 | 101,792 | 0.798 | no |
| `crudeoil_lee4_10` | 17,809 | 120,632 | 0.783 | no |
| `nuclear104` | 39,098 | 257,806 | 0.770 | no |
| `crudeoil_lee4_06` | 10,429 | 55,492 | 0.715 | no |
| `gams05` | 17,364 | 252,910 | 0.708 | no |
| `arki0016` | 7,993 | 37,208 | 0.699 | sparse two-pass tier |

A three-repeat targeted probe measured the slowest eligible sparse-medium row,
`arki0016`, at 0.699 s. The selected two calls together cost about 21 ms at the
worst isolated development measurements. The full-chain hidden failure is the
reason no other medium pass is retained.

## Validation commands

```sh
yukon run
SSI_ALLOW_UNSANDBOXED_WORKER=1 cargo test -p ssi-candidate-worker \
  --release probe_timing_and_score -- --ignored --nocapture --test-threads=1
cargo test --release -p ssi-candidate-worker
```

The scored run passed all 300 development matrices at `0.792076`. The direct
probe reproduced that score and measured a 0.798-second worst call. The worker
suite verifies deterministic output, bijections, exact window equivalence,
score monotonicity, and work-budget behavior.

## Risk assessment

The hidden machine and matrices differ from the development environment. A
local timing margin cannot prove that no hidden row is close to two seconds.
This retry addresses the observed failure directly: dense rows receive no new
work, small rows have bounded dimension and density, and sparse medium rows pay
for two selected calls instead of ten. The expected hidden score gain is also
smaller than the first draft's because 0.92 local bip was deliberately traded
for runtime safety.

If this still reaches the hidden cap, the next fallback is the small tier alone
(`n <= 3,000 && nnz <= 80,000`), measured at roughly 1.62 development bips. If
it passes but misses the score threshold, an orthogonal terminal strict-accept
family is preferable to restoring more medium window passes.

Effort: high.
