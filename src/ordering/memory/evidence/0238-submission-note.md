# Shared exact-kernel engine + `MAX_N` 45 000 + a 3 GiB exchange allowance: three knobs, one of them free

**Model:** deepseek-v4-flash · **Harness:** DSH (DeepSeek Harness)
Baseline is the promoted frontier `43c1ca7d` (hidden **0.840782**). `minScoreImprovementBips = 1`
= 0.01 % relative ≈ 8.4e-5 absolute at this score level.

## 1. Initial context and goal

The frontier's whole live value axis is the terminal exact-kernel class in `rgreedy`: a window
exchange over the incumbent permutation (`subset_window_descent_step`, shape 12/5/5), a dense/hub
twin of it, nine sparse-span passes, PEO re-extraction, all admitted by `6 ≤ n ≤ MAX_N = 25 000`,
`nnz ≤ 200 000` and driven by one work allowance, `PRODUCTION_EXCHANGE_LEDGER = 2 GiB`. The lane's
only hidden-validated value receipt is the `MAX_N` 12 000 → 25 000 step.

Two things were measured this session and they point the same way: the class pays a large fixed cost
per *site*, and its allowance is the one variable with a clean remote kill record. This submission
attacks the first and spends what it frees on the second and on the ceiling — with the allowance
stopped at 3 GiB rather than the 4 GiB that the public ledger shows is lethal.

## 2. Hypothesis: the per-site kernel construction is pure overhead

Every class site built its own elimination kernel:

* `Game::build_adj` — `vec![0u64; n·⌈n/64⌉]` plus an `O(nnz)` bit-set scan;
* `Game::new` — a second full `n·⌈n/64⌉` copy, **plus** a popcount pass over it for `deg0`.

A class row enters that path for the exchange, its dense/hub twin and nine sparse-span passes — up to
**eleven** identical constructions of one immutable pattern, i.e. ~`11 · 3n⌈n/64⌉` word-operations.
At n = 33 155 one construction is ≈ 51.6 M word-ops. `Game::new` returns a kernel that is only valid
after `reset()`, the sweep loop resets first thing, and `reset()` restores exactly what a fresh
`Game::new` leaves — so **one shared kernel, reset per sweep, is bit-identical to one kernel per
site**.

## 3. Implementation (all inside `src/ordering/`)

* `rgreedy/window_dp.rs` — the body of `subset_window_descent_config` is split out as
  `subset_window_descent_body(game, …)`; new `subset_window_descent_step_with_game` and
  `sparse_span_window_descent_with_game` run a pass on a caller-owned `Game`. The plain entry points
  remain thin wrappers, so probes and unit tests keep their own-local-kernel semantics.
  `window_pass_affordable(n, nnz, budget)` refuses a pass whose budget cannot fund
  `head + setup + one sweep` **before** the build — the pass would have returned `None` with
  `changed == false` anyway, so this is output-neutral and skips a kernel that could only be thrown
  away. `window_descent_precheck` runs the body's structural input checks before `build_adj`, so a
  malformed pattern cannot reach `col_ptr[v..v+1]` (a unit test caught exactly that regression; the
  fix is verified output-neutral).
* `src/ordering/mod.rs` — one `class_pristine` bitset + one `class_game` per `order()` call, **gated
  on the exchange's own key** and shared by the exchange, the dense/hub twin and the spans. A union
  gate would build a kernel for dense/hub rows whose fill key then rejects every site — a fresh
  allocation on a row that built nothing before, which is the only place this refactor could *add*
  work. With the exchange key it cannot: every row either runs the exchange (and now shares) or
  behaves exactly as the frontier did.
* `rgreedy::MAX_N` 25 000 → **45 000**; `PRODUCTION_EXCHANGE_LEDGER` 2 GiB → **3 GiB**.

**No decision moves.** Every work ledger keeps its own `setup` charge, so which windows a budget
funds is unchanged; the kernel sharing is a pure allocation/copy/popcount removal. Verified row-by-row
against the pre-change log: **296 of 300 dev rows byte-identical**, and the four that move are exactly
the rows the ceiling admits. 126 unit tests pass (`cargo test --release -p ssi-candidate-worker`).

## 4. Measurements

One binary, one session, all 300 dev rows, the repository's own probe frame
(`probe_timing_and_score`, `#[cfg(test)]`, never compiled into the graded worker — the official
scorer minus the 2 s cap). Frontier baseline on this frame: **0.790425**, buckets
0.8873 / 0.8373 / 0.6826, reproducing the published ledger-2 GiB arm exactly.

