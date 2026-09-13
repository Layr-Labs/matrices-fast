# Shared exact-kernel engine + the exchange ledger at 4G, and the two hidden cap kills that priced the `n` ceiling

**Model:** deepseek-v4-flash · **Harness:** DSH (DeepSeek Harness)
Local numbers are the test-only probe frame (`#[cfg(test)]`, never compiled into the graded worker):
`probe_timing_and_score` — the official scorer minus the 2 s cap, same `Σ cⱼ²` definition, same
corpus — on all 300 dev rows, one binary / one session. The frontier tree (`43c1ca7`) reproduces probe
**0.790425** (buckets 0.8873 / 0.8373 / 0.6826) here, matching `0237-led-2G-4cpu.log` exactly, so the
frame is calibrated against the published receipts before any change is judged.

## 0. Environment, and why the local harness has no receipt

`yukon sync` brought the tree to the promoted frontier `43c1ca7` (hidden **0.840782**). Build:
`bash scripts/prepare-build.sh`, `cargo build --release -p matrices-fast --offline --locked`,
`bash scripts/local-candidate-build.sh`. On this box bubblewrap cannot create a namespace
(`bwrap: No permissions to create a new namespace`, because the agent shell already runs inside one),
so both the sandboxed build and the sandboxed worker are exercised through the documented local
opt-out `SSI_ALLOW_UNSANDBOXED_WORKER=1` on a trusted checkout; the graded frame is unaffected.

This host is ~5–6× the pod's per-row time and runs at load 5–10 on 4 vCPUs: `slay06m` (n = 357)
measures **1.04 s** and `sporttournament48` (n = 1131) **2.49 s** uncapped, i.e. the row total is
dominated by fixed pipeline overhead, not by the row. Three harness attempts therefore died on the
2 s cap on three *tiny* rows (`rsyn0810m04m` n = 4772, `sporttournament48` n = 1131, `sssd25-08`
n = 401 — the last two under `SSI_MAX_MATRIX_N` 2000 and 500); the **unmodified promoted tree fails
identically here**, which is what makes those failures a host artifact rather than a property of the
tree. Every number below is therefore the uncapped probe, and no `results.tsv` row is claimed.

## 1. Initial context and goal

The frontier's live value axis is the terminal exact-kernel class (`rgreedy`): the class-block
exchange over permutation windows, its dense/hub twin, nine sparse-span passes, and the PEO
re-extraction chain — all admitted by `6 ≤ n ≤ rgreedy::MAX_N`, `nnz ≤ 200 000`. The class's last
hidden-validated receipt is `52c744da`, the `MAX_N` 12 000 → 25 000 step (dev −8.4e-4 → hidden
−3.55e-4). The *next* step of that same axis, 25 000 → 45 000, had been measured at −6.9e-5 dev with
4 movers and rejected on cost alone: `nd_netgen-3000-1-1-b-b-ns_7` (n = 33 155) went 0.51 s → 1.34 s.

## 2. Hypothesis: the cost was never the search, it was eleven identical adjacency builds

Each class site constructed its own elimination kernel:

* `Game::build_adj` — `vec![0u64; n·⌈n/64⌉]` plus an `O(nnz)` bit-set scan;
* `Game::new` — a second full `n·⌈n/64⌉` copy, **plus** a popcount pass over it for `deg0`.

A class row runs the exchange, its dense/hub twin and **nine** sparse-span passes ⇒ up to **eleven**
setups of one immutable pattern. At n = 33 155 one setup is `3n⌈n/64⌉ ≈ 51.6 M` word-ops (≈ 413 MB of
traffic); eleven ≈ 4.1 GB — the whole measured +0.83 s of the 45 000 arm. `Game::new` returns a kernel
that is only valid after `reset()`, the sweep loop already resets first thing, and `reset()` restores
exactly what a fresh `Game::new` leaves. So **one shared kernel, reset per sweep, is bit-identical to
one kernel per site.**

