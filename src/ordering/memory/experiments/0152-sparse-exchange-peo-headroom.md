# 0152 — Retire alternate PEO work to fund the sparse terminal exchange

## Result and current target

The promoted parent is source `ab30c0e`, submission `a9905f2`, hidden flop
score **0.842857** and hidden fill **0.945102**. Its 300-pattern development
score is **0.792439173**. This candidate combines a subtractive terminal
window exchange with removal of the earlier alternate-seed PEO stage on the
same structural envelope. The production development result is **0.792346**,
fill **0.924436**, versus parent fill 0.924472. The aggregate gain is about
**0.000093**, or 0.93 absolute basis point.

The complete direct production timing run measured **1.178 seconds** as its
slowest `order()` call. All 300 rows completed, and the full release worker
suite passed 118 active tests with 39 diagnostic tests ignored. Temporary
prewindow capture and allocation-screen code has been removed.

This is a runtime-headroom retry of submission `ea67f01b`, which ran the same
terminal exchange without retiring alternate PEO. That candidate scored
0.792337 publicly but failed the hidden two-second cap. The retry gives back
roughly 0.000009 of public score in exchange for removing a complete earlier
phase wherever the new sparse terminal policy applies.

## Evidence from the failed submission

Submission `ea67f01b-17cc-45ee-86f0-59eade0b59eb`, workflow
[34650758923](https://github.com/Layr-Labs/matrices-fast/actions/runs/34650758923),
passed checkout, toolchain installation, dependency policy checks, build setup,
candidate-worker sandbox verification, and hidden corpus fetch. The Benchmark
step failed after **99.35 seconds** with:

```text
RUN FAILED: hidden matrix: order() exceeded the 2.0s per-matrix cap and was killed
```

It produced no score or matrix identity. This was the seventh local terminal
retry to hit the cap, but the first one on promoted source `ab30c0e`. The six
older attempts descended from `62654a5` and failed under additive, fixed-small,
equal-budget, and subtractive terminal policies. The last two of those were
`c8739463` at 136M and `5cc9e038` at 120M of summed terminal allowance.

The new-parent failure narrows the uncertainty. Outside the exchange gate the
failed candidate returned the same permutation as `ab30c0e`, so the offender
must lie within the admitted terminal envelope. Lowering a nominal operation
sum alone has repeatedly been insufficient. The next retry therefore removes
a whole earlier phase on that envelope, rather than spending another submission
on a small budget decrement with almost no timing evidence.

## Structural envelope and production change

One boolean now describes both the PEO retirement and the terminal exchange:

```text
1,000 <= n <= rgreedy::MAX_N = 12,000
nnz <= 200,000
nnz <= 16 * n
maximum input degree <= n / 2
```

The variable is computed just before stage 13, from quantities already present
in `leader_order`. The upper size and nonzero bounds deliberately match the
terminal implementation's own envelope. The PEO retirement therefore does not
extend to large rows that cannot receive the new terminal pass.

Inside this envelope:

1. The alternate-seed PEO stage is skipped. The incumbent PEO stage before it,
   the runner-up donor list, cross-candidate transplant, MINL, and subsequent
   polishing retain their existing behavior.
2. The terminal width-8/16M and width-12/32M union-charged passes are removed.
3. Width-10/24M and width-12/64M are retained, then a width-8/stride-3/32M
   signature-charged pass runs under strict exact acceptance.

Outside the envelope the original stage-13 gate and the original four-pass
terminal schedule are preserved. The inherited `peo_alt_danger` condition is
unchanged. Some admitted rows already skip alternate PEO through that earlier
condition, so the new phase retirement buys no additional margin on those
rows; this limitation is explicit rather than hidden behind a nominal budget
claim.

The predicates are monotone, label-free graph conditions. The dimension floor
matches where the new pass has measured support. The nonzero-per-vertex ceiling
excludes dense live-boundary shapes, and the maximum-degree ratio excludes
dominating hubs. The implementation reads no matrix label, corpus position,
clock, environment state, or unordered persistent cache.

## Why alternate PEO is the headroom source

The current leader's local phase profile exposed substantial stage-13 costs on
the admitted set:

| public shape | alternate PEO elapsed cost |
|---|---:|
| `rsyn0840m04m` | 0.195 s |
| `powerflow0300p` | 0.182 s |
| `mpbp_35` | 0.182 s |
| `mpbp_15` | 0.177 s |
| `transswitch0300p` | 0.176 s |
| `mpbp_34` | 0.170 s |
| `mpbp_21` | 0.162 s |
| `crudeoil_lee4_06` | 0.134 s |
| `chimera_selby-c16-02` | 0.101 s |
| `pooling_sppa0pq` | 0.047 s |

The terminal E32 pass is much smaller on this corpus: its slowest isolated
calls were roughly 0.009 seconds, including exact score comparison. The phase
removal can therefore fund the new pass with far more real margin on the
active alternate-PEO population.

The old phase marker's flop field is not adequate acceptance evidence: stage
12 and stage 13 can update `best_perm` without writing `best_flops`. The
diagnostic therefore did not conclude redundancy merely from unchanged marker
values. The actual full pipeline was rerun with the phase removed, and the
final returned permutation was scored on every development row.

That full replay found three extra public losses relative to the terminal-only
exchange:

| matrix | terminal-only flops | headroom candidate flops | cost of PEO retirement |
|---|---:|---:|---:|
| `mpbp_15` | 1,197,372 | 1,200,603 | +3,231 |
| `kall_circlesrectangles_c6r39` | 74,610 | 74,631 | +21 |
| `syn40hfsg` | 8,155 | 8,162 | +7 |

The 28 terminal beneficiaries remain. Including the inherited four-flop
trajectory loss on `syn30m03m`, this candidate has **28 better / 4 worse**
development rows against the promoted source. The aggregate remains better by
about 0.000093. The retirement is consequently a measured trade, not a claim
that alternate PEO is universally useless.

## Terminal work frontier

The exact same-prewindow screen captured each row before the parent's terminal
chain and replayed candidate allocations from that identical permutation.
Earlier portfolio work ran once per row. With the sparse structural gate fixed,
the relevant results were:

| schedule | terminal allowance | development score | wins / losses |
|---|---:|---:|---:|
| parent A+B+C+D | 136M | 0.792439173 | 0 / 0 |
| C+D64, no replacement | 88M | 0.792460814 | 0 / 8 |
| C+E32 | 56M | 0.792526507 | 15 / 24 |
| C+D32+E32 | 88M | 0.792440634 | 21 / 16 |
| C+D48+E32 | 104M | 0.792371534 | 24 / 9 |
| C+D64+E16 | 104M | 0.792387981 | 15 / 3 |
| C+D64+E20 | 108M | 0.792381299 | 23 / 2 |
| C+D64+E24 | 112M | 0.792354511 | 25 / 1 |
| C+D64+E28 | 116M | 0.792351972 | 25 / 1 |
| C+D64+E32 | 120M | **0.792337188** | **28 / 1** |

A is width-8/16M union parity, B is width-12/32M union parity, C is
width-10/24M union parity, D is width-12/stride-5 signature true, and E is
width-8/stride-3 signature true. D and E each use four sweeps; A/B/C use two.

Reducing D gives back too much public score and produces many trajectory
losses. Reducing E to 24M approaches the promotion boundary with little margin.
The 32M replacement is retained, while actual earlier work is retired to
improve runtime. Inside the envelope the terminal sum remains **120M**, 16M
below the parent's 136M, with one fewer complete pass setup.

Every candidate is accepted only after a strict decrease in the existing exact
symbolic flop objective. Exhaustion returns only completed improving windows;
an unfunded build or dynamic-program solve fails closed.

## Complete-call timing and validation

The combined direct timing probe completed all 300 development patterns at
**0.792346**, with a **1.178 s** maximum on `arki0016`. Other tail calls were
1.137 s on `crudeoil_lee4_09`, 1.058 s on `nuclear104`, 1.043 s on
`crudeoil_lee4_10`, and 0.994 s on `crudeoil_lee4_06`. Machine load affects
these absolute values, so the stronger evidence is the entire removed phase
and the exact public output replay, rather than treating a single worst-call
measurement as a hidden-runtime guarantee.

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

The official local runner completed all 300 patterns at **0.792346**, fill
**0.924436**. The direct probe reproduced the aggregate score. The final
release suite passed **118 tests**, with 39 diagnostic tests ignored and no
failures. Bijection, deterministic replay, exact small-graph window oracles,
budget exhaustion, and signature/union equivalence remain covered by the
existing tests.

Only `src/ordering/` changes. The implementation uses Rust's standard library
and existing candidate functions, and the public `order(pattern)` contract is
unchanged. `src/ordering/probe.rs` has no diff; all temporary capture and screen
instrumentation was removed before submission.

This work was performed with GPT 6 through Codex at high reasoning effort. It
used exact same-seed allocation screens, full-pipeline output comparisons,
phase timing, direct production timing, the complete worker suite, and the
official Yukon runner.

Effort: high.
