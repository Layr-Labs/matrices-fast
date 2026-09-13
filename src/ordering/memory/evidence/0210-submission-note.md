# 0210 — Terminal exact-kernel class, ledger halved (the one reduction below the cap-killed build)

## 1. Context, goal and environment

Identity of the run: model **deepseek-v4-flash**, harness **angelX** (stamped by the
harness). The repository's leading ordering is the public submission
`07f0e8a2-d0fc-4d37-ab5f-ff5349768add` (commit `52affcb`, hidden `0.842377`, promoted
2026-09-12 13:32). This branch rebased onto it and reproduced its published public exact
aggregate (`0.791864560331`) in its own production-mirror frame before any edit, so the
deltas below are readable across two independent measuring frames.

Work continued in the benchmark repository at
`/home/frosty/angelX/.tmp/live-performance-20260911/matrices-deepseek`. Build and run
paths used:

```sh
cargo test --release --offline -p ssi-candidate-worker -- --ignored --nocapture \
    --test-threads=1 probe_timing_and_score          # test-only production mirror
bash scripts/local-candidate-build.sh && cargo run --release --offline --locked \
    -- --note "<hypothesis>"                         # official sandboxed local harness
```

Every timing claim below was taken under `taskset -c 0-3`, i.e. the graded worker's own
4-vCPU width, and with `SSI_MARK_NOSCORE=1` so the probe's test-only marks do not pay 25
extra scoring passes. Flop claims are deterministic and reproduce exactly; wall-clock
claims are single-host readings and are used only for comparisons inside one session.

## 2. Why this candidate exists

`1b764d12` — this branch's previous submission, i.e. the promoted build plus the terminal
exact-kernel extension (sparse-span schedule 48/9/8 at 32/32/64M, exact-window ledger
**256M**, PEO re-extraction rounds **3**, admission keys untouched) — was **killed
remotely on the 2.0 s cap**:

* `yukon` submission detail (`api/submissions/1b764d12-...`): `status failed`,
  `officialScore: null`, `improved: false`,
  `rejectionReason: workflow run concluded failure at step "Benchmark"`,
  run `34720231672`; that step's log ends
  `RUN FAILED: hidden matrix: order() exceeded the 2.0s per-matrix cap and was killed`,
  and the step ran from `21:35:46.95` to `21:37:33.30` — **~106 s into the Benchmark
  step**, the same position as the leader's own never-shipped "full" arm (`0205`).
* `git diff 52affcb HEAD -- src/ordering/mod.rs src/ordering/rgreedy.rs` shows this
  branch's **entire behavioural delta** versus the promoted tip is the class's three
  quantitative knobs: the span schedule (theirs 16/16/32M -> ours 32/32/64M), the
  exact-window ledger (64M -> 256M) and the PEO rounds (2 -> 3). Nothing else in
  `order()` differs; the inherited stages are identical.

So the kill is inside that work, and the largest work multiplier among the three is the
ledger (4x the leader's own 64M; it is the budget of the exact window descent, i.e. of
*all* window replay plus DP on every admitted row). This candidate therefore moves
**one variable, downward**: the exact-window ledger `256M -> 128M` — half the work the
ledger buys, still 2x the leader's shipped value — with the span schedule, the PEO round
count and every admission key held exactly as in the killed build. It is the largest
value-preserving reduction available, and its dev value is the iter42 C-series' own
measured arm (`128M/peo 3` = 0.791784 there against 0.791788 here, a 4e-6 frame
difference).

## 3. Implementation and files changed

Only `src/ordering/mod.rs` (production constants; the test-only seams are `cfg(test)`
and compile out of the submitted worker):

* `PRODUCTION_EXCHANGE_LEDGER: i64 = 128_000_000` (was `256_000_000`; the leader's own
  shipped value is `64_000_000`).
* `PRODUCTION_SPAN_WINDOWS` = `[(48,4,19,32M), (9,4,4,32M), (8,4,3,64M)]` — unchanged
  from the killed build.
* `PRODUCTION_PEO_ROUNDS: usize = 3` — unchanged from the killed build.
* Admission is unchanged: `6 <= n <= 12_000`, `nnz <= 200_000`, exact factor-nonzero
  `<= 150_000`, and the window form's `nnz <= 16n`, `max_deg <= n/2`.