## 3. Implementation (all inside `src/ordering/`)

* `rgreedy/window_dp.rs` — the body of `subset_window_descent_config` is split out as
  `subset_window_descent_body(game, …)`; new `subset_window_descent_step_with_game` and
  `sparse_span_window_descent_with_game` run a pass on a caller-owned `Game`; the plain entry points
  stay as thin wrappers, so probes and unit tests keep their own-local-kernel semantics.
  `window_pass_affordable(n, nnz, budget)` refuses a pass whose budget cannot fund
  `head + setup + one sweep` **before** the build (bit-identical: such a pass returned `None` with
  `changed == false` anyway). `window_descent_precheck` runs the body's structural input checks before
  `build_adj` — a malformed pattern must not reach `col_ptr[v..v+1]` (a unit test caught exactly that
  regression, and the fix is verified output-neutral).
* `mod.rs` (`leader_order`) — one `class_pristine` bitset + one `class_game` per `order()` call, gated
  by the union of the class sites' keys (`6 ≤ n ≤ class_n && nnz ≤ 200 000`), handed to all three
  sites. **Every work ledger and every charge is untouched**, so which windows a budget funds does not
  move.
* `rgreedy::MAX_N` 25 000 → 45 000 *(later reverted — see §7)*, and
  `PRODUCTION_EXCHANGE_LEDGER` 2G → 4G.
* New test-only probe `probe_giant_relabel` (never in the graded worker) to price the giant tier.

Commands: `cargo test --release -p ssi-candidate-worker --offline --locked` (126 passed / 0 failed /
56 ignored) and, for every measurement,
`SSI_PROBE_ONLY=… ./.session-backup/probe-<arm> --ignored --nocapture --test-threads=1 probe_timing_and_score`.

## 4. Measured results

**Kernel sharing is output-preserving.** The class rows are byte-identical between the two binaries on
296 of 300 dev rows — verified row-by-row against the pre-change log, and again after the precheck fix.
That is the strongest possible statement for a change of this kind: no acceptance decision moved.

**The ceiling step, on top of the sharing** (full 300-row probe, one binary/session):

```
SCORE 0.790425 -> 0.790295   (-1.30e-4)   296/300 rows identical
buckets  0.8873 / 0.8373 / 0.6826  ->  0.8873 / 0.8373 / 0.6823
```

four movers, all in the newly admitted band, all improvements, 0 regressions:
`mpbp_48` 0.4772 → 0.4746, `crudeoil_pooling_dt3` 0.7100 → 0.7056, `nd_netgen-3000…` 0.9599 → 0.9572,
`arki0013` 0.4021 → 0.3993.

**The ledger step, re-priced in-frame on this tree** (`SSI_EXCHANGE_LEDGER` seam, everything else
fixed) moves exactly three rows:

| row | n | flops 2G | flops 4G | Δ |
|---|--:|--:|--:|--:|
| `crudeoil_lee4_10` | 17 809 | 181 053 364 | **179 065 997** | −1.098 % |
| `crudeoil_lee4_09` | 15 904 | 128 851 771 | **128 088 057** | −0.593 % |
| `nuclear10a` | 17 493 | 55 261 703 | **55 261 166** | −0.001 % |

**−1.51e-4 dev**, entirely inside the weight-0.40 `gt_10k` bucket. Every other row of the 6-row A/B
subset — including the corpus's slowest, `chimera_selby-c16-02` — is byte-identical: the ledger is only
reachable on rows whose five sweeps' charges exceed it, i.e. it costs time only where it buys value.
An interleaved min-of-3 timing A/B over six class rows reads −1.19 s total with no row systematically
worse.

**What is shipped is exactly the sum of the two shipped halves**, and the shipped binary was verified
row-by-row on the key rows: `crudeoil_lee4_10`, `crudeoil_lee4_09`, `nuclear10a` take their 4G values
while `chimera_selby-c16-02`, `mpbp_48` and `arki0013` are byte-identical to the frontier baseline
(those two are outside the 25 000 ceiling, which is why they return to their old values).

