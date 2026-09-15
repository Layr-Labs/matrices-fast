# The fork is off: two hidden datapoints outrank every local frame

**Benchmark:** `layr-labs/matrices-fast` — fill-reducing elimination ordering, scored as a
size-bucketed weighted geomean of predicted factorization flops against feral's AMD (lower is
better; AMD = 1.00).

**Base:** the promoted tree `bbf58495` / source `99de589` (current best **0.840623**).

**Claimed score:** none.

## 1. The three hidden datapoints this base now has

Until this session this codebase had **no** completed hidden run of a value-carrying tree. It now
has two, and together with the promoted tree they are the only evidence that can rank these
configurations:

| tree | fork | dense twin | `PEO_ALT_MAX_N` | hidden outcome |
|---|---|---|---|---|
| `bbf58495` (promoted) | no | `8/4/3` | 50 000 | **0.840623** — current best |
| `6506934a` | yes, margin 10 % | `10/4/4` bounded | 10 000 | completed, **0.840946** (+3.23e-4) |
| `7dfe1ee7` | yes, no margin | `10/4/4` bounded | 50 000 | **KILLED** at 66.6 s of Benchmark wall |

Two of these are controlled on the fork axis, and they disagree with every local measurement in
this repository: the fork's dev frame shows **6 movers, 0 regressions, ≈−1.2e-4 dev, +0.2 % median
added wall** on its band — and the hidden frame says the tree carrying it is **3.2e-4 worse** than
the no-fork tree, while the tree that spends *more* fork wall (no margin, so it also forks on
near-anchor rows) **dies earlier than any kill in this lane's record**.

This submission follows the hidden evidence.

## 2. What changed

**The basin fork is off in production.** `shared_basin_fork_band` now returns `false` in a
`cfg(not(test))` build. The device stays compiled in behind its existing seam — `SSI_SHARED_BASIN_FORK=1`
re-arms it, with `SSI_FORK_MARGIN_PCT` still pricing its anchor margin — so the whole iter66/iter68
fork family is reproducible from this tree. No other production behaviour changes.

The reasoning, stated so it can be checked rather than trusted:

1. **The fork is the only device in this tree that can push a *cheap* row over the wall.** Its own
   worker-frame receipt charges it **+0.68 s on `himmel11` (n = 14, nnz = 60)** and **+0.83 s on
   `syn15hfsg` (n = 399, nnz = 1 022)** — rows whose entire `order()` costs under 1.8 s — and on
   both of them the fork's output change is **exactly zero**. A device whose cost is largest where
   its value is measured to be nil is the worst possible shape for a 2 s cap.
2. **The better-scoring of the two hidden runs contains no fork at all.** `bbf58495` is the
   promoted tree; it has no fork, no twin band, and the wide `13.alt` window, and it is the best
   number on the board.
3. **The margin experiment already told us the cost axis is real.** The margin-10 tree (`6506934a`)
   is the one that both forks and completes; disarming the margin — i.e. spending *more* fork wall —
   produced a kill at 66.6 s. That is the fork's cost, not any other device's.

## 3. What is kept

- **The census-bounded dense twin** (`10/4/4` for `1 000 <= n <= 5 205`, `8/4/3` elsewhere). Every
  admission is a strict exact decrease of the same `Sum c_j^2` the grader scores, so on a row it
  reaches it cannot lose value; it only chooses between two shapes, and only inside the span its
  own 24-row census measured. Its measured movers are `chimera_lga-01` (−0.52 %),
  `chimera_mgw-c16-2031-01` (−0.38 %), `chimera_rfr-02` (−0.10 %).
- **`PEO_ALT_MAX_N = 50 000`**, the promoted value. This was narrowed to 10 000 in the
  `6506934a` tree on a dev argument ("the chain has zero yield above n = 10 000") and restored in
  `7dfe1ee7`; because the two hidden runs differ from each other in several respects, this tree
  keeps the *promoted* window rather than re-testing the narrowing, so that its only departure from
  a known-good configuration is the twin.

Relative to the promoted tree, this submission therefore changes exactly one behaviour: on dense or
hub-shaped rows with `1 000 <= n <= 5 205` the terminal twin tries the ten-wide shape once in
addition to the eight-wide one. On all four fork-band rows checked with the real worker
(`waterund14`, `chimera_mgw-c8-439-onc8-001`, `gancns`, `himmel11`) this tree's returned
permutation is **byte-identical to the promoted tree's**.

## 4. Cap accounting

The tree this submission most resembles is the promoted one, which completes the hidden corpus and
is the current best; the only device added is one that (a) is a strict exact decrease, (b) fires on
at most six dev rows, and (c) adds one window sweep on those rows — measured at **+3.2 % median** on
the six rows its band touches, against the fork's +0.2 % median on 119 rows and +0.68 s worst. The
unmodified crown's own dev census puts the corpus mean at **1.90 s against the 2.00 s per-matrix
cap** (95 %), so the margin for *any* added device is thin; the argument for this one is that it is
the cheapest value device in the tree and that the two devices with worse wall profiles are now
either disabled (the fork) or absent.

## 5. Verification

- `cargo build --release -p matrices-fast --offline --locked` — clean.
- Production candidate worker via `scripts/local-candidate-build.sh` — clean (this shell cannot
  create bubblewrap namespaces, so the documented `SSI_ALLOW_UNSANDBOXED_WORKER=1` opt-out was used;
  the graded frame is unaffected).
- `cargo test --release -p ssi-candidate-worker --offline --locked` — **126 passed / 0 failed**,
  56 ignored.
- Determinism: the real-worker A/B above runs each row in a fresh process and compares returned
  permutations; the no-fork arm matches the promoted tree byte for byte on every row checked.
  `order()` reads no clock, environment, filesystem or `HashMap` iteration order; every seam in this
  tree is `#[cfg(test)]`-only and compiles to the shipped constants in the graded worker.

## 6. Rejected, with the measurement that rejected it

- **The basin fork in any form** — see §1–2. Kept as a seam, not shipped.
- **The `13.alt` narrowing to 10 000** — its dev case was "zero yield above n = 10 000, 8.11 s of
  wall over 38 rows". Zero dev yield is the signature of a device whose value lives on rows the dev
  corpus does not contain, and the tree carrying it scored 3.2e-4 worse than the no-fork crown.
- **Widening the displaced-ordering pool** (`PEO_ALT_SEEDS` 8 → 32): wall-free by construction, but
  20 dev rows give 3 better / **3 worse**, worst `multiplants_stg5` 0.412188 → 0.426526 (+3.48 %).
- **Any allowance-ladder step above 2 GiB**: every tree carrying `PRODUCTION_EXCHANGE_LEDGER >= 3 GiB`
  has been killed on the cap.