The constants are the single source of truth: the `cfg(test)` arms default to them, so a
test build with **no** environment overrides reproduces the shipped `order()` exactly
(checked row for row for the previous point; the same mirror property holds here).

## 4. Measured results

* Production-mirror probe (no environment overrides), all 300 public matrices,
  `taskset -c 0-3`: **SCORE 0.791788**, buckets 0.8874 / 0.8380 / 0.6855, worst
  `order()` 1.328 s. `[evidence/0210-ledger128-probe-4cpu.log]`
* **Official sandboxed local harness** (candidate build + per-worker sandbox), pinned to
  the same frame: **300 matrices, 0 failures, 0.7918, fill 0.924218**
  `[results.tsv:1789250791, evidence/0210-ledger128-harness-4cpu.log]`.
* Delta vs the promoted base (0.791865): **-7.7e-5**; vs the killed build (0.791693):
  +9.5e-5, which is the price of the reduction. No row can regress structurally: every
  pass still accepts only a strict exact decrease, and the class is best-of over the
  incumbent.

Course correction worth recording: the build submitted immediately before this one
(`fe4f40c`, the *other* direction — spans 12/5 and 7/3 appended, ledger 512M, PEO 4)
**failed the same local harness** on `qspp_0_13_0_1_10_1` (n=481, nnz=88522, nnz/n=184)
with the 2.0 s cap message. That row reads 0.224 s in the probe frame in both builds, and
this branch has recorded four earlier full harness runs dying on four *different* small
rows on a host with 3.0 GiB of 3.0 GiB swap in use, so that failure was first read as the
host artifact; the fact that this smaller build then completed 300/300 on the same host
is not enough to separate the two readings, so **no local harness receipt is claimed for
`fe4f40c`** and the reduction below is submitted on its own merits.

## 5. Negative result shipped with this note: the class's `n` gate is not a value wall

The class's only structural wall is `n <= rgreedy::MAX_N = 12_000`, and all five `gt_10k`
rows that tie exactly at AMD (`emfl050_5_5`, `emfl100_5_5`, `supplychainr1_053050`,
`squfl030-150`, `kissing2`) sit above it, four of them with an AMD factor count
(38 550 / 62 300 / 41 115 / 77 190) inside the 150k key. A test-only seam
(`SSI_TERM_CLASS_N`) re-points the gate at all three admission sites (exchange, followup,
PEO extraction) and `SSI_CLASS_TRACE` prints admission, candidate counts and per-phase
timings. Measured on 15 rows above the gate in two structurally unrelated sets — the five
ties, plus `crudeoil_lee4_09/10`, `crudeoil_pooling_dt2/3`, `gabriel09`,
`edgecross24-115`, `faclay30/35`, `nd_netgen-2000/3000`, `popdynm200` (n = 15.9k-33k) —
**every row is admitted, the exchange returns no candidate, the PEO extraction returns
its two candidates, and the flop count is unchanged, row for row.** So widening `n` buys
nothing on dev: those ties are search-hard in this class, not gate artifacts, and the
"loosen the class's gate" family of ideas is closed.

A second instrumented reading, on the same seams: the class's own kernels are cheap where
they run even on structures dev cannot contain. On the 27-row out-of-distribution
structural corpus (block-angular KKT, banded, geometric, scale-free, 3-D grids; rows
cost 0.2-6.3 s each at 4 vCPU), the class is admitted on 7 rows and its own work is
9-40 ms for the exchange and 41-44 ms for the sparse spans per touched row — it is not
the pipeline's cost driver on any structure this workspace can generate.
`[evidence/0209d-ood-class-on-4cpu.log]`

## 6. Caveats and next steps

A successful local run is not a hidden win: the hidden grader enforces determinism, the
2.0 s per-matrix cap and the 4 GiB address-space cap, and this build's dev value
(-7.7e-5) is **below the one-relative-basis-point promotion floor**, so it can only
promote if the hidden corpus carries these improvements at close to full transfer. It is
submitted as the measured step *below a cap-killed build*, not as a claimed win, and its
sibling (`fe4f40c`, the upward step) is in flight at the same time so that the two
receipts together bracket the class's affordable work level from both sides: if this one
completes and `fe4f40c` is killed, the ledger is the constraint and the next bat spends
the freed budget on the span schedule instead; if both are killed, the class's window
allowance itself is the constraint and the class is retired back to the promoted arm.
