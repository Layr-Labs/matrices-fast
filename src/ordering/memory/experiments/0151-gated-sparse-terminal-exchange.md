# 0151 — Gate the subtractive terminal exchange by sparse graph structure

## Outcome and submission target

This candidate starts from promoted source `ab30c0e`, submission `a9905f2`,
whose hidden flop score is **0.842857** with fill **0.945102**. The source
reproduces at **0.792439173** on the 300-pattern development corpus. The
candidate scores **0.792337188** in the exact schedule screen and **0.792337**
in `yukon run`, with fill **0.924431**. The weighted flop delta is
**-0.000101985**, or 1.01985 absolute basis points. Lower is better.

The change exchanges terminal search work only when all three monotone,
label-free structural predicates hold:

```text
n >= 1,000
nnz <= 16 * n
maximum input degree <= n / 2
```

On those rows it removes the inherited width-8/16M and width-12/32M
union-charged passes, retains width-10/24M and width-12/64M, then adds a
width-8/stride-3/32M signature-charged pass. The resulting allowance is 120M,
16M below the promoted terminal schedule, and it contains one fewer pass. All
other rows retain the promoted four-pass terminal schedule.

The same-prewindow screen reports **28 improvements and one tiny trajectory
loss** against the promoted source. The direct production timing probe
completed all 300 rows and measured **1.308 seconds** as the slowest complete
`order()` call. The release worker suite passed 118 active tests.

## Why this starts from a different parent

The preceding terminal-search work was developed on promoted source `62654a5`,
hidden score 0.843173. Six descendants of that source failed the hidden
two-second per-matrix cap:

| submission | development result | attempted runtime control | hidden result |
|---|---:|---|---|
| `42491bf1` | 0.791984 | additive multiscale window chain | timeout |
| `a1f928e2` | 0.792076 | two-tier size and sparsity envelope | timeout |
| `113841b9` | 0.792241 | fixed low-width budgets | timeout |
| `3a4f3ed0` | 0.792241 | low-width budgets plus dense PEO headroom | timeout |
| `c8739463` | 0.792188 | equal-budget width exchange, 136M | timeout |
| `5cc9e038` | 0.792191 | subtractive width exchange, 120M | timeout |

The last two are especially useful negative evidence. Merely matching or
lowering the nominal terminal allowance did not remove the offender on that
parent. The hidden grader identifies neither the matrix nor the elapsed time,
so repeated size-window tuning could not establish a causal boundary.

While those runs were in flight, submission `a9905f2` promoted source
`ab30c0e`. It removes three narrow stage-1b independent-set adoption windows
and retains forced adoption only for `n >= 20,000`. Its public score is worse
than `62654a5`, but its hidden score is better by 0.000316. That result is direct
evidence that the changed early trajectory generalizes better. This experiment
therefore rebases onto `ab30c0e` and treats all six failures as evidence about
the old trajectory, not proof that every terminal width-8 exchange must fail
on the new one.

## Structural gate

The gate was selected from input structure and from breadth screens, without
matrix names, corpus positions, clocks, environment state, or randomized
decisions.

The `n >= 1,000` floor follows the measured support of the new signature pass:
the public screen found no aggregate value below 1,000 vertices. It also avoids
changing the many tiny cases whose complete production call is dominated by
fixed portfolio setup rather than terminal work.

The `nnz <= 16*n` ceiling admits sparse graphs where an eight-pivot window has
a compact live boundary. It retains a broad mixture of 105 public rows after
the degree test, including pooling, crude-oil, power-flow, edge-crossing,
network, synthetic, and process models. It excludes dense KKT-like graphs
where boundary extraction and exact state construction can dominate even when
the pass's internal dynamic-program charge is bounded.

The `maximum_degree <= n/2` condition excludes near-stars and other dominating
hub shapes. Such graphs can expose a broad live halo to many consecutive
windows. The condition is scale-free and is computed from the input adjacency
already built by `order()`. Representative extreme-hub public rows such as
`chimera_mgw-c16-2031-01`, `chimera_lga-01`, and `torsion50` remain exactly on
the promoted terminal schedule under this rule.

These predicates describe the cost model of the algorithm. They do not encode
particular corpus rows. The gate is also conservative in work: every admitted
row receives 120M rather than 136M of summed terminal allowance, and every
non-admitted row receives the already hidden-proven promoted schedule.

## Exact schedules

The promoted terminal sequence is:

| order | window | sweeps | stride | charge model | allowance |
|---:|---:|---:|---:|---|---:|
| A | 8 | 2 | 4 | union parity | 16M |
| B | 12 | 2 | 6 | union parity | 32M |
| C | 10 | 2 | 5 | union parity | 24M |
| D | 12 | 4 | 5 | signature true | 64M |
|  |  |  |  | **total** | **136M** |

The candidate uses the following sequence only inside the structural gate:

| order | window | sweeps | stride | charge model | allowance |
|---:|---:|---:|---:|---|---:|
| C | 10 | 2 | 5 | union parity | 24M |
| D | 12 | 4 | 5 | signature true | 64M |
| E | 8 | 4 | 3 | signature true | 32M |
|  |  |  |  | **total** | **120M** |

Each completed candidate is scored by the existing exact symbolic objective
and accepted only on a strict decrease. An exhausted pass fails closed. The
allowances bound charged algorithmic work; they are not elapsed-time budgets.
Removing A and B is therefore useful twice: the sum falls by 16M after adding
E, and one complete pass setup disappears.

## Same-prewindow schedule screen

The diagnostic captured the incumbent immediately before the terminal chain
and replayed each schedule from that exact permutation. Earlier portfolio work
ran once per row, so changes in early-stage scheduling did not contaminate the
comparison.

