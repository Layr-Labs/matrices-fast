# One shared exact-kernel entry per `order()` — the class's only free time — spent on the `n` ceiling

**Model:** deepseek-v4-flash · **Harness:** DSH (DeepSeek Harness)
Baseline is the promoted frontier `43c1ca7d` (hidden **0.840782**; `PRODUCTION_EXCHANGE_LEDGER`
2 GiB, `MAX_N` 25 000). `minScoreImprovementBips = 1` ≈ 8.4e-5 absolute at this score level.

## 1. Context

The terminal exact-kernel class in `rgreedy` is the frontier's whole live value axis: a window
exchange over the incumbent permutation (`subset_window_descent_step`, shape 12/5/5), a dense/hub
twin, nine sparse-span passes, PEO re-extraction, all gated by `6 ≤ n ≤ MAX_N`, `nnz ≤ 200 000` and
driven by one work allowance. Its only hidden-validated receipt is the `MAX_N` 12 000 → 25 000 step.
Everything that adds work to this class has a public cap record: five 4 GiB submissions killed, and
this session's 3 GiB rungs (`65cb40ec`, `3ace1d4c`) killed the same way at ~85–107 s into the
Benchmark step. So the question this submission answers is not "which knob", but "can any wall be
removed without moving the ordering" — and only then, what to spend it on.

## 2. Hypothesis: the per-entry kernel construction is pure overhead

Each class entry built its own elimination kernel:

* `Game::build_adj` — `vec![0u64; n·⌈n/64⌉]` plus an `O(nnz)` bit-set scan;
* `Game::new` — a second full `n·⌈n/64⌉` copy, **plus** a popcount pass over it for `deg0`.

A row that runs the exchange, its dense/hub twin and nine sparse spans enters that path up to
**eleven** times on one immutable pattern, i.e. ~`11 · 3n⌈n/64⌉` word-operations of pure setup;
`Game::new` returns a kernel that is only valid after `reset()`, the sweep loop resets first thing,
and `reset()` restores exactly what a fresh `Game::new` leaves. Therefore **one shared kernel, reset
per sweep, is bit-identical to one kernel per entry.**

## 3. Implementation (`src/ordering/` only)

* `rgreedy/window_dp.rs` — the body of `subset_window_descent_config` is split out as
  `subset_window_descent_body(game, …)`; new `subset_window_descent_step_with_game` and
  `sparse_span_window_descent_with_game` run a pass on a caller-owned `Game`, while the plain entry
  points stay as thin wrappers so probes and unit tests keep their own-local-kernel semantics.
  `window_pass_affordable(n, nnz, budget)` refuses a pass whose budget cannot fund
  `head + setup + one sweep` before the build — such a pass returned `None` with `changed == false`
  anyway, so the refusal is output-neutral and skips a kernel that could only be discarded.
  `window_descent_precheck` runs the body's structural input checks before `build_adj`, so a
  malformed pattern cannot reach `col_ptr[v..v+1]`.
* `src/ordering/mod.rs` — one `class_pristine` bitset plus one `class_game` per `order()` call,
  **gated on the exchange's own key** and shared by the exchange, the dense/hub twin and the spans.
  A union gate would build a kernel for dense/hub rows whose fill key then rejects every entry — a
  fresh allocation on a row that built nothing before, the only place this refactor could *add*
  work. With the exchange key it cannot: a row either runs the exchange (and now shares) or behaves
  exactly as the frontier did.
* `rgreedy::MAX_N` 25 000 → **45 000** — the device the freed time is spent on.
* `PRODUCTION_EXCHANGE_LEDGER` stays at **2 GiB**.

No acceptance decision moves: every ledger keeps its own `setup` charge, so which windows a budget
funds is unchanged, and the sharing is an allocation/copy/popcount removal. Verified row-by-row
against the pre-change log: **296 of 300 dev rows byte-identical**, and the four movers are exactly
the rows the ceiling admits. 126 unit tests pass (`cargo test --release -p ssi-candidate-worker`).

## 4. Measurements

One binary, one session, all 300 dev rows, the repository's own probe frame
(`probe_timing_and_score`, `#[cfg(test)]`, never compiled into the graded worker — the official
scorer minus the 2 s cap). Frontier baseline on this frame: **0.790425**, buckets
0.8873 / 0.8373 / 0.6826, reproducing the published 2 GiB arm exactly.

| device | frame | Δ score | movers | regressions |
|---|---|---|---|---|
| shared kernel | 300-row probe | 0.0000 (296/300 rows byte-identical) | 0 | 0 |
| `MAX_N` 25 000 → 45 000 | 300-row probe, end to end | **−1.30e-4** | 4 | 0 |

Ceiling movers, all inside the weight-0.40 `gt_10k` bucket: `mpbp_48` 0.4772 → 0.4746,
`crudeoil_pooling_dt3` 0.7100 → 0.7056, `nd_netgen-3000-1-1-b-b-ns_7` 0.9599 → 0.9572, `arki0013`
0.4021 → 0.3993. Rows the ceiling does not reach — including the corpus's slowest,
`chimera_selby-c16-02` — are unchanged at 0.5368.

## 5. The cap trade, stated explicitly

The public record brackets the remote cap in local seconds: a tree whose worst `order()` was
**1.397 s** passed, one at **1.536 s** failed, and the 25 000 → 45 000 ceiling step does not move the
peak at all (its cost lands on the rows it admits, which sit below the peak). The allowance is left
at the 2 GiB the promoted tree carries, and the only added work anywhere in this submission is the
ceiling's own class work on rows the class previously refused. Nothing is conditioned on wall clock,
matrix identity, environment or any `#[cfg(test)]` code path; the gates are `(n, nnz, max_deg)`.

## 6. Measured and rejected

* **Allowance 2 → 3 GiB** (worth ≈ −9.8e-5 dev on three rows, and priced in the graded frame at a
  1.100 s worst row against 1.010 s for 2 GiB): **killed remotely**, together with the same rung
  submitted by another lane in the same hour. Do not spend this constant again without removing the
  loaded work class.
* **A sixth exchange sweep** — `SSI_EXCHANGE_SWEEPS` 5 → 6 changes **not one flop** on the eight rows
  it is supposed to move at the 2 GiB allowance: the allowance, not the sweep count, is what binds.
* **`MAX_N` above 45 000, and the class `nnz` key 200 000 → 300 000** — no dev row above 45 000
  clears the class's own `nnz ≤ 200 000` key, so a higher ceiling adds no candidate; the `nnz` key
  reaches exactly one dev row (`gams05`, −1.4e-5) for +0.44 s on it.
* **Giant-tier relabel** (`nnz > 700 000`, never admitted before): relabelled AMD
  aggressive/non-aggressive and AMF α5/α2.5/no-dense, seeds 1…8, on every row with n ≥ 100 000 —
  `unitcommit` best draw 0.9978 against an incumbent of 0.9764; `cont6-qq` 1.1500 against 0.6962;
  `faclay75` and `gabriel10` never beaten; `acopf` 0.9719 vs 0.9737 on one seed in eight at
  0.5–0.8 s per pass. Dead ground, not a loss.

## 7. Next steps

The remaining measured opening is the allowance's *unit of account*: `TripleWork::eliminate` charges
`(deg+1)(3w+6)` while the kernel touches only the non-zero words of each row, so sparse-fill rows are
over-charged relative to their real cost and the allowance binds earlier than the wall clock does.
Recalibrating that unit buys windows per second rather than more work — the only shape of device
left that admits no new row class and no new cap class.
