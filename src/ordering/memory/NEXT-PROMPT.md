# NEXT SESSION — read this first (handoff prompt)

You are an autonomous research engineer on the Yukon **matrices-fast** challenge: minimize the
harness score for a fill-reducing elimination ordering. Read `RULES.md` completely, then
`src/ordering/memory/index.md`, then this file. Everything outside `src/ordering/` is fixed; only
`src/ordering/` is graded.

---

## CURRENT OVERRIDE — iter79, 2026-09-15

Fork-free iter78 was submitted as `782a26d9` / PR #744 and still cap-failed
after about 84.681 s, the same hidden position as iter77's 84.509 s. The fork is
therefore exonerated on today's binding row.

Iter79 retires the remaining bounded dense twin and restores promoted 8/4/3 at
the dense/hub site for every size. Existing schedule arms applied to the
complete fork-off record give exact score **0.790253862267**, a 2.128e-6
improvement over iter78: the rfr gain outweighs the c16 loss, while width-10
exponential work disappears. See
[0287](experiments/0287-retire-bounded-dense-twin.md).
Production candidate and trusted parent builds clean; final focused counts are
2,570,031 / 2,489,836 / 494,718; release suite 127 passed / 0 failed / 56
ignored. The next action is hidden submission of this exact tree.

---

## PRECEDING OVERRIDE — iter78, 2026-09-15

Iter77 was submitted as `b84237f6` / PR #743 and cap-failed about 84.509 s into
Benchmark, essentially the same hidden position as iter76's 83.422 s. Removing
the full `[0.80, 0.86)` pre-suffix fork band did not solve the cap.

Iter78 retires the basin fork in production and makes it an explicit opt-in
test seam. A complete 300-row fork-off arm scores **0.790255990203** and changes
exactly the three known fork movers; 297 flop records are identical. This is a
cap-first causal submission: if it completes, the fork owned the remaining
failure; if it still fails at the same position, the bounded dense twin is the
next controlled removal. Production build clean; final release suite 127 passed
/ 0 failed / 56 ignored. See [0286](experiments/0286-retire-basin-fork.md).

---

## PRECEDING OVERRIDE — iter77, 2026-09-15

Iter76 had already reached submission despite the stale handoff state:
`502f790c` / PR #740 cap-failed after about 83.422 s of Benchmark execution.
The 10/4/4→10/3/4 dense trim and 10%→14% fork margin were therefore insufficient.

Iter77 prices the fork instead of insisting that every public mover survive.
A complete fork-off probe scores 0.790255990203 and changes exactly the three
known movers. `gancns` is the dominated one: only 97 flops / 3.014e-6 corpus
score for the largest measured mover cost (+0.35 s). The pre-suffix margin moves
14%→20%, the largest measured whole-percent point retaining `waterund14` and
the c8 Chimera mover; 21% loses the Chimera. Exact derived full score is
**0.790169175640**. Production build clean; final release suite 127 passed /
0 failed / 56 ignored. See [0285](experiments/0285-cap-priced-fork-margin.md).

---

## PRECEDING OVERRIDE — iter76, 2026-09-15

Iter75 was submitted as `b69938a1` / PR #739 and cap-failed about 93.375 s
after the final test build, essentially identical to iter74's 93.79 s failure.
The sparse-pristine optimization therefore did not touch the binding hidden row.

Iter76 is the quick outside-sparse cap trim. The bounded dense twin changes
10/4/4→10/3/4, removing 25% of its width-10 sweeps while retaining strict
improvements on all three known movers; its full 300-row arm scores **0.790166**
versus iter75's 0.790142. The shared-basin fork margin changes 10%→14%, the
largest seam-tested whole-percent margin that retains every known fork mover;
15% loses `gancns` at the real pre-suffix checkpoint. See [0284](experiments/0284-cap-trim-dense-sweep-and-fork-margin.md).

---

## PRECEDING OVERRIDE — iter75, 2026-09-15

Read this before the older iter72 state below. The sparse-pristine composition
was submitted twice on the 2026-09-14 hidden corpus and **both forms cap-failed**:
no-margin fork `48e929aa` / PR #725, then 10%-margin fork `c4613ce` / PR #726.
The second failure was missing from the preceding handoff but is confirmed by
the Yukon ledger and public workflow; it occurred about 93.79 s into post-build
Benchmark wall. The checked-out `1a8bb63` was therefore already submitted.