The strongest structural gate in the screen was the selected combination:

| gate for adding E32 after the parent | score | wins | paid public rows |
|---|---:|---:|---:|
| all terminal-eligible rows | 0.792329292 | 33 | broad |
| `n>=1k`, `nnz<=16n` | 0.792330469 | 32 | 110 |
| `n>=1k`, `nnz<=16n`, `maxdeg<=n/2` | **0.792334478** | **28** | **105** |
| same, `maxdeg<=n/4` | 0.792334478 | 28 | 100 |
| same, `maxdeg<=n/8` | 0.792345212 | fewer | fewer |

The `n/2` degree ceiling removes the most exposed hub shapes without giving up
any measured aggregate gain relative to the density-only gate. Tightening to
`n/4` removes five more rows but brings no score evidence; tightening to `n/8`
starts discarding useful rows. The least restrictive tied rule is used.

With that gate fixed, the exact schedule allocation screen was:

| admitted-row schedule | allowance | aggregate score | wins / losses vs parent |
|---|---:|---:|---:|
| promoted `A+B+C+D` | 136M | 0.792439173 | 0 / 0 |
| additive `A+B+C+D+E32` | 168M | 0.792334478 | 28 / 0 |
| equal `A+C+D+E32` | 136M | 0.792334751 | 28 / 0 |
| **selected `C+D+E32`** | **120M** | **0.792337188** | **28 / 1** |
| `A+C+D+E24` | 128M | 0.792352074 | fewer | small losses |
| `C+D+E24` | 112M | 0.792354511 | fewer | small losses |

Removing B costs only 0.000000273 relative to the additive schedule and brings
the allowance back to the parent's level. Removing A costs another 0.000002437
while buying the full 16M margin below the parent. Reducing E from 32M to 24M
loses substantially more score. The selected schedule is the clearest measured
point above one absolute public basis point with less terminal work than the
promoted source.

## Public behavior and timing

Representative final candidate counts include:

| matrix | candidate flops | purpose in the check |
|---|---:|---|
| `pooling_sppa0pq` | 1,217,666 | sparse broad-boundary beneficiary retained |
| `crudeoil_lee2_06` | 17,659,239 | medium sparse beneficiary retained |
| `edgecross10-030` | 100,139 | low-degree sparse beneficiary retained |
| `transswitch0300p` | 378,205 | larger sparse beneficiary retained |
| `mpbp_34` | 893,123 | existing promoted trajectory preserved |
| `chimera_mgw-c16-2031-01` | 2,600,520 | dominating hub excluded |
| `chimera_lga-01` | 497,542 | dominating hub excluded |
| `torsion50` | 1,119,177 | dominating hub excluded |

An additive diagnostic timed E32 itself at roughly ten milliseconds or less on
its slowest public beneficiaries: 0.00991 s on `powerflow0300p`, 0.00988 s on
`pooling_sppa0pq`, 0.00895 s on `crudeoil_lee1_07`, and 0.00853 s on
`popdynm25`. These figures explain the public cost but do not substitute for
the full production timing probe.

The production probe ran the submitted `order()` on all 300 matrices. Its
slowest complete call was **1.308 s** on `crudeoil_lee4_09`; that dense row is
outside the exchange gate. The next slowest calls were 1.191 s on `arki0016`,
1.184 s on `crudeoil_lee4_10`, and 1.153 s on `crudeoil_lee4_06`. All completed
below the two-second cap on this machine. The probe reproduced the official
aggregate **0.792337** score.

## Correctness, determinism, and scope

The implementation changes only `src/ordering/mod.rs` plus these records under
`src/ordering/memory/`. It uses the Rust standard library and existing ordering
functions. The public `order(pattern)` signature is unchanged. Temporary probe
instrumentation was removed; `src/ordering/probe.rs` has no diff.

The decision depends solely on `n`, symmetric input nonzeros, and maximum input
degree. Integer saturation protects `16*n`. Every returned candidate goes
through the existing exact score comparison. Window refinement preserves all
vertices outside its window and returns every inside vertex once. Existing
tests cover bijection, determinism, budget boundaries, malformed inputs,
signature/union equivalence, and exact small-graph window oracles.

For rows outside the structural gate, the generated candidates and their order
are unchanged from `ab30c0e`. The added `best_flops` assignment after D only
records the score of an accepted final incumbent so E can compare against it;
where E is disabled, no later decision reads that value and the returned
permutation is unchanged.

## Validation

The exact production tree was checked with:

```sh
yukon run
SSI_ALLOW_UNSANDBOXED_WORKER=1 cargo test --release \
  -p ssi-candidate-worker probe_timing_and_score \
  -- --ignored --nocapture --test-threads=1
SSI_ALLOW_UNSANDBOXED_WORKER=1 cargo test --release \
  -p ssi-candidate-worker
git diff --check -- src/ordering
```

`yukon run` completed 300 of 300 matrices at **0.792337**, fill **0.924431**.
The timing probe reproduced the score and reported a **1.308 s** maximum. The
release suite passed **118 tests**, with 39 diagnostic tests ignored and no
failures. The scope and whitespace checks are clean.

The candidate combines a broad 28-row public signal with an aggregate delta
slightly above one absolute basis point. Its hidden outcome remains empirical,
but the runtime policy is now tied to the live-boundary cost of the pass, and
the work exchange is strictly subtractive wherever it runs.

This work was performed with GPT 6 through Codex at high reasoning effort. It
used exact same-seed schedule screens, deterministic work accounting, direct
production timing, the complete release worker suite, and the official Yukon
runner.

Effort: high.
