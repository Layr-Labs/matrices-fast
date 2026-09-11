# 0150 — Equal-budget terminal window exchange

## Outcome and submission target

This candidate scores **0.792188201** with a **0.924327** fill tiebreak on all
300 development matrices. The current hidden leader remains submission
`3085f82`, score **0.843173**, from promoted source `62654a5`. Relative to that
source's locally reproduced score of 0.792299626, the candidate gains
**0.000111425**, or 1.114 absolute basis points. It changes 33 development rows:
32 improve and one regresses by 30 symbolic flops.

The change is a work exchange inside the promoted terminal budget. It removes
the inherited width-12, two-sweep, 32M union-charged window pass and adds a
width-8, four-sweep, stride-3, 32M signature-charged pass at the end. All other
terminal passes remain in their promoted order. The sum of their deterministic
allowances stays **136M**, exactly equal to source `62654a5`; no size class gets
an additive terminal allowance.

A direct production probe completed all 300 rows at the same 0.792188 score and
measured **0.727 seconds** as the worst complete `order()` call. The isolated
terminal chain's maximum was 0.02273 seconds, below the promoted terminal
chain's 0.02788 seconds in the same screen. This candidate is intended to keep
the public improvement while returning to the amount of terminal work already
accepted by the hidden runner.

## Evidence from four hidden timeouts

Four additive descendants of the promoted parent failed the same hidden
two-second cap:

| experiment | submission | workflow | development score | hidden result |
|---|---|---:|---:|---|
| 0146 | `42491bf1` | `34442427563` | 0.791984 | per-matrix timeout |
| 0147 | `a1f928e2` | `34444803770` | 0.792076 | per-matrix timeout |
| 0148 | `113841b9` | `34511625092` | 0.792241 | per-matrix timeout |
| 0149 | `3a4f3ed0` | `34514415941` | 0.792241 | per-matrix timeout |

The sequence ruled out several initial explanations. Experiment 0147 removed
most medium-row work; 0148 removed every new terminal pass above 3,000 vertices
and reduced each surviving allowance to 8M. Experiment 0149 then preserved the
0148 score while removing 80–90 ms from inherited alternate-PEO work on large,
dense public analogues. The hidden timeout remained.

Source `62654a5` itself passed hidden validation. The common difference in all
four failures is extra terminal work on at least some eligible rows. The 0149
failure makes an unrelated large-row bottleneck less plausible because its new
guard did not move the failure. The robust next step is therefore to stop
adding terminal work and buy any new pass by deleting an inherited one.

## Hidden result

Submission `c8739463` failed the hidden Benchmark step in workflow
`34644516850`. The grader started `order()` evaluation at 20:33:32.527 UTC and
reported the two-second per-matrix kill at 20:35:11.727 UTC, **99.20 seconds**
later. This is effectively the same corpus position as the preceding failures.

The result falsifies the note's strongest safety claim. Equal summed operation
allowances do not imply equal wall time when the pass shape and charge model
change. The removed width-12 pass used union-parity charging; the replacement
uses honest signature charging, visits smaller components at more offsets, and
can fund a different number of windows on a hidden halo shape. Its development
maximum was lower, but that did not constrain every hidden graph.

The follow-up [0151](0151-subtractive-terminal-window-exchange.md) removes the
inherited width-8/16M pass as well. It preserves almost all of this experiment's
score, reduces the terminal schedule from four passes to three, and lowers its
deterministic allowance to 120M, 16M below the promoted parent.

## Promoted budget and replacement

The promoted parent runs these terminal exact-window passes when
`6 <= n <= MAX_N` and `nnz <= 200,000`:

| order | window | sweeps | stride | charge model | allowance |
|---:|---:|---:|---:|---|---:|
| 1 | 8 | 2 | 4 | union parity | 16M |
| 2 | 12 | 2 | 6 | union parity | 32M |
| 3 | 10 | 2 | 5 | union parity | 24M |
| 4 | 12 | 4 | 5 | signature true | 64M |
|  |  |  |  | **total** | **136M** |

The candidate runs:

| order | window | sweeps | stride | charge model | allowance |
|---:|---:|---:|---:|---|---:|
| 1 | 8 | 2 | 4 | union parity | 16M |
| 2 | 10 | 2 | 5 | union parity | 24M |
| 3 | 12 | 4 | 5 | signature true | 64M |
| 4 | 8 | 4 | 3 | signature true | 32M |
|  |  |  |  | **total** | **136M** |

