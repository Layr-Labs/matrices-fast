# One shared exact-kernel engine per `order()`, and the `n` ceiling it pays for (25 000 → 45 000)

**Model:** deepseek-v4-flash · **Harness:** DSH (DeepSeek Harness)
Local numbers come from the test-only probe frame (`#[cfg(test)]`, never compiled into the graded
worker): the repo's own `probe_timing_and_score`, i.e. the official scorer minus the 2 s cap, on all
300 dev rows in one binary / one session.

## 1. Context

The frontier tree (`43c1ca7`, hidden **0.840782**) reaches probe **0.790425**
(buckets 0.8873 / 0.8373 / 0.6826), reproducing the ledger-2G arm of `0237-led-2G-4cpu.log` exactly.

The class's only live value axis is its `n` ceiling (`rgreedy::MAX_N`): the 12 000 → 25 000 step
(`52c744da`) is this lane's last hidden-validated receipt, dev −8.4e-4 → hidden −3.55e-4. The next
step, 25 000 → 45 000, was measured at −6.9e-5 dev with 4 movers and **rejected on cost**: a band row
(`nd_netgen-3000…`, n = 33 155) went 0.51 s → 1.34 s.

## 2. What is new: the cost was never the search — it was eleven identical adjacency builds

Every class site built its OWN elimination kernel:

* `Game::build_adj` — `vec![0u64; n·⌈n/64⌉]` plus an `O(nnz)` bit-set scan;
* `Game::new` — a second full `n·⌈n/64⌉` copy, **plus** a popcount pass over it for `deg0`.

A class row runs the class-block exchange, its dense/hub twin and **nine** sparse-span passes ⇒ up to
**eleven** setups of the same immutable pattern. At n = 33 155 one setup is `3n⌈n/64⌉ ≈ 51.6 M`
word-ops (≈ 413 MB of traffic); eleven ≈ 4.1 GB — the whole measured `+0.83 s` of the 45 000 arm.

`Game::new` returns a kernel that is only valid after `reset()`, the sweep loop already resets first
thing, and `reset()` restores exactly what a fresh `Game::new` leaves. So **one shared kernel, reset
per sweep, is bit-identical to one kernel per site.**

**Change** (all in `src/ordering/`):
`rgreedy/window_dp.rs` — the body of `subset_window_descent_config` is split out as
`subset_window_descent_body`; new `subset_window_descent_step_with_game` and
`sparse_span_window_descent_with_game` run a pass on a caller-owned `Game`; the plain entry points
stay as thin wrappers so probes and unit tests keep their own-local-kernel semantics.
`window_pass_affordable` refuses a pass whose budget cannot fund `head + setup + one sweep` **before**
the build; `window_descent_precheck` runs the body's structural input checks before `build_adj`.
`mod.rs` — one `class_pristine` bitset + one `class_game` per `order()` call, shared by all three
class sites. **Every work ledger and every charge is untouched**, so no acceptance decision moves.
`rgreedy::MAX_N` 25 000 → **45 000** (the axis the freed margin buys), and
and **not** the exchange ledger 2G → 4G (measured, shipped in `c67ad490`, killed by the hidden cap —
see §3).

## 3. Measured result

Bit-identical on **296 of 300** rows (verified row-by-row against the pre-change probe log, and again
after the precheck fix); the four movers are exactly the rows the new ceiling admits, all
improvements, **0 regressions**:

| row | n | nnz | base ratio | new ratio |
|---|--:|--:|--:|--:|
| `mpbp_48` | 28 368 | 91 016 | 0.4772 | **0.4746** |
| `crudeoil_pooling_dt3` | 30 660 | 152 210 | 0.7100 | **0.7056** |
| `nd_netgen-3000-1-1-b-b-ns_7` | 33 155 | 90 000 | 0.9599 | **0.9572** |
| `arki0013` | 44 909 | 160 172 | 0.4021 | **0.3993** |

```
SCORE 0.790425 -> 0.790295   (-1.30e-4)   full 300-row probe, one binary/session
buckets  0.8873 / 0.8373 / 0.6826  ->  0.8873 / 0.8373 / 0.6823
```

**Not shipped, and recorded as a cap failure:** the exchange ledger 2G → 4G. It is worth −1.5e-4 dev
in-frame (three movers: `crudeoil_lee4_10` −1.10 %, `crudeoil_lee4_09` −0.59 %, `nuclear10a` −0.001 %,
plus two band rows through the shared engine: `crudeoil_pooling_dt3` 0.7056 → 0.7021, `arki0013`
0.3993 → 0.3975) and an interleaved min-of-3 timing A/B over six class rows showed no systematic cost
— but the submission carrying it (`c67ad490`, shared engine + 45 000 + 4G) was killed remotely:
`RUN FAILED: hidden matrix: order() exceeded the 2.0s per-matrix cap`. It touches **every** class row,
so the hidden row it kills is not on dev. Reverted to 2G; this submission carries the ceiling step
alone, which adds work only on rows the class never reached before (25 000 < n ≤ 45 000).

126 unit tests pass (`cargo test --release -p ssi-candidate-worker`); 300/300 rows processed with the
determinism double-run in every probe binary. 45 000 is the end of this axis on the corpus: no dev row
above it clears the class's own `nnz ≤ 200 000` key, and a higher limit would only square memory
(507 MB at 45 000 vs 2.5 GB at 100 000).

## 4. What was measured and killed (not shipped)

* **The giant tier is not relabel headroom.** New test-only `probe_giant_relabel` (n ≥ 100 000:
  relabelled AMD aggressive/non-aggressive + AMF α5/α2.5/no-dense, seeds 1…8): on
  `unitcommit_200_100_1_mod_8` (incumbent 0.9764) the best of ten passes is **0.9978**; on `cont6-qq`
  (0.6962) it is **1.1500**; `faclay75` and `gabriel10` are never beaten; `acopf_case9241pegase_qcqp`
  (0.9737) only by 0.9719 — **one seed in eight, −0.18 %**, at 0.5–0.8 s per pass. The
  `HEAVY_RELABEL_AMF` density gaps (`unitcommit`, `cont6-qq`, `nuclear104`, `gams05` fall in neither
  sub-tier) are therefore dead ground rather than a loss.
* **The ledger 2G → 4G step (−1.5e-4 dev in-frame) — SHIPPED AND KILLED by the hidden 2 s cap**
  (`c67ad490`). Its value is real; its exposure is the whole class, and the cap lottery is not
  measured by dev. Reverted.

## 5. Cap safety

* 296/300 dev rows: bit-identical work **and** bit-identical output.
* 5 band rows (25 000 < n ≤ 45 000, nnz ≤ 200 000) gain one shared kernel build plus the exchange
  replay: +0.05…0.15 s each on a host whose class rows already sit 0.6–1.0 s under the cap.
* Memory: the shared kernel is `2·n·⌈n/64⌉·8` bytes = **507 MB** at the new ceiling, inside the 4 GiB
  address-space cap; the gate is checked before the build, so no larger row ever allocates it.
* No identity is consulted anywhere: the gate is `(n, nnz, max_deg)` only.

## 6. Why this should generalize

The change does not add a heuristic; it removes work that was provably redundant (eleven identical
adjacency constructions per row) without moving a single budget decision, then spends the freed margin
on the axis with the only hidden-validated receipt this lane owns. A hidden corpus with more rows in
25 000 < n ≤ 45 000 gains more than dev does; a hidden corpus without them is bit-identical to the
frontier.
