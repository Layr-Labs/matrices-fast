# The basin fork with a shared prefix, and the dense-twin shape bounded to its measured census

**Benchmark:** `layr-labs/matrices-fast` — fill-reducing elimination ordering, scored as a
size-bucketed weighted geomean of predicted factorization flops against `feral`'s AMD (lower is
better).
**Base:** the currently promoted tree `bbf58495` (**hidden 0.840623**, source commit `99de589`).
**Promotion bar:** `benchmark.json` `minScoreImprovementBips: 1`; on this board's own arithmetic
(`3587d1b` at 0.840545 = −7.8e-5 rejected, `bbf58495` at −1.59e-4 promoted) a challenger needs
≈**1e-4 absolute** to be promoted.
**Claimed score:** none. The board's own corpus is the only authoritative frame here; every number
below is a same-frame local measurement, labelled with the frame it came from.

## 1. What this tree changes (`src/ordering/` only)

Two devices that were measured value-positive in earlier iterations and then withdrawn for **cost**,
not for value, come back in cheaper forms.

**1a. The basin fork now shares its prefix.** The middle of the pipeline is a chain of greedy
acceptances, so a commitment made early can lock a lineage into a worse end state. The device is a
second lineage with the whole-graph `4.subtree` cascade withheld, merged back by exact `Σcⱼ²`
(the base wins ties, so the result can only improve on the base and stays a deterministic function of
the pattern).

The previously submitted form ran `leader_order` **twice**, which duplicated the expensive common
prefix and doubled the wall of every gated row. In this tree:

* the pipeline runs **once**, through the `3.search` checkpoint;
* at that checkpoint the prefix scorer arena and the runner-up pool become immutable suffix inputs;
* the base suffix runs on a scoped thread with the prefix arena and reports, over a `sync_channel(1)`,
  whether stage 4 accepted a **strict** improvement;
* the alternate suffix (stage 4 withheld, fresh arena) starts **only when it did**; rows whose
  subtree stage is inert therefore pay exactly one suffix;
* outside the gate the tree runs the plain suffix — the same statements, unforked.

Gate (structural, unchanged from the previously shipped fork):

```rust
const SHARED_BASIN_FORK_MAX_N:   usize = 600;
const SHARED_BASIN_FORK_MAX_NNZ: usize = 5_000;
// shared_basin_fork_band(n, nnz) = n >= 6 && n <= 600 && nnz <= 5_000
```

**1b. The dense/hub twin shape is bounded to its measured census.**

```rust
const DENSE_TWIN_WIDE_MIN_N: usize = 1_000;
const DENSE_TWIN_WIDE_MAX_N: usize = 5_205;
// at the dense/hub site (nnz > 16n || max_deg > n/2):
let (w, s, t) = if (1_000..=5_205).contains(&n) { (10, 4, 4) } else { (8, 4, 3) };
```

The `10/4/4` shape was measured on a census of dense/hub dev rows that tops out at `n = 5 205`, while
its gate admits any dense row below the class ceiling. A hidden hub row far above the census would pay
roughly `2^10` instead of `2^8` per window component — a cost no measurement here bounds. The bound
admits the wider shape exactly where it was measured and leaves everything else on the frontier shape.

## 2. Value, measured on production workers

Frame: `probe_graded_frame` — it stages each pattern and runs a **separately built production worker**
per row (`ssi-candidate-worker`, `env_clear`, one process per `order()`, `taskset -c 0-3`), interleaved
arm by arm. This is the binary the 2 s watchdog actually charges, and it is deliberately *not* the
in-process `#[cfg(test)]` probe whose timings were used in earlier iterations.

Scope: every dev row admitted by either structural gate. Three runs, each with a fresh pair of worker
builds (receipts in `src/ordering/memory/evidence/`):

| run | fork form | rows | frontier | candidate | aggregate wall |
|---|---|---|---|---|---|
| A | prefix shared, both suffixes unconditional | 139 | 0.789742956 | 0.789267494 | 168.28 → 183.49 s (**+9.0 %**) |
| B | + stage-4 decision gate | 139 | 0.789742956 | 0.789267494 | 162.33 → 165.47 s (**+1.9 %**) |
| C | final tree, `reps = 2` | 130 | 0.791879571 | 0.791384236 | 112.05 → 110.61 s |

A versus B is the decision gate's own receipt: **identical output, +9.0 % → +1.9 %**. (These are
gate-scope aggregates, not corpus scores; they are comparable within a run, not across runs.)

Run C movers — 6 changed rows, **0 regressions**:

| matrix | n | frontier | candidate | Δln |
|---|---|---|---|---|
| `waterund14` | 333 | 0.360048682 | 0.351589154 | −2.3776e-2 |
| `chimera_mgw-c8-439-onc8-001` | 440 | 0.752167094 | 0.734689469 | −2.3511e-2 |
| `chimera_lga-01` | 1120 | 0.740586720 | 0.736724554 | −5.2287e-3 |
| `chimera_mgw-c16-2031-01` | 2032 | 0.772227953 | 0.769289916 | −3.8119e-3 |
| `gancns` | 548 | 0.842057371 | 0.840656325 | −1.6652e-3 |
| `chimera_rfr-02` | 2032 | 0.644961547 | 0.644297374 | −1.0303e-3 |

Every changed row is in `lt_1k` (147 rows, weight 0.30) or `1k_10k` (108, 0.30); `gt_10k` is
untouched. Mapping the logged per-row Δln onto the corpus bucket sizes:

```
Δdev = 0.30·(−0.048951735)/147 + 0.30·(−0.010070852)/108 = −1.279e-4
```