| device | frame | Δ score | movers | regressions |
|---|---|---|---|---|
| shared kernel | 300-row probe | **0.0000** (296/300 rows byte-identical) | 0 | 0 |
| `MAX_N` 25 000 → 45 000 | 300-row probe, end to end | **−1.30e-4** | 4 | 0 |
| ledger 2 → 3 GiB | seam A/B on its rows | **−9.8e-5** | 3 + 2 band rows | 0 |

Ceiling movers (full-corpus probe): `mpbp_48` 0.4772 → 0.4746, `crudeoil_pooling_dt3`
0.7100 → 0.7056, `nd_netgen-3000-1-1-b-b-ns_7` 0.9599 → 0.9572, `arki0013` 0.4021 → 0.3993 —
every one of them inside the weight-0.40 `gt_10k` bucket.

Allowance curve, same binary, `SSI_EXCHANGE_LEDGER` seam:

| allowance | `crudeoil_lee4_10` | `crudeoil_lee4_09` | `nuclear10a` |
|---|---|---|---|
| 2 GiB | 181 053 364 | 128 851 771 | 55 261 703 |
| **3 GiB (shipped)** | **179 861 327** | **128 287 700** | **55 261 166** |
| 4 GiB | 179 065 997 | 128 088 057 | 55 261 166 |

3 GiB captures 44 % / 74 % of the 4 GiB step's value on the two largest movers, i.e. ≈ −9.8e-5 of dev
score, for about half its added work. Shipped values verified row-by-row on the final binary:
`crudeoil_lee4_10` 0.6107, `crudeoil_lee4_09` 0.6144, `nuclear10a` 0.6706, `mpbp_48` 0.4746,
`crudeoil_pooling_dt3` 0.7032, `arki0013` 0.3986, `nd_netgen-3000…` 0.9572, and
`chimera_selby-c16-02` unchanged at 0.5368.

Expected dev total: **≈ −2.3e-4 (0.790425 → ≈0.79019)** in this frame.

## 5. The cap trade, stated explicitly

The public ledger brackets the remote cap line in local seconds: a tree whose worst `order()` is
1.397 s passed, one at 1.536 s failed, and the 4 GiB allowance has a 0-for-5 remote record while the
2 GiB tree is the one that passed. So the allowance is bought back only halfway (3 GiB) and only on
top of a change that removes work from the very rows it loads: the shared kernel deletes nine of
eleven kernel constructions on the class rows the allowance binds on (n ≈ 16 000–18 000), which is the
same order as the +0.05 s the extra gibibyte costs there. Everything else is frontier-identical.

Nothing here is conditioned on wall clock, matrix identity, environment or any `#[cfg(test)]` code
path; the gates are `(n, nnz, max_deg)` only.

## 6. What was measured and rejected

* **4 GiB allowance** — worth a further −5.3e-5 dev on the same rows, refused: 0-for-5 remotely, and
  it is the one variable that separates every killed tree from the promoted one.
* **A sixth exchange sweep** — `SSI_EXCHANGE_SWEEPS` 5 → 6 changes **not one flop** on the eight rows
  it is supposed to move at the 2 GiB allowance: the allowance, not the sweep count, is what is
  binding here.
* **`MAX_N` above 45 000 / the class `nnz` key 200 000 → 300 000** — no dev row above 45 000 clears
  the class's own `nnz ≤ 200 000` key, so a higher ceiling adds no candidate; the `nnz` key reaches
  exactly one dev row (`gams05`, −1.4e-5) for +0.44 s on that row.
* **Giant-tier relabel** (`nnz > 700 000`, never admitted before): relabelled AMD
  aggressive/non-aggressive and AMF α5/α2.5/no-dense, seeds 1…8, on every row with n ≥ 100 000 —
  `unitcommit` best draw 0.9978 against an incumbent of 0.9764; `cont6-qq` 1.1500 against 0.6962;
  `faclay75` and `gabriel10` never beaten; `acopf` 0.9719 vs 0.9737 on one seed in eight at
  0.5–0.8 s per pass. Dead ground, not a loss.

## 7. Next steps

The remaining measured opening is the allowance's *unit of account*: `TripleWork::eliminate` charges
`(deg+1)(3w+6)` while the kernel touches only the non-zero words of each row, so sparse-fill rows are
over-charged by up to an order of magnitude relative to their real cost. Recalibrating that unit would
buy more windows per second on exactly the rows the allowance binds on, without admitting a single new
row — the only shape of device left that adds no cap class.
