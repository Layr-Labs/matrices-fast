# 0146 — Multiscale terminal exact-window descent

## Result

Starting from promoted source `62654a5` (submission `3085f82`, hidden score
`0.843173`), a terminal chain of exact window optimizers reduced the refreshed
300-matrix development score from **0.792300** to **0.791984**. This is a
3.16-bip local reduction. The fill tiebreak moved from **0.924373** to
**0.924245**. The full candidate-worker suite passed with **118 passed, 0
failed, 39 ignored**, and the direct timing probe measured **0.803 s** at the
slowest `order()` call on this machine.

The change is confined to `src/ordering/mod.rs`. It uses the existing exact
component-factored window dynamic program and does not add a new ordering
family, dependency, source of randomness, or input classifier.

## Context

The promoted parent already ends with four forms of local cleanup:

1. width 8, 12, and 10 component-factored window passes with two sweeps;
2. a width-12 pass with four sweeps and offset step 5;
3. small-matrix adjacent-pivot passes; and
4. exact score verification before accepting each candidate.

The window solver optimizes a contiguous section of the current elimination
order. It factors the window into connected components in the live fill graph
and solves each component exactly by subset dynamic programming. A window has
at most 14 pivots, so the exponential term is strictly bounded. Different
window widths and offset strides expose different boundaries. A local optimum
for width 12 at offsets `{0, 5, 10, ...}` need not be locally optimal for width
9 at offsets `{0, 4, 8, ...}`.

The previous session tried an adaptive terminal refinement that changed the
PEO implementation and added work over a much wider structural envelope. It
timed out on hidden matrices at broad gates, then passed the cap at a narrow
gate but regressed remotely to `0.844866`. That experiment was fully removed
before this one. The lesson applied here is to keep the promoted pipeline
intact, use the already-verified exact window primitive, put the new work at the
very end, and retain the parent's `n <= 12_000 && nnz <= 200_000` gate.

## Hypothesis

The existing terminal pass stops after one set of width-12 boundaries. A
sequence of different widths and strides should cross those boundaries and
find strict improvements that an isolated repeat cannot see. Placing the
sequence after every other production phase avoids cascade regressions: each
accepted result is the final incumbent for the next pass, and no later
conditional portfolio gate can be perturbed by it.

This is a structural hypothesis. The production decision depends only on `n`,
`nnz`, fixed width/stride constants, and the exact symbolic flop score. It does
not inspect matrix names, corpus order, hashes, or any feature intended to
identify a development instance.

## Screening

The first screen used the repository's ignored `probe_next_windows` and
`probe_window_offsets` tests on the unmodified promoted source. Each candidate
was applied independently to the finished incumbent.

| candidate | strict wins | projected score | worst added call |
|---|---:|---:|---:|
| width 12, 4 sweeps, ordinary half-width offset | 6 | 0.792256 | 0.0164 s |
| width 14, 4 sweeps, ordinary half-width offset | 17 | 0.792269 | 0.0257 s |
| width 12, 4 sweeps, stride 1 | 8 | 0.792249 | 0.0165 s |
| width 14, 4 sweeps, stride 5 | 25 | 0.792257 | 0.0255 s |

Chaining those four candidates in production reduced the official local score
to **0.792216**, an 0.84-bip gain. This established that the independently
measured gains were at least partly complementary, but the package was too thin
for a one-bip remote promotion threshold.

A second test-only grid screened widths 6 through 14 on the new `0.792216`
incumbent. The strongest independent configurations were:

| candidate | strict wins | projected score | incremental gain | worst added call |
|---|---:|---:|---:|---:|
| width 9, stride 4 | 35 | 0.792097 | 1.19 bip | 0.0076 s |
| width 11, stride 5 | 37 | 0.792122 | 0.94 bip | 0.0130 s |
| width 8, stride 3 | 29 | 0.792140 | 0.76 bip | 0.0045 s |
| width 10, stride 3 | 32 | 0.792148 | 0.68 bip | 0.0092 s |
| width 7, stride 3 | 29 | 0.792163 | 0.53 bip | 0.0039 s |
| width 6, stride 1 | 19 | 0.792180 | 0.36 bip | 0.0039 s |

The production chain appends these configurations in that order. Their gains
overlap, so the sum of the independent projections is not the measured result.
The complete sequential chain was measured from scratch with `yukon run`; its
actual score is **0.791984**.

The temporary width/stride grid was removed after screening. Only the selected
production calls remain.

## Implementation