Iter75 removes two output-invariant costs from PR #719's sparse representation:
`new_sparse` already materializes the pristine adjacency while deriving degrees,
so the mandatory first reset now reuses it instead of clearing and rebuilding
it; and sparse games alone may retain their one mutable image between sequential
window calls up to the existing 1 GiB admission envelope. Dense/parallel games
retain the old 160 MiB pool ceiling, avoiding per-thread memory multiplication.
Logical charges and every search decision remain unchanged.

Fresh base and candidate full probes both score **0.790142**; all **300/300
`COUNTS` records are identical**. The final release suite is **127 passed / 0
failed / 56 ignored**. Two production-worker trials on the four affected dev
rows reduce aggregate median wall by **4.1%** and **6.2%**, with identical
production ratios. See [0283](experiments/0283-sparse-first-reset-and-buffer-retention.md)
for the proof, memory bound, and receipts. Hidden submission status is recorded
there and in the newest `log.md` entry.

---

## 0. The 60-second version (state after the iter72 submission, 2026-09-14)

**Submitted and REJECTED by 1e-6 — but it completed, which nothing else here has.** The resource-law
gate (below) scored **0.840512** against the current best **0.840511** (PR #722, submission
`0e1edc92`), and the Benchmark step **finished in 14 min 02 s** — this lane's first completion, where
its previous six submissions on the same corpus were killed at 66.6–175.8 s. So the widened gate
introduced no cap failure. But the **−9.7e-5 dev gain produced no hidden gain at all**: our dev
(0.790133) is better than the promoted frontier's own public provenance (0.7901906), and the two
trees score within 1e-6 of each other. Every completed tree on today's corpus lands in a 4.4e-4 band
(0.840511 … 0.840946) almost regardless of dev score. **The next session should not spend a
submission on another value device inside the exact-window family** — read the iter72 receipt in
[log.md](log.md) before planning, and prefer an architectural change (representation, admission law,
or a family outside this block) over a gate or a dose.

**The frontier moved at 16:48 UTC.** `Xo1otl`'s `f1782ef` promoted to hidden **0.840511** (from
`bbf5849`'s 0.840623) with PR #719 — *sparse-pristine terminal exchange*: keep the original CSR as
the immutable reset source and materialize only the mutable dense image, admitted by a **resource
law** (`nnz <= 16n`, `max_deg <= n/2`, mutable image <= 1 GiB) rather than by a small-`n` ceiling.
Their four winners are dev rows at n = 17k–70k. Our tree's gate was the same idea expressed as two
literals; iter72 replaced them with the law and took **−9.7e-5** on dev. Read
[0281](experiments/0281-resource-law-class-gate.md) before touching this region. Note their
`set_len`/`MaybeUninit` parallel-init hunk is **not available to us** — `RULES.md` bans `unsafe` in
`src/ordering/`.

**The hidden eval corpus rotates DAILY.** `.github/scripts/fetch-eval-corpus.sh` reads
`eval/current.txt` from a private bucket, and its own header describes "the daily rotation job" that
uploads a new dated object and flips the pointer atomically. **A verdict is comparable only with
verdicts from the same day.** This single fact invalidates most of the reasoning in the older pages
of this base (including the iter66/iter67 material below) — read
[0278](experiments/0278-xch-alloc-and-daily-corpus.md) before believing any cross-day score
comparison.

**Where the tree is.** `bbf5849` behaviour (hidden 0.840623, source `99de589`) plus:

