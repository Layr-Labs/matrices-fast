# 0276 — shared-prefix basin fork and the bounded dense-twin shape

- **Date:** 2026-09-14
- **Base:** the iter65-audit tree, i.e. frontier `bbf5849` behavior (`MAX_N` 45 000,
  `PRODUCTION_EXCHANGE_LEDGER` 2 GiB, 12 class-block sweeps, shared `Pristine`/`Game` memo, `ADJ_POOL`).
- **Measured value:** production-worker frame, 130 admitted rows, **6 movers / 0 regressions**;
  implied full-corpus dev delta **−1.279e-4** (bucket arithmetic below).
- **Status:** **shipped and cap-killed** — submission `7febce96` (2026-09-14); value measured in the
  production-worker frame, hidden run died on the per-matrix cap (see "Cap accounting — and the
  hidden verdict").

## Hypothesis

Two devices were measured value-positive in iter60–64 and then withdrawn from production after four
hidden cap failures: the **basin fork** (run `leader_order` twice, once with the `4.subtree` cascade
withheld, merge by exact `Σcⱼ²`) and the **dense/hub twin shape** (`8/4/3 → 10/4/4`). Both were
withdrawn for cost, not for value:

1. The submitted fork **duplicated the whole pipeline**, so it doubled the wall of every row it
   gated. If the common prefix is instead run once and only the *divergent suffix* is duplicated, the
   same output can be produced for a fraction of the added wall.
2. The twin's `10/4/4` was measured on a census that stops at `n = 5 205` while its gate is
   `nnz > 16n || max_deg > n/2` at any `n ≤ 45 000`. Bounding the shape to the measured census
   removes the unmeasured large-hub class without touching the rows that were actually measured.

## What changed (`src/ordering/` only)

* **Fork mechanics (`mod.rs`).** Everything through `3.search` now runs once. At that checkpoint the
  prefix scorer arena (`scoring_ws::ScoreWorkspace`) and the runner-up pool become immutable suffix
  inputs. The suffix is a local `run_suffix(...)` closure taking `(best_perm, best_flops,
  indep_deferred, run_subtree, score_workspace, stage4_signal)`.
  * Gate: `shared_basin_fork_band(n, nnz)` = `n >= 6 && n <= 600 && nnz <= 5_000`
    (`SHARED_BASIN_FORK_MAX_N`, `SHARED_BASIN_FORK_MAX_NNZ`); test-only seam `SSI_SHARED_BASIN_FORK=0`.
  * Inside the gate the base suffix runs on a `std::thread::scope` thread with the prefix arena and
    reports over a `sync_channel(1)` whether stage 4 accepted a **strict** improvement. The alternate
    suffix (stage 4 withheld) is started **only** when it did, on a fresh arena. Rows whose subtree
    stage is inert therefore pay exactly one suffix.
  * Merge is a strict `Σcⱼ²` comparison in a fresh workspace; the base wins ties. Output is a
    deterministic function of the pattern and can only improve on the base.
  * Outside the gate the tree runs the plain suffix — the same statements, unforked.
* **Twin shape (`mod.rs`).** `dense_default = (10, 4, 4)` for `1_000 <= n <= 5_205`
  (`DENSE_TWIN_WIDE_MIN_N`/`DENSE_TWIN_WIDE_MAX_N`), else `(8, 4, 3)`.
* **Inherited cleanup (iter65).** The disabled width-band / fork switches left the production path;
  `star_window_descent` is `#[cfg(test)]`.

## Result — production-worker frame

Frame: `probe_graded_frame` stages each pattern and runs a **separately built production worker** per
row (`taskset -c 0-3`, `env_clear`, one process per `order()`), interleaved arm by arm. This is the
frame the 2 s watchdog charges, and it is *not* the in-process probe frame that the 0271–0275 timing
claims were read from. Arms: `frontier` = worker built from the pre-refactor tree; `candidate` =
worker built from this tree.

| run | scope | rows | frontier score | candidate score | wall, frontier → candidate |
|---|---|---|---|---|---|
| A — prefix shared, both suffixes always | fork band + twin | 139 | 0.789742956 | 0.789267494 | 168.28 → 183.49 s (**+9.0 %**) |
| B — + stage-4 decision gate | same | 139 | 0.789742956 | 0.789267494 | 162.33 → 165.47 s (**+1.9 %**) |
| C — final tree, `SSI_GRADED_REPS=2` | fork band + bounded twin | 130 | 0.791879571 | 0.791384236 | 112.05 → 110.61 s |

A vs B is the decision gate's own receipt: **identical scores**, +9.0 % → +1.9 % wall. Subset scores
are not corpus scores (the scope is only the admitted rows), so they are comparable within a run.

Movers in run C (ratio = your flops / AMD flops, lower is better):

| matrix | n | frontier | candidate | Δln |
|---|---|---|---|---|
| `waterund14` | 333 | 0.360048682 | 0.351589154 | −2.3776e-2 |
| `chimera_mgw-c8-439-onc8-001` | 440 | 0.752167094 | 0.734689469 | −2.3511e-2 |
| `chimera_lga-01` | 1120 | 0.740586720 | 0.736724554 | −5.2287e-3 |
| `chimera_mgw-c16-2031-01` | 2032 | 0.772227953 | 0.769289916 | −3.8119e-3 |
| `gancns` | 548 | 0.842057371 | 0.840656325 | −1.6652e-3 |
| `chimera_rfr-02` | 2032 | 0.644961547 | 0.644297374 | −1.0303e-3 |

Every mover is in `lt_1k` (147 rows, weight 0.30) or `1k_10k` (108, 0.30); `gt_10k` is untouched.
Summing the logged per-row Δln by bucket and mapping onto the corpus bucket sizes:

```
Δdev = 0.30·(−0.048951735)/147 + 0.30·(−0.010070852)/108 = −1.279e-4
```

**Attribution.** The three `1k_10k` movers reproduce, to six decimals, the deltas of the twin's own
24-row census at `10/4/4` vs `8/4/3` (`0271-dense-twin-arms.log`); the twin moves nothing else in
that census. So the `1k_10k` gains are the twin's, and the `n = 440` gains are the fork's — with one
exception, which is the whole subject of the next section.

## Why the bounded shape was needed — and what it actually did

In run B the candidate carried **one** regression: `chimera_mgw-c8-439-onc8-002` (n = 440)
`0.881986308 → 0.883457648` (+1.667e-3 ln). It is admitted by both gates
(`nnz = 3 478 ≤ 5 000`, `max_deg = 440 > n/2`).

Run C differs from B by exactly one source edit (`DENSE_TWIN_WIDE_MIN_N = 1_000`) and exactly one
row's candidate ratio: `…-002` returns to `0.881986308` — the frontier value. Every other row,
including `…-001`, is bit-identical between B and C. So the edit removed the regression **and
nothing else**, and the two-`n=440`-row attribution above is measured, not assumed.

Note for the next session: this is the *opposite* of the naive reading. `…-001`'s −2.35e-2 is
fork-driven (the twin is inert on it), while `…-002`'s regression was twin-driven. Gate membership
does not tell you which device moved a row; only the arm census does.

## Cap accounting — and the hidden verdict

Submitted on 2026-09-14 as `7febce96-dd57-4b4b-95e4-0461ff867329` and **killed**:

```
RUN FAILED: hidden matrix: order() exceeded the 2.0s per-matrix cap and was killed
```

PR <https://github.com/Layr-Labs/matrices-fast/pull/707>, workflow run `34825987718`, dispatched
09:05:00Z → failed 09:10:07Z (**≈307 s** of benchmark wall). Receipt:
`../evidence/0276-remote-verdict-pr707.txt`. No score was produced.

| tree | added wall on the rows it gates | hidden receipt |
|---|---|---|
| fork, whole pipeline duplicated (`e4302dab`, `e1e6c7f2`) | ≈ +22 % on its band | **2/2 killed**, at 81.5–184.1 s |
| fork, prefix shared, both suffixes (run A) | +9.0 % aggregate, 139 rows | not submitted |
| fork, prefix shared, stage-4 gated (runs B/C) | +1.9 % (139 rows) / median **+0.2 %** (119 fork rows) | **killed**, at ≈307 s |

Run C per-row wall (min of 2): 119 fork-gated rows median **+0.2 %**, mean +0.6 %, 11 rows above
+10 %, worst `himmel11` 0.944 → 1.623 s. The 6 twin rows ran +3.2 % median. The tail is the risk:
the fork still roughly doubles a cheap row's suffix when stage 4 accepts.

**What the kill does and does not say.** The ≈307 s survival against 81.5–184.1 s for the
whole-pipeline form says the cost reduction was real — this tree got roughly 1.7–3.8× further into
the corpus before crossing the cap. It does **not** say the fork band owns the failing row: the row
is redacted, and a tree whose median gated-row overhead is +0.2 % still died. Nor does it say the
value is wrong — the value was never the binding constraint. It closes the "trim the band" line of
attack: five submissions of this value package (`fe17562d`, `873f23f0`, `e1e6c7f2`, `e4302dab`,
`7febce96`) have now died on the cap, while a **score-neutral** tree from another lane (`6864d7b`,
0.840622) completed on the same day. The cap is currently behaving like a property of the corpus
window, and the next admissible change is one that *removes* deterministic work from the globally
slowest rows, not one that trims a band that already runs cheap.

The close variant `3587d1b` still completed a hidden corpus at **0.840545** (−7.8e-5, sub-bar) with
the *old* duplicated fork; against the current best **0.840623** the bar is ≈1e-4 absolute, so even a
clean completion is not obviously promotable.

## Verification

* `cargo build --release -p matrices-fast --offline --locked` — clean.
* Production candidate worker via `scripts/local-candidate-build.sh` — clean (the local shell cannot
  create bubblewrap namespaces, so the documented `SSI_ALLOW_UNSANDBOXED_WORKER=1` opt-out was used;
  the graded frame is unaffected).
* `cargo test --release -p ssi-candidate-worker --offline --locked` — **126 passed / 0 failed**,
  59 ignored.
* Determinism: run C used `SSI_GRADED_REPS=2`; the probe asserts permutation equality across the two
  worker runs (and bijection) per arm per row — passed on all 130 rows.
* Builds used by the A/B (sha256 of the staged workers; rebuild with `local-candidate-build.sh` and
  compare): frontier `b8d7f484967bcf00a0f8d8a8fdec9218e3cd9079f254e4e60e0fdb6a28201b8d`,
  pre-edit candidate `7507e7880d8f8d1068e2ba15e4dbc71874d68b9fc5431e911d283454feeefdaf`,
  final candidate `f6156ecc3a06d132512101580712edddeaa3cf59688ffbd61c530a7e4f966691`.
* A local `cargo run --release` was attempted and did **not** produce a score: it stopped on
  `rsyn0810m04m` (`n=4772`, `nnz=13836`), which is outside both gates, and the same-window frontier
  control worker fails that row identically (2.949 s vs 2.808 s, ratio `0.767229402` for both). No
  `score.json` was produced and no score is claimed here.

## Follow-ups

- The remaining unknown is hidden cap margin, not value. A cheaper suffix (or a wall-removing device
  on the same class) is what would make this package defensible; another dose change is not.
- The `1k_10k` twin gains are cheap and bounded; if the fork is ever retired, they can ship alone for
  ~2.2e-5.
- Re-price the decision gate's tail (`himmel11`, `syn15hfsg`, `nvs02`) before widening the fork band.

## Links

- Experiments: [0271 basin fork and width band](0271-basin-fork-and-width-band.md),
  [0239 public-board receipts](0239-public-board-receipts.md),
  [0238 shared class engine](0238-shared-class-engine-ceiling-45k.md)
- Techniques: [best-of-portfolio](../techniques/best-of-portfolio.md)
- Evidence: `../evidence/0276-graded-frame-shared-fork.log`,
  `../evidence/0276-graded-frame-decision-gated.log`, `../evidence/0276-graded-frame-reps2.log`