The existing width-12/stride-5 block computed the candidate's exact score but
updated only `best_perm`, leaving `best_flops` at its previous value. That was
harmless while it was the last statement before return, but it would make a
following chain compare against a stale threshold. The block now stores the
accepted candidate's score in `best_flops` as well.

After that pass, one fixed list drives the additional calls:

```text
(width, sweeps, stride, budget, charge model)
(12, 4, 1, 64M, signature)
(12, 4, 6, 64M, union/ordinary)
(14, 4, 5, 96M, signature)
(14, 4, 7, 96M, union/ordinary)
( 9, 4, 4, 48M, signature)
(11, 4, 5, 64M, signature)
( 8, 4, 3, 32M, signature)
(10, 4, 3, 48M, signature)
( 7, 4, 3, 32M, signature)
( 6, 4, 1, 32M, signature)
```

The ordinary calls use the original union-parity work charging and their
half-width offset. The stride calls use the certified boundary-signature path.
Both paths solve the same exact window objective; the charge model changes only
how conservatively the fixed work allowance is accounted.

Every returned permutation is scored by the trusted-in-process symbolic score
workspace. The incumbent is replaced only when `flops < best_flops`, and the
stored score is updated at the same time. Consequently this terminal stage is
monotone on every input, including unseen inputs. The only competitive risk is
runtime, not a worse ordering.

## Development result

The full 300-row run produced:

| metric | promoted parent | multiscale chain | change |
|---|---:|---:|---:|
| weighted flop score | 0.792300 | **0.791984** | **-0.000316** |
| weighted fill tiebreak | 0.924373 | **0.924245** | **-0.000128** |
| `lt_1k` flop geomean | 0.8874 | **0.8873** | lower |
| `1k_10k` flop geomean | 0.8397 | **0.8390** | lower |
| `gt_10k` flop geomean | 0.6854 | **0.6852** | lower |

Examples observed in the full before/after runs include:

- `sporttournament48`: `594138 -> 592177` flops;
- `pooling_sppa0pq`: `1234887 -> 1214182`;
- `chp_shorttermplan1a`: `301837 -> 300717`;
- `crudeoil_pooling_ct3`: `869821 -> 868311`;
- `syn40m04hfsg`: `73873 -> 73562`;
- `mpbp_35`: `977650 -> 977063`; and
- `powerflow0300p`: `293009 -> 292482`.

These examples span several graph families and both medium and large score
buckets. The strict terminal acceptance prevents any counterbalancing row
regressions.

## Runtime envelope

The gate is inherited unchanged:

```text
6 <= n <= 12,000
nnz <= 200,000
```

Thus the new phase never runs on the giant rows that caused the earlier hidden
timeouts. Every call also has a deterministic operation budget. There is no
wall-clock branch, so the grader's required duplicate calls return the same
permutation.

The direct full-corpus timing probe reported a worst call of **0.803 s**. The
slowest rows were `crudeoil_lee4_09` at 0.803 s and
`crudeoil_lee4_10` at 0.795 s; both have more than 12,000 vertices and therefore
execute none of the added calls. The largest relevant measured rows remained
below those inherited worst cases. Individual screened calls added at most
about 26 ms, with the six strongest narrow passes each below 13 ms in isolation.

## Validation

Commands run from the benchmark work directory:

```sh
yukon run
SSI_ALLOW_UNSANDBOXED_WORKER=1 cargo test -p ssi-candidate-worker \
  --release probe_timing_and_score -- --ignored --nocapture --test-threads=1
cargo test --release
cargo test --release -p ssi-candidate-worker
```

Results:

- Yukon purity/build/scored run: passed all 300 matrices, score `0.791984`;
- direct timing probe: score `0.791984`, worst `order()` `0.803 s`;
- repository release tests: all non-ignored tests passed;
- candidate worker: **118 passed, 0 failed, 39 ignored**;
- determinism and bijection tests: passed as part of the worker suite.

## Caveats and next step

The development and hidden corpora are different, so the 3.16-bip local gain is
not a guarantee of a one-bip hidden gain. The evidence for generalization is the
algorithmic monotonicity, the spread across widths and graph families, and the
structural gate. Unlike a mid-pipeline win, these changes cannot alter downstream
candidate selection because they execute immediately before return.

The rejected `87e7f7b` submission found a separate 0.31-bip hidden improvement
by conditionally repeating the transplant stage, but that path can perturb
later gates and has known timeout history. It was deliberately not combined
here. If this exact-window package falls short remotely, the clean next test is
to combine it with a similarly terminal, strict-accept refinement under a
separate small/medium work ledger, rather than widen the giant-matrix gate.

Effort: high.
