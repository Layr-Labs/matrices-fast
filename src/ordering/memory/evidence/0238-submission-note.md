# The sparse-span schedule's narrow widths, stacked on the shared-kernel tree

**Model:** deepseek-v4-flash · **Harness:** DSH (DeepSeek Harness)
Baseline: the promoted frontier `43c1ca7d` (hidden **0.840782**). This tree carries the whole of the
previous submission (`f9b2fe4e`, hidden 0.840725: one shared exact-kernel entry per `order()` plus
`MAX_N` 45 000) and adds one schedule change.

## 1. What changed

`PRODUCTION_SPAN_WINDOWS` grows from 9 to 17 entries: the eight narrow widths `4, 5, 13, 15, 16, 17,
18, 20` at 64 M, beside the shipped `48, 9, 8, 12, 7, 10, 11, 14, 6`. Nothing else in `order()`
changes; the acceptance at every span entry is still a strict exact decrease against the incumbent,
so no row can regress.

## 2. Why this device, and why it had to wait

The class is **saturated on the rows it already covers**: at the 2 GiB allowance the sixth exchange
sweep changes not one flop on the eight rows it is supposed to move, and the allowance ladder's
dev value sits on three rows only. So on covered rows the only thing that can still move a
permutation is a *different neighbourhood*, and the wide-span schedule is exactly that: spans up to
64 positions wide, with the exact component DP still capped at fourteen vertices.

It had to wait because each entry pays its own replay, and the previous attempt at this family —
thirteen widths, on a tree without the shared kernel — was cap-killed, with the added work priced at
roughly three times the four-width step before it (worst row 1.221 s → 1.266 s for four widths).
Two things changed since: the class now shares one pristine image and one kernel per `order()`
instead of building up to eleven (`f9b2fe4e`), which is the same order of time back on every row the
spans run on; and this arm takes the conservative half of that schedule — eight widths, not thirteen.

## 3. What is measured

* The eight widths are the ones the repository's own `0230`/`0226` pages measured at **−3.5e-5 dev**
  against the nine-window schedule (`0.791551` for +8 widths vs `0.791586` for the shipped nine, one
  binary, one session, 300 rows), with `+13` adding only a further −7e-6.
* This tree's own spot check (seven rows spanning the range: `rsyn0810m04m`, `mpbp_48`,
  `crudeoil_pooling_dt3`, `crudeoil_lee4_10`, `chimera_selby-c16-02`, `nd_netgen-3000-1-1-b-b-ns_7`,
  `arki0013`) reproduces every previous value exactly and moves one row:
  `chimera_selby-c16-02` 2 297 782 → **2 297 664**.
* The full-corpus run of this arm was still in flight when the submission was queued; the claim here
  is the prior receipt for the schedule plus the spot check, not a fresh end-to-end number.

## 4. Cap position

Unchanged from `f9b2fe4e` except for the eight span replays on the rows where the spans are
affordable (they are silently refused by the budget pre-screen above roughly n = 20 000, so the band
the ceiling admits pays none of this). The allowance stays at the 2 GiB the promoted tree carries —
3 GiB and 4 GiB are both cap-killed, twice each on this corpus day — and the shared kernel's saving
(one kernel construction per class row instead of up to eleven) is what pays for the new entries.

No gate reads wall clock, matrix identity, environment or any `#[cfg(test)]` path; the gates remain
`(n, nnz, max_deg)`.

## 5. Prior work this tree stands on (all in `src/ordering/`)

* **One shared exact-kernel entry per `order()`** (`f9b2fe4e`, hidden 0.840725). Every class entry
  used to build its own elimination kernel: `Game::build_adj` (`n·⌈n/64⌉` allocation plus an `O(nnz)`
  bit-set scan) and `Game::new` (that array copied **and** popcounted for `deg0`). A row that runs the
  exchange, its dense/hub twin and nine spans entered that path up to eleven times on one immutable
  pattern. `Game::new` + `reset()` is the same state as `reset()` alone and the sweep loop resets
  first, so one shared kernel, reset per sweep, is bit-identical: 296 of 300 dev rows verified
  byte-identical, and the four that move are exactly the rows `MAX_N` 45 000 admits. The class sites
  take it through new `subset_window_descent_step_with_game` / `sparse_span_window_descent_with_game`
  entry points; the plain ones remain thin wrappers so probes and unit tests keep their own-local-kernel
  semantics. `window_pass_affordable` refuses a pass whose budget cannot fund `head + setup + one
  sweep` before the build (output-neutral: such a pass returned `None` with no change anyway), and
  `window_descent_precheck` keeps malformed patterns away from `col_ptr[v..v+1]`.
* **`MAX_N` 25 000 → 45 000**, priced end-to-end at −1.30e-4 dev on a full 300-row probe, four movers
  (`mpbp_48`, `crudeoil_pooling_dt3`, `nd_netgen-3000-1-1-b-b-ns_7`, `arki0013`), zero regressions.
* **Allowance held at 2 GiB.** The 3 GiB and 4 GiB rungs were both submitted today and both were
  cap-killed (`RUN FAILED: hidden matrix: order() exceeded the 2.0s per-matrix cap and was killed`);
  every 2 GiB tree completes.

## 6. Falsified this session (so the next attempt does not repeat them)

* Allowance 2 → 3 GiB (worth ≈ −9.8e-5 dev on three rows, graded-frame worst row 1.100 s against
  1.010 s for 2 GiB): killed remotely.
* A sixth exchange sweep: inert at the 2 GiB allowance.
* Giant-tier relabel (`nnz > 700 000`, seeds 1…8, relabelled AMD aggressive/non-aggressive and AMF
  α5/α2.5/no-dense): on `unitcommit` the best of ten passes is 0.9978 against an incumbent of 0.9764;
  on `cont6-qq` 1.1500 against 0.6962; `faclay75` and `gabriel10` never beaten; `acopf` only 0.9719 vs
  0.9737 on one seed in eight.
* Class `nnz` key 200 000 → 300 000: reaches one dev row (`gams05`) for +0.44 s on it.
* The union-gated shared kernel (building a kernel on dense/hub rows that then use none): fixed by
  gating on the exchange key; that build was the only place the refactor could add work.

## 7. Caveats and next steps

* The arm's own end-to-end number is not claimed; the prior receipt for the schedule and the spot
  check are. If it is below the 1 bip bar the schedule can be extended to the thirteen-width form, but
  only after the shared kernel's saving is measured on the rows the spans run on.
* The one measured opening that adds no new row class is the allowance's *unit of account*:
  `TripleWork::eliminate` charges `(deg+1)(3w+6)` while the kernel touches only each row's non-zero
  words, so sparse-fill rows are over-charged relative to their real cost and the allowance binds
  before the wall clock does. Recalibrating that unit rebalances windows per second between row
  classes instead of buying more work.
