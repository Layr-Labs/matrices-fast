# 0208 — Terminal exact-kernel extension of the promoted class

Identity of the run: model **deepseek-v4-flash**, harness **angelX** (stamped by the harness).

## Base, and how this candidate differs

The repository's leading ordering at the time of this work is the public
submission `07f0e8a2-d0fc-4d37-ab5f-ff5349768add` (commit `52affcb`, hidden
`0.842377`, promoted 2026-09-12 13:32). It is inherited exactly as the round's
own rule provides ("Each round starts from the leading ordering pushed back
into the repo"), and its own published public numbers were reproduced before
any edit: a production-mirror `order()` run over all 300 public matrices gives
**0.791865**, bucketed `0.8874 / 0.8382 / 0.6855`, matching that submission's
published exact aggregate `0.791864560331`. That agreement is the frame check
that makes the deltas below readable across two independent measuring frames.

This candidate is a **window-and-ledger extension of the promoted terminal
exact-kernel class**, with the class's own admission key left untouched:

| knob | promoted | this candidate |
|---|---|---|
| sparse-span sweep allowance | `16M + 16M + 32M` | `32M + 32M + 64M` |
| exact-window descent ledger (terminal exchange and the dense/hub window step) | `64M` | `256M` |
| PEO re-extraction rounds after the terminal search | `2` | `3` |
| admission key (unchanged) | `n in [6, 12000]`, `nnz <= 200000`, incumbent `flops <= 2e10`, **exact factor-nnz `<= 150000`** | identical |

Everything else — the producer fence above 20 billion flops, the full
portfolio, the terminal greedy stream, the promoted greedy descent, the
alternate-PEO envelope, the donor transplant — is byte-identical to the
inherited ordering, and every replacement the new allowances make is still
accepted only when the **independently recomputed exact score strictly
decreases**.

## Measured on the 300 public matrices (4-vCPU frame, one binary, one session)

| configuration | score | peak `order()` |
|---|---:|---:|
| inherited base | 0.791865 | 1.31 s |
| + full sweep allowance (`32/32/64M`) | 0.791795 | 1.29 s |
| + ledger `128M`, PEO `3` | 0.791784 | 1.31 s |
| + ledger `256M`, PEO `3`, factor key `150k` (**shipped**) | **0.791693** | 1.35 s |
| ledger `512M`, PEO `4` | 0.791654 | 1.35 s |

Shipped point: **0.791693** — a decrease of `1.72e-4` (1.72 relative basis
points) against the inherited ordering in the same frame, from **21 improved
rows (0 / 15 / 6 by size bucket), 2 slightly regressed, 277 unchanged**. Bucket
geomeans move `0.8874 / 0.8382 / 0.6855` -> `0.8874 / 0.8379 / 0.6853`. The
`512M`/4-round point buys a further `3.9e-6` for twice the allowance and is not
shipped.

Official sandboxed local run (trusted parent, sandboxed candidate build and
per-worker sandbox): **300 matrices, 0 failures, score 0.7917, fill 0.9242**
(`score.json` exact `0.791693 / 0.924171`).

## Cost, and why the admission key is not loosened

The promoted ordering's own note prices the failure mode of this class: an
earlier member of it passed all 300 public matrices and still died on the
hidden 2.0 s cap because its original input-`nnz` gate "did not limit the
filled graph on which the extra replay searches operated". The repair was the
exact factor-nonzero admission bound, and this candidate keeps that bound at
its promoted value of `150000`. The published arm table prices the loosened
bound (`300k`) at only `1.8e-5` of public score in the small-allowance arm and
`2.2e-5` in the large one; measured here, `300k` versus `150k` at the shipped
allowances is `5e-6`. A `5e-6` gain is not worth relaxing an admission bound
that the class's only hidden kill was fixed by, so the bound is unchanged and
the added work is confined to rows whose own symbolic factor is small.

The extension only widens *fixed allowances* on rows that already pass that
key: the ledgers are work allowances, so the added per-row work is bounded by
the allowance on every admitted row and is zero on every other row.

## Negative results recorded with the submission

* **The four-stream terminal round is dominated on this base.** Carried over
  from this branch's own tree, admitted by the same factor key and a
  `5e8`-per-stream budget, it moves the public score by `-5e-6` (21 -> 21
  improved rows, no new row) while raising the peak `order()` by `+0.019 s`.
  The promoted terminal kernels already collect the value that walk was buying
  on the inherited ordering, so the round is **disabled in production**
  (`FANOUT_BUDGET = 0`) and its seam is kept only for measurement.
* **No bookkeeping repair is available.** The supported-self-loss audit over
  all 300 patterns (every full-pattern score the pipeline pays is recorded and
  compared with the permutation actually returned) reports
  `leak_rows = 0`, `recoverable_bips = 0.00`: on this base the pipeline ships
  the best permutation it evaluated on every public matrix.

## Limits

The hidden corpus is not visible here. The promotion floor is one relative
basis point, the two same-class hidden steps of the inherited ordering moved
`-1.9e-4` and `-1.5e-4` against public moves of `-2.0e-4` and `-1.7e-4`, and
this candidate's public move is `-1.72e-4`; that is the arithmetic behind
expecting it to clear the floor, and it is an expectation, not a claim. The
per-row time deltas on this host are within its wall-clock noise (repeat runs
of one configuration put the peak row anywhere in `1.29`–`1.54 s`), so the cap
argument rests on the unchanged admission key and on the allowances being
fixed work budgets, not on a timing margin measured locally.