The implementation's allowance is a deterministic work counter, not a timer.
Each pass first charges graph construction and then refuses work it cannot
fund. Completed strict gains survive later exhaustion. Consequently the table
is an upper envelope: a row may spend less, but cannot acquire a larger nominal
terminal allowance than the promoted source.

The width reduction also improves the cost shape beneath that equal envelope.
Exact component search allocates and traverses a subset dynamic program with
up to `2^k` states for a connected window component. Reducing the replaced
window from width 12 to width 8 lowers the largest state count from 4,096 to
256. Four sweeps and stride three inspect more overlapping positions, which is
where the score gain comes from, but each exact component is substantially
smaller. The direct isolated measurements confirm that this trade is faster on
the development corpus rather than relying only on asymptotic reasoning.

## Exact exchange screen

I captured the incumbent immediately before the terminal window chain and ran
every replacement from that identical seed. This avoids attributing a score
change to a different earlier candidate trajectory. The pre-window incumbent
scored 0.792671611, and the promoted terminal sequence scored 0.792299626.

The most useful screened exchanges were:

| replacement for inherited width-12/32M pass | score | isolated worst |
|---|---:|---:|
| promoted sequence, no replacement | 0.792299626 | 0.02788 s |
| width 7, stride 3, 32M | 0.792211545 | below promoted maximum |
| **width 8, stride 3, 32M** | **0.792188201** | **0.02273 s** |
| width 9, stride 4, 32M | 0.792219604 | below promoted maximum |
| width 8/8M followed by width 9/24M | 0.792208074 | below promoted maximum |

The selected exchange gives 58 strict wins relative to the shared pre-window
incumbent in that diagnostic. It is also better than the earlier additive
low-budget chain, which scored 0.792241346 and had a 0.03190-second isolated
maximum. That distinction matters: the best score came from reallocating work,
not simply from choosing the smallest measured runtime.

All temporary capture switches and probe instrumentation were removed. The
production source contains only the exchange and the previously committed
dense alternate-PEO guard.

## Per-row score movement

Against promoted source `62654a5`, the selected exchange changes 33 rows. The
largest exact improvements are:

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

The remaining improvements range from 349 flops down to four flops and span
pooling, crude-oil, chimera, synthetic, power-flow, hydroenergy, torsion, and
other matrix families. This is broader than a single-row ticket. The sole
regression is `chimera_mgw-c8-439-onc8-002`, from 86,464 to 86,494 flops. It
occurs because removing the old pass changes the seed seen by later passes;
strict acceptance is monotone within the new chain, but two different chains
need not be pointwise ordered. The weighted aggregate and fill tiebreak both
improve exactly.

## Correctness and determinism

Every terminal candidate is produced from the supplied pattern and current
permutation with fixed widths, strides, sweep counts, and integer work limits.
There is no clock, randomness, matrix name, corpus index, environment input, or
hash iteration order in the decision. The returned result is deterministic for
a given pattern.

The window routine preserves all vertices outside the active window and emits
the exact vertices inside it once each. The outer selector scores every
candidate with the benchmark's exact symbolic objective and accepts it only on
a strict improvement over the current incumbent. If a work allowance cannot
cover construction or the next component, the routine returns no candidate or
keeps only an already completed strict gain. The final worker tests cover
bijection, determinism, certificates, score workspace behavior, and exhaustive
small graph cases.

The dense alternate-PEO guard from 0149 remains. It is disjoint from the
terminal exchange's score argument and changed no development flop count. Its
public runtime reduction still provides useful headroom when a large dense row
matches that shape, while the equal-budget exchange directly addresses the
terminal-eligible class implicated by the fourth timeout.

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

`yukon run` completed 300 of 300 matrices at **0.792188**, with a
**0.924327** fill tiebreak. The direct timing-and-score probe reproduced the
same aggregate score and reported **0.727 seconds** as the worst complete
`order()` call. The release worker suite passed **118 tests**, with 39 explicit
measurement probes ignored and no failures. The source diff check is clean,
and `src/ordering/probe.rs` has no production diff.

The recent-submission frontier was refreshed after validation. The best hidden
score is still 0.843173 from `3085f82` / source `62654a5`. A more recent scored
submission at 0.843142 was rejected because its improvement was only 0.000031;
this candidate's development delta is more than three times that magnitude and
exceeds one absolute basis point. Hidden translation is corpus-dependent, but
the package clears the observed public promotion scale and distributes its
wins across many rows.

This work was performed with GPT 6 through Codex at high reasoning effort. It
used exact same-seed terminal screens, per-row symbolic comparisons, direct
production timing, and the full release worker suite.

Effort: high.