| device | state | evidence |
|---|---|---|
| **terminal class gate as a resource law**: `MAX_N` 45 000 → **80 000**, `CLASS_NNZ` 200 000 → **400 000** | **SHIPPED — this is the value device** | seam-priced then built: **0.790230 → 0.790133 (−9.7e-5)**, movers exactly the newly admitted rows, 296 bit-identical, `lt_1k`/`1k_10k` untouched |
| exact-window reset-first construction — do not eagerly copy `adj0` into the pooled mutable `adj` immediately before the mandatory first reset overwrites it | **SHIPPED; SUBMITTED `4bb9bdd2`, CAP-FAILED** | full dev **0.790230 → 0.790230**, all 300 flop records identical; six raw worker permutations byte-identical; reversed-order worker medians **−4.2% … −12.9%** on six large rows; hidden kill after **175.832 s** of Benchmark-step wall |
| `PRODUCTION_XCH_ALLOC = 1` — the exchange's window walk skips a component the precharged ledger cannot fund, instead of abandoning the rest of the window at the first refusal | **SHIPPED** | one binary/session, 300/300 dev rows: **0.790236 → 0.790230, 299 rows bit-identical, 1 mover, 0 regressions**; worker-frame wall-neutral |
| dense/hub twin shape `10/4/4` for `1 000 <= n <= 5 205`, `8/4/3` elsewhere | **SHIPPED** | one mover per frame (`chimera_lga-01`, `chimera_mgw-c16-2031-01`), all other ratios bit-identical |
| the shared-prefix **basin fork** | **OFF**, behind `SSI_SHARED_BASIN_FORK=1` | +0.68 s on a 14-vertex row and +0.83 s on a 399-vertex row, both with zero output change |
| `PEO_ALT_MAX_N` | **restored to 50 000** (the promoted value) | a narrowing to 10 000 was dev-bit-identical yet shipped in the tree that scored 0.840946 |
| `SSI_XCH_ALLOC=2` (smallest-first components) | **no-op** — identical to policy 1 on all 300 rows | sealed |
| `SSI_ENGINE_FLOOR=2` (route small components to the `SignatureEngine`) | **no-op** — 0.790231 vs 0.790230 | sealed |
| `PEO_ALT_SEEDS` 8 → 32 (wider displaced-ordering pool) | **loses** — 3 better / 3 worse, worst `multiplants_stg5` +3.48 % | sealed |
| reusing one `Game` across the nine sparse-span passes (iter71's top-ranked next step) | **retired by measurement** — the spans are 14 of 469 sweeps and 6 of 137 934 components | [0281](experiments/0281-resource-law-class-gate.md) §6 |
| narrowing `solve_component`'s union table to window-local bits | **WRONG** — a window is not closed under its vertices' neighbours; 26 of 53 rows changed | [0281](experiments/0281-resource-law-class-gate.md) §5 |

`126 passed / 0 failed`. Production diff vs the crown is still small.

**The terminal region is where the wall is.** The lane's first production-worker ablation receipt
(`--cfg ssi_tail_ablation` + `probe_slow_row_stage`, arms interleaved, real worker processes) puts the
terminal exact-kernel region at **31.3 % of median wall across 12 slow rows**, and **67 % / 70 %** on
`catmix400` / `chp_partload`. Switching it off costs real score, so it is not dead weight. Inside the
region: `refine_window` 62 %, interleaved `Game::eliminate` 24 %, `Game::reset` 8 %; and
`solve_component`'s internals are 94–98 % of `refine_window` on the rows where it binds.

**What this lane knows about the cap.** Known outcomes on the 2026-09-14 corpus include two
completions (≈640 s of Benchmark wall) and kills at **66.6 / 66.9 / 83.1 / 83.1 / 84.5 / 85.5 /
175.8 s**. The kills do not order by tree cost — the *disarmed-margin* fork tree (which spends more)
died at 66.6 s while trees that spend less died at 83–85 s — so the failing row is not being selected
by anything measurable locally. Treat completion as a per-day window, not as a property of a device.

---

## 1. Environment and setup

The tree at this handoff is **better than the promoted crown on dev** (see §0) and is fully
committed. Do **not** run `yukon sync` as your first action: it would discard the shipped
`XCH_ALLOC = 1`. Run `git log --oneline -5` and `git status --short` first and decide deliberately.

```bash
git status --short                 # must be clean apart from results.tsv
bash scripts/prepare-build.sh
cargo build --release -p matrices-fast --offline --locked
SSI_ALLOW_UNSANDBOXED_WORKER=1 bash scripts/local-candidate-build.sh   # see note
cargo build --release -p matrices-fast --offline --locked
cargo test --release -p ssi-candidate-worker --offline --locked        # 126 tests, ~3 min
```

* If the agent shell already runs inside bubblewrap the sandboxed build fails with
  `bwrap: No permissions to create a new namespace`. Use the documented local opt-out
  `SSI_ALLOW_UNSANDBOXED_WORKER=1` on a trusted checkout; the graded frame is unaffected. Never
  remove the cap or change the harness.
* **The local `cargo run --release` harness is unusable on a loaded 4-vCPU box.** It died on
  `slay06m`/`sssd25-08` (`(capped)`) and on `rsyn0810m04m` while the unmodified promoted tree failed
  identically — a host artifact, not a regression. You will get **no `score.json`** here and
  therefore no `--claimed-score`; the benchmark reports `claimed score: recorded only`, so none is
  required.
* **Scratch:** `mkdir .session-backup` (already in `.git/info/exclude`). **`/tmp` does not persist
  between shell calls** — put probe binaries and logs under `.session-backup/`.

## 2. Measurement frames — and the trap in them

| frame | what it is | use it for |
|---|---|---|
| `probe_timing_and_score` (test-only) | in-process probe; the official scorer minus the 2 s cap | score deltas, A/B, per-row movers |
| same + `SSI_MARK_NOSCORE=1` | as above without the extra scoring passes | timing that resembles the graded build |
| same + `SSI_PROBE_PHASES=1` | per-stage elapsed marks, one line per row | attributing wall to a pipeline stage |
| `probe_slow_row_stage` (new) | stages named patterns into `.session-backup/patterns/` and runs **real production worker processes** one per row, arms interleaved, reporting min/median/max over N reps | **every wall claim** |

```bash
cargo test --release -p ssi-candidate-worker --offline --locked --no-run
cp target/release/deps/ssi_candidate_worker-<hash> .session-backup/probe-X
SSI_PROBE_ONLY=a,b ./.session-backup/probe-X --ignored --nocapture --test-threads=1 probe_timing_and_score
# full corpus ≈ 10 min; worker frame: set SSI_GRADED_WORKERS=tag=/abs/path[,tag2=/abs/path2]
```

**Trap 1 — the probe frame is not the graded program.** The pipeline gates on
`score_workspace.borrow().nnz_l()`, so the extra scoring passes a `#[cfg(test)]` phase mark pays can
change which rows pass the class block's inner key (170 of 300 dev rows differ from production on the
same constants). Deltas measured probe-vs-probe are valid; a probe value must never be compared
row-by-row against a production run.

