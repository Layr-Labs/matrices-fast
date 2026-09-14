# A strictly monotone exchange-admission policy, on top of the best-completing header

**Benchmark:** `layr-labs/matrices-fast` — fill-reducing elimination ordering, scored as a
size-bucketed weighted geomean of predicted factorization flops against feral's AMD (lower is
better; AMD = 1.00).

**Base:** the promoted tree `bbf58495` / source `99de589` (current best **0.840623**).

**Claimed score:** none.

## 1. What this submission is

Three production constants over the promoted tree, each one measured, each one unable to lose
value on any row it reaches:

| # | change | measured effect |
|---|---|---|
| 1 | `PRODUCTION_XCH_ALLOC` **0 → 1** — the class-block exchange's window walk now *skips* a component its precharged ledger cannot fund and keeps walking, instead of abandoning the rest of the window at the first refusal | dev **0.790236 → 0.790230** (one binary, one session, 300/300 rows): **299 rows bit-identical, 1 mover, 0 regressions** |
| 2 | the dense/hub twin shape is `10/4/4` for `1 000 <= n <= 5 205`, `8/4/3` elsewhere (census-bounded) | one mover in the worker frame (`chimera_lga-01` −0.52 %), all other ratios bit-identical |
| 3 | the shared-prefix basin fork and the `PEO_ALT_MAX_N` 50 000 → 10 000 narrowing are **not** in this tree | they were in the two trees that were killed or scored worse today; both stay compiled in behind test seams |

## 2. Why the admission policy is safe to ship

The window DP precharges `2^k * (16k + 6w + 24)` against a deterministic work ledger before it
walks the window's connected components. The shipped policy 0 offers the components to that ledger
in position order and, at the first one the ledger cannot fund, ends the walk — so the ledger
effectively funds components in *position* order and abandons everything behind an expensive one.
Policy 1 keeps the precharge exactly as it is and only changes what happens on a refusal: skip that
component, keep walking the rest, in the same deterministic order. Three consequences, all
structural:

- **It cannot spend more.** The precharge is unchanged and every component still charges the same
  cost, so the window's total spend is bounded by the same ledger. Policy 1 only re-allocates what
  policy 0 already budgeted.
- **It cannot lose value.** Every reordering the walk produces is admitted by the pipeline's exact
  scorer with a strict `<` against the incumbent's `Sum c_j^2`. A component policy 0 never reached
  is a component whose improvement policy 0 also never got; policy 1 can add adoptions, never remove
  one.
- **It is deterministic and general.** The walk is a stable traversal of a deterministic
  enumeration (ties broken by the component's bit mask); it keys on ledger affordability, a
  structural property of the window, never on a matrix's identity.

**Measurement.** Two full-corpus arms in one binary, one session, graded-closest frame
(`SSI_MARK_NOSCORE=1`):

```
SSI_XCH_ALLOC=0   SCORE = 0.790236
SSI_XCH_ALLOC=1   SCORE = 0.790230
```

**299 of 300 rows return bit-identical flop counts**; the single mover is `crudeoil_lee1_07`
(n = 3 670, ratio 0.745866999 → 0.744049962, Δln = −2.44e-3). There are **no regressions**.

**Worker-frame check.** One production child process per row, min of 3 runs, 13 rows including the
mover, `SSI_STAGE_KEEP_ENV=1` so the seam reaches the worker: no systematic wall change (the mover
reads 3.49/3.62/3.65 s at policy 0 and 3.50/3.82/3.88 s at policy 1, i.e. inside this host's run
spread; every other row is within noise or faster). That is what the precharge argument predicts.

## 3. Why the fork and the narrowing are not here

Both are hidden-measured today, and both are excluded on that evidence:

- the **basin fork** (+margin, with this twin and the narrowed window) is the tree that completed at
  **0.840946**; without the margin it was **killed at 66.6 s** of Benchmark wall. Its own receipt
  charges it **+0.68 s on a 14-vertex row** and **+0.83 s on a 399-vertex row**, on both of which its
  output change is zero. It remains behind `SSI_SHARED_BASIN_FORK=1`.
- the **`13.alt` narrowing to 10 000** was justified by "the chain yields zero above n = 10 000 on
  dev", which is exactly the shape of a device whose value lives on rows the dev corpus does not
  contain. `PEO_ALT_MAX_N` is back at the promoted 50 000.

So this tree's only departures from a configuration with a completion record are the twin band
(one mover, bit-identical elsewhere) and the admission policy (one mover, bit-identical elsewhere).

## 4. Cap accounting

The header this tree most resembles is the promoted crown, which completes the hidden corpus; the
two devices on top of it are the two cheapest value devices in the tree. Neither adds work outside
the class block: the admission policy redistributes an existing per-window budget, and the twin
band adds one window sweep on dense rows in a 4 205-wide span (measured **+3.2 % median** on the six
dev rows it touches, with the row ratios bit-identical on four of them). The unmodified crown's own
dev census puts the corpus mean at **1.90 s against the 2.00 s per-matrix cap** (95 %), so the
margin for added work is thin; the argument here is that this tree adds the least work of any
value-carrying form this session has measured.

## 5. Verification

- `cargo build --release -p matrices-fast --offline --locked` — clean.
- Production candidate worker via `scripts/local-candidate-build.sh` — clean (this shell cannot
  create bubblewrap namespaces, so the documented `SSI_ALLOW_UNSANDBOXED_WORKER=1` opt-out was used;
  the graded frame is unaffected).
- `cargo test --release -p ssi-candidate-worker --offline --locked` — **126 passed / 0 failed**,
  56 ignored.
- Determinism: the worker-frame runs use one fresh process per row and assert an identical returned
  permutation across repetitions. The admission policy is pure: same enumeration, same order, same
  ledger. `order()` reads no clock, environment, filesystem or `HashMap` iteration order; every seam
  named here is `#[cfg(test)]`-only and compiles to a constant in the graded worker.

## 6. Rejected this session, with the measurement that rejected it

- **The basin fork in every form tested** — see §3.
- **The `13.alt` window narrowing** — see §3.
- **Widening the displaced-ordering pool** (`PEO_ALT_SEEDS` 8 → 32): wall-free by construction, but
  20 dev rows give 3 better / **3 worse**, worst `multiplants_stg5` 0.412188 → 0.426526 (+3.48 %).
- **A cheaper sparse-span schedule in the terminal tail** (15.9 % of corpus wall, dominated by the
  nine span windows at 0.14–0.30 s per firing row): their measured value is why they exist; the
  substitution is a score trade, not a free saving.