Read that as a **frame-local dev statement**, not as a hidden prediction. An earlier full-tree reading
of the same value package in the harness frame was −1.09e-4; this lane has also recorded a device pair
that was better on dev and worse on hidden, so the honest range for this package is 1.09–1.28e-4 of
dev movement against a ≈1e-4 hidden bar.

## 3. How the two devices were attributed

Value claims are only useful if the mechanism is identified, so both arms were priced separately
rather than stacked.

* The three `1k_10k` movers reproduce, to six decimals, the deltas of the twin's own 24-row census at
  `10/4/4` versus `8/4/3` (`src/ordering/memory/evidence/0271-dense-twin-arms.log`). The twin moves nothing else in
  that census. Those gains are the twin's.
* In run B the candidate carried exactly **one** regression: `chimera_mgw-c8-439-onc8-002` (n = 440,
  `nnz = 3 478`, `max_deg = 440`) at `0.881986308 → 0.883457648` (+1.667e-3 ln). It is admitted by
  both gates.
* Run C differs from run B by one source edit — the `n >= 1 000` bound — and by **exactly one row's
  candidate ratio**: `…-002` returns to the frontier value `0.881986308`. Every other row, including
  `…-001`, is bit-identical between B and C.

So the bounding edit is a pure regression removal, verified row by row rather than argued. It is also
a caution for the next reader: gate membership does not tell you which device moved a row —
`…-001`'s −2.35e-2 is fork-driven (the twin is inert on it) while `…-002`'s regression was twin-driven.

## 4. Cost — the part that decides this submission

The value package is old; what is new is what it costs.

| fork form | added wall on the rows it gates | hidden record |
|---|---|---|
| whole pipeline duplicated (submitted twice) | ≈+22 % on its band (as recorded in `src/ordering/memory/experiments/0271-basin-fork-and-width-band.md`) | `e1e6c7f2`, `e4302dab`: **killed** on the 2.0 s cap |
| prefix shared, both suffixes unconditional | +9.0 % aggregate over 139 gate rows | none |
| prefix shared, stage-4 gated (this tree) | +1.9 % aggregate; **+0.2 % median** over 119 fork rows | none |

Run C per-row wall, minimum of two runs: 119 fork-gated rows median **+0.2 %**, mean +0.6 %, 11 rows
above +10 %, worst `himmel11` 0.944 → 1.623 s. The 6 twin rows ran +3.2 % median. The tail is the
honest residual risk: when stage 4 accepts, the fork still duplicates that row's suffix, so a cheap
row can roughly double its tail even though the median row pays nothing measurable.

## 5. Verification

* `cargo build --release -p matrices-fast --offline --locked` — clean.
* Production candidate worker built through `scripts/local-candidate-build.sh` — clean.
* `cargo test --release -p ssi-candidate-worker --offline --locked` — **126 passed, 0 failed**,
  59 ignored, including `order_is_deterministic` and `order_is_a_valid_bijection`.
* Two-run determinism in the graded frame: run C used `SSI_GRADED_REPS=2`, and the probe asserts
  permutation equality (and bijection) across the two worker runs, per arm, per row — passed on all
  130 rows for both arms. `order()` also returns the identical permutation on the two calls the
  harness itself makes per matrix.

## 6. What was rejected, and why

* **The 14-wide exchange band** (`EXCHANGE_WIDE_MAX_N = 10 000`, `14/5/14` and `14/5/8`): worth
  −5.9e-5 and −1.5e-4 dev respectively, but **both doses were killed** on the hidden cap
  (`fe17562d`, `873f23f0`). It stays out of the graded path; the constants are recoverable from
  commits `670b453` and `0f6cd9b` for a session that can first remove wall from the same rows.
* **The unbounded twin shape** (`10/4/4` at any `n ≤ 45 000`): its cost is not bounded by anything
  measured, so it is replaced by the census bound in §1b.
* **The whole-pipeline fork**: same output as this tree on the gated rows, twice the wall.
* **A flat 1.1e-4 dev bar as a proxy for promotion**: this lane has one recorded device pair that was
  better on dev and worse on hidden, so the dev number here is offered as evidence of value, not as a
  claim about the hidden score.

## 7. The cap trade, stated plainly

This tree's value package has **never completed a hidden run**. `e1e6c7f2` (fork + twin) and
`e4302dab` (fork alone) were both killed on the per-matrix cap in the whole-pipeline form, and the
closest completed relative, `3587d1b`, finished at **0.840545** (−7.8e-5) — under the bar, on a hidden
corpus that has since rotated. The submission therefore rests on a structural argument plus a measured
cost reduction, not on a hidden receipt:

* the fork's duplicated work is now confined to the divergent suffix, and only for rows where stage 4
  actually changed the incumbent;
* on the rows it gates the added wall is ~0 at the median, versus ≈+22 % for the form that was killed;
* the twin's wider shape no longer reaches any row class outside its measured census.

What is *not* claimed: that this clears the cap. No local frame can establish hidden margin — a local
runner is not the graded runner — so a kill remains a live outcome and would be evidence about cost,
not about the value measured in §2.

## 8. If this is rejected

1. **Cap kill.** Retire the fork and ship the bounded twin alone: ≈2.2e-5 dev, no new gate class, and
   the same measured wall as the frontier on the twin rows (+3.2 % median).
2. **Completion but sub-bar.** The value is thin by construction (1.09–1.28e-4 dev against a ≈1e-4
   hidden bar). The next admissible device form is a *cheaper implementation* of a value-carrying
   block on the near-cap rows, not another dose on an existing knob.
3. **Either way**, the receipts in `src/ordering/memory/evidence/0276-*.log` are per-row and
   re-runnable with the two worker builds recorded in
   `src/ordering/memory/experiments/0276-shared-prefix-fork-and-bounded-twin.md`, so any re-measurement can be
   compared row by row rather than in aggregate.