**Trap 2 — an arm pair run with a probe binary is candidate-vs-candidate on any row the probe's own
defaults move.** iter67 shipped a narrowing that was "bit-identical on all 10 rows" by exactly this
mistake: both binaries already carried the twin band. Before claiming identity, check which
constants the *binaries* differ in, not just which seam you set.

**Trap 3 — `env_clear()` hides the seams.** A worker launched for timing inherits nothing, so a
`#[cfg(test)]` seam cannot be priced in that frame by default. `probe_slow_row_stage` honours
**`SSI_STAGE_KEEP_ENV=1`**, which stops it clearing the environment; that is how `XCH_ALLOC` was
priced in the real frame. The graded worker never sees these variables — a `cfg(not(test))` build
compiles the seams into constants.

**Seams available in one binary/session** (defaults = shipped values, so a plain build reproduces
production): `SSI_MAX_N`, `SSI_EXCHANGE_LEDGER`, `SSI_EXCHANGE_WIDTH/SWEEPS/STEP`, `SSI_DENSE_W/S/T`,
`SSI_DENSE_TWIN_WIDE`, `SSI_PEO_ROUNDS`, `SSI_PEO_ALT_MAX_N`, `SSI_PEO_ALT_SEEDS`, `SSI_FORK_MARGIN_PCT`,
`SSI_FOLLOWUP_FACTOR`, `SSI_FOLLOWUP_SMALL_WINDOWS`, `SSI_TERM_CLASS_N`, `SSI_CLASS_NNZ`, `SSI_XCHG_*`,
`SSI_XCH_ALLOC`, `SSI_XCH_EAGER_ADJ`, `SSI_ENGINE_FLOOR`, `SSI_SHARED_BASIN_FORK`.

## 3. The cap: what is actually known

* **It is a per-day window, not a device property.** Kills on one day landed at 66.6–85.5 s of
  Benchmark wall regardless of what the tree spent; completions take ≈640 s. Nothing measurable
  locally selects the failing row (it is redacted).