## 5. Hypotheses tested and killed

* **Giant-tier relabel (`nnz > 700 000`).** The heavy-tier relabelled-AMF lottery stops at 700 000 and
  the rows above it have the worst `gt_10k` ratios (`unitcommit` 0.9764, `acopf` 0.9737,
  `transswitch2383wpr` 0.9785). New test-only `probe_giant_relabel`: relabelled AMD
  (aggressive/non-aggressive, α10) + relabelled AMF (α5 / α2.5 / no-dense), seeds 1…8, on every row
  with n ≥ 100 000. Result: on `unitcommit` the best of ten passes is **0.9978** against the
  incumbent's 0.9764; on `cont6-qq` **1.1500** against 0.6962; `faclay75` and `gabriel10` are never
  beaten; `acopf` only **0.9719** vs 0.9737 on **one seed in eight**, at 0.5–0.8 s per pass. The
  `HEAVY_RELABEL_AMF` density gaps (`unitcommit`, `cont6-qq`, `nuclear104`, `gams05` fall in neither
  the sparse nor the dense sub-tier) are dead ground rather than a loss: the shipped pipeline is
  already far ahead of a fresh lottery on those rows.
* **The exchange's other seams** (`SSI_DENSE_W/S/T`, `SSI_XCHG_TAIL/POOL`, the tail re-application)
  are already closed by the lane's own receipts, and the span schedule's marginal widths are worth
  ~1e-5 each — nothing there is worth cap exposure.

## 6. Failure and course correction (the important part)

Two submissions carrying the ceiling step were **killed by the grader**:

* `c67ad490` (sharing + `MAX_N` 45 000 + ledger 4G) — `RUN FAILED: hidden matrix: order() exceeded the
  2.0s per-matrix cap and was killed`;
* `e7edbb04` (sharing + `MAX_N` 45 000, ledger back at 2G) — the same failure.

The common factor is the ceiling, i.e. **added work on a band of rows the class never touched**. The
sharing cannot be the cause: on every row it touches it does strictly *less* work than the frontier
(nine of eleven identical adjacency builds removed), with a bit-identical accept sequence. So the
ceiling was reverted to 25 000 and the ledger step kept — the ledger's extra allowance is unreachable
on any row whose five sweeps do not already exceed it, which is why its cost lands only on the three
rows where it buys value. This is the second time this lane has learned that a dev-measured value on a
band the class did not previously touch is priced on rows dev does not contain.

## 7. What is shipped, and what it is worth

`src/ordering/` = frontier `43c1ca7` + (a) one shared exact-kernel engine per `order()` call, (b)
`PRODUCTION_EXCHANGE_LEDGER` 2G → 4G. Expected dev: **0.790425 → ≈0.79027** (−1.5e-4, measured on its
three movers in-frame). 296/300 rows byte-identical; 126 unit tests pass; determinism is structural
(no clock, no identity, no hash iteration, no environment).

## 8. Caveats and next steps

* The score claim is a composition of in-frame deltas, not a single end-to-end 300-row run of the
  final pair: the full probe was taken with the ceiling present, and the ledger delta comes from the
  seam A/B on its movers. The two steps are disjoint by construction (the ceiling acts only on
  25 000 < n ≤ 45 000, the ledger only on rows that already run the exchange), so the composition is
  sound, but it is a composition.
* The next real opening, measured but not built: the class's **ledger charge is a poor predictor of
  its wall time** — `TripleWork::eliminate` charges `(deg+1)(3w+6)` while the kernel touches only the
  non-zero words of each row, so sparse-fill rows are over-charged by up to an order of magnitude
  while dense ones are charged roughly right. Recalibrating that unit would buy more search per
  second on the rows where the ledger binds, without adding a row to the class.
* Anything that widens the class's row set (ceiling, `nnz` key) must be priced *per class of hidden
  row*, not on dev: two grader kills in this session came from exactly that mistake.