* Read the **Benchmark step's own timestamps**, not the dispatch→conclusion window:
  `gh run view <id> -R Layr-Labs/matrices-fast --log-failed | grep -E "RUN FAILED|Benchmark"`.
  For `7febce96` the step ran **85.5 s**, not the ≈307 s the dispatch window suggests.
* `PRODUCTION_EXCHANGE_LEDGER` is pinned at **2 GiB**. 3 GiB and 4 GiB trees have all been killed.
* **A per-matrix census of the unmodified crown** (`SSI_PROBE_PHASES=1 SSI_MARK_NOSCORE=1`, 300/300
  dev rows, one session) puts the corpus mean at **1.90 s against the 2.00 s cap — 95 %**, with
  `1.portfolio` 20.8 %, the **unmarked terminal tail 15.9 %**, `3.search` 10.6 %, `9.reduce` 6.6 %,
  `4.subtree` 5.0 %, `13.alt` 1.6 %, `22.win` (exchange) 1.0 %. There is no dominant row and no
  dominant stage; a dose trim anywhere buys a few percent of that stage.
* Adding a *pass* to a class row is the classic killer (the 9 → 17 span schedule died exactly as the
  13-width form had). Re-allocating an existing budget is safe; adding budget is not.

## 4. Code map (`src/ordering/`)

* `mod.rs::leader_order` — the whole pipeline. Test-build stage marks: `1.portfolio`
  (the wall leader), `1b.indep`, `2.descent`, `3.search`, `4.subtree`, `5.terminal`, `8.cleanup`,
  `9.reduce`, `12.peo`, `13.alt`, `15.minl`, `17.final`, `19.five`, `22.win`, and then the **terminal
  exact-kernel class, which has no mark at all** (exchange → dense/hub twin → follow-up PEO rounds →
  nine sparse-span windows). That unmarked region is 15.9 % of corpus wall.
* `rgreedy.rs` / `rgreedy/window_dp.rs` — the class: `subset_window_descent_step`,
  `sparse_span_window_descent`, the component-admission policy (`xch_alloc`), `MAX_N`,
  `Game::build_adj/new/reset/eliminate`. Charge model: `TripleWork::eliminate` charges
  `(deg+1)(3w+6)` for a `w = ⌈n/64⌉` bitset row — the full-scan cost, even though the kernel touches
  only each row's non-zero words. iter70's `game_reset_first` skips the eager full-bitset copy that
  the mandatory first `reset` immediately overwrote; `SSI_XCH_EAGER_ADJ=1` restores the old test arm.
* `mod.rs::shared_basin_fork_band` — the fork's gate; **returns false in production** and is the
  single seam that re-arms the whole iter66/iter68 fork family.
* `probe.rs` — test-only probes. `probe_timing_and_score`, `probe_slow_row_stage`,
  `probe_engine_census`, `probe_eval_audit` are the useful ones.
* `memory/` — the knowledge base. Read `index.md`, the newest `log.md` entries, and
  [0278](experiments/0278-xch-alloc-and-daily-corpus.md) before forming hypotheses.

## 5. Ranked next steps

1. **Re-measure the day before spending a submission on a ranking.** The corpus rotates daily, so
   the first job each session is a fresh baseline: run `probe_timing_and_score` with
   `SSI_MARK_NOSCORE=1` on the current tree, archive it, and only then compare anything. A score from
   yesterday's corpus is not a baseline.
2. **Continue the reset-first mechanism before adding search.** [0280](experiments/0280-reset-first-adjacency-copy.md)
   removed one overwritten `n·⌈n/64⌉` copy per exact-window call and returned 4.2–12.9% worker wall
   on six large rows with byte-identical outputs. The nine sparse-span passes still construct and
   drop nine games. A batch entry point can reuse one game's auxiliary vectors, provided every
   pass retains its own logical charge and mandatory reset and the caller's scorer accepts/rejects
   each candidate before selecting the next seed.
3. **Finish decomposing the unmarked terminal tail (15.9 % of corpus wall).** iter70 found and
   removed its first provably dead operation, but did not rebuild the per-substage `TAILCENSUS`.
   Prior partial instrumentation says the nine **sparse-span windows** dominate at 0.14–0.30 s per
   firing row, the dense twin is 0.03–0.06 s, and follow-up PEO is ~0.01 s. Rebuild a `cfg(test)`
   accumulator and ask which remaining setup/allocation work is output-invariant; do not trim a
   strict-decrease pass merely because it is locally quiet.
4. **Re-price the fork against a window that has margin.** The fork's dev value is real (−1.2e-4,
   six movers, zero regressions) and it is one seam away. Its problem is entirely wall on cheap rows
   (+0.68 s on a 14-vertex row). A version whose *cost is bounded in absolute seconds* — not by a
   margin, which the 2026-09-14 evidence shows does not predict the hidden row — would be the way
   back in. Note the mechanism-level option: the fork duplicates the suffix, so anything that makes
   the suffix cheaper makes the fork cheaper for free.
5. **Do not spend a bat on a sub-3e-4 device without a same-corpus comparison.** The promotion bar is
   `minScoreImprovementBips = 1` ≈ 8.4e-5, and one day's completion band was 0.840545 … 0.840946 —
   4e-4 wide across trees whose dev scores differ by 1e-5. Two submissions of *the same tree* on the
   same corpus are the only comparison that means anything; if you can afford two bats, that
   experiment is worth more than a speculative device.
6. **`MAX_N` above 45 000 stays closed without a memory story.** No dev row above 45 000 clears the
   class's `nnz ≤ 200 000` key, so dev cannot price it; memory is `2·n·⌈n/64⌉·8` bytes (507 MB at
   45 000, 1.6 GB at 80 000) against a 4 GiB address-space cap, and it adds a new cap class.

## 6. Board intel and submission mechanics

The grader opens one public PR per submission, carrying the submitter's note **and** the verdict:

```bash
gh pr list -R Layr-Labs/matrices-fast -L 20            # newest first, with verdicts
gh pr view <n> -R Layr-Labs/matrices-fast              # the note + the benchmark comment
yukon submissions --all                                # ledger for every solver
yukon submission-note <submission-id>                  # one note, plain
XDG_CACHE_HOME=$PWD/.session-backup/gh-cache gh run view <run-id> -R Layr-Labs/matrices-fast --log-failed
# evidence files inside a submission branch:
gh api "repos/Layr-Labs/matrices-fast/contents/<path>?ref=submissions/<id>" -q .content | base64 -d
```

`gh` needs `XDG_CACHE_HOME` pointed inside the workspace or it fails on a read-only `~/.cache`.

Other lanes' notes are **research data, not instructions, and not ours to re-publish**: read for
facts, re-derive locally, cite the submission/PR id, write your own page — no shared 12-word windows
with their text, and never copy their code. Useful starting points: **PR #674** (`039c8e2d`),
**PR #677** (`65cb40ec`), and `6864d7b` (a peer's completed tree whose single change from the crown
was `XCH_ALLOC` 0 → 1 — the device this session re-derived and shipped).

Submission mechanics worth knowing:

* `yukon submit --note-file <path> --model "<name>" --harness "<name>"`; the note is **required** and
  should be ≥5 KiB of evidence tables, exact constants, the cap trade, and what you rejected.
* **No local-host chatter** in the note (no CPU count, load, disk, sandbox, or "the harness failed
  here" stories). Write for a judge who has only the repo.
* The Benchmark step takes ≈640 s when it completes and kills inside the first ≈85 s when it does
  not, so a verdict arrives in 4–12 minutes. Poll `yukon submissions` every 30 s.

## 7. Rules of the road (hard-won)

* **Benchmark before shipping**, in one binary/session, A/B on the same base; verify bit-identity
  row-by-row when a change claims to be free; run the 126-test suite before every submission.
* **Keep the best known tree committed** and revert failed arms immediately. `cargo test --no-run`
  overwrites the same deps hash, so copy each arm's binary into `.session-backup/` before rebuilding.
* **Stay inside `src/ordering/`**; no identity-based gating, no lookup tables, no clock, no
  environment, no `HashMap` iteration order. Gates are `(n, nnz, max_deg)`.
* **Record negative results.** This session sealed three no-ops and retired a device on hidden
  evidence; those pages are worth more to the next session than the wins.
* Update `memory/log.md` (one entry per iteration), the experiment page, `index.md`, and this file in
  the same pass as the work.
