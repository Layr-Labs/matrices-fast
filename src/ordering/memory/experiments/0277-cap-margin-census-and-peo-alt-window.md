# 0267 — the cap, priced per stage: a full-corpus wall census and the `13.alt` window

- **Date:** 2026-09-14 (iter67)
- **Base:** the synced crown `bbf5849` / source `99de589` — `MAX_N` 45 000,
  `PRODUCTION_EXCHANGE_LEDGER` 2 GiB, the shared `Pristine`/`Game` memo, `ADJ_POOL`,
  **12** class-block sweeps, `PRODUCTION_SPAN_WINDOWS` at 9 widths.
- **Status:** census measured and reproducible; one wall device (`13.alt` scope) priced here;
  submission decision deferred to the session's final entry.
- **Instruments added (test-only, never in the graded worker):** `probe_slow_row_stage` in
  `probe.rs` — it stages the named patterns into `.session-backup/patterns/` for hand runs and
  reports per-row **min / median / max of N real production-worker runs** in one interleaved
  pass, plus the ratio the worker's own permutation scores. Seams `SSI_PEO_ALT_SEEDS` and
  `SSI_PEO_ALT_MAX_N` were added to `mod.rs` (`cfg(test)` only; a `cfg(not(test))` build
  compiles the shipped constants).

## Why this page exists

Five submissions of this lane's value package died on the 2 s per-matrix cap
(`fe17562d`, `873f23f0`, `e1e6c7f2`, `e4302dab`, `7febce96`), and the last one died **85.5 s into
the Benchmark step** while a peer submission from the same corpus window completed a whole
corpus. No local frame had ever been used to ask *where the seconds actually are*. This page is
that measurement, on the crown itself, with the shipped constants.

## 1. Full-corpus phase census (300/300 dev rows, one binary, one session)

Frame: `probe_timing_and_score` with `SSI_PROBE_PHASES=1 SSI_MARK_NOSCORE=1` — the graded-closest
probe frame (no extra scoring passes inside the marks). Cumulative wall across the 300 rows:
**570.9 s**, i.e. **1.90 s mean per row**, against a 2.00 s per-matrix cap. That ratio is the
whole story: the tree is not dominated by a handful of pathological rows. It is uniformly close to
the cap, so no single stage and no single row can be trimmed to buy margin.

The marks are **per-phase elapsed** (`_tph` is re-armed after every mark), so the column below is
directly the stage's own wall.

| stage | wall over 300 rows | share | rows firing |
|---|---|---|---|
| `1.portfolio` | 119.0 s | 20.8 % | 219 |
| unmarked tail (terminal exact-kernel class + return) | 90.8 s | 15.9 % | — |
| `3.search` | 60.5 s | 10.6 % | 219 |
| `9.reduce` | 37.6 s | 6.6 % | 219 |
| `4.subtree` | 28.7 s | 5.0 % | 219 |
| `1b.indep` | 17.0 s | 3.0 % | 219 |
| `16.late` | 10.1 s | 1.8 % | 219 |
| `13.alt` (8.98 s; its `13p.*` decomposition is listed separately and overlaps) | 9.0 s | 1.6 % | 219 |
| `15.minl` | 8.2 s | 1.4 % | 219 |
| `22.win` (class-block exchange, before the tail) | 5.5 s | 1.0 % | 219 |
| `17.final` | 5.4 s | 0.9 % | 219 |
| `20.lt1k` | 4.6 s | 0.8 % | 219 |
| every remaining mark (`2.descent`, `5.terminal`, `6.extra`, `8.cleanup`, `10.completion`, `11.corecand`, `12.peo`, `14.transplant`, `18.simp`, `19.five`, `21.comp`, `15.peel`, `4b.indep-accept`, `7.telos`) | 18.9 s | 3.3 % | 219 |

Two facts worth carrying forward:

1. **The unmarked tail is the second-largest single item** (15.9 %) and the marks before it are
   only 5.5 s, so the cost lives in the terminal class block *after* `22.win` — the follow-up PEO
   rounds, the dense/hub twin and the nine sparse-span windows — not in the exchange itself.
2. **Nothing is concentrated.** The largest `1.portfolio` on any row is 2.40 s (`ringpack_30_2`)
   and the largest `3.search` is 1.11 s (`netmod_kar1`); 164 rows spend > 0.2 s in `3.search` and
   167 rows spend > 0.2 s in `1.portfolio`. A budget cut at any one site is a few percent of that
   site, and a few percent of 20 % is not margin.

## 2. Real-worker timing of the slowest 28 rows

`probe_slow_row_stage` with `SSI_GRADED_WORKERS=frontier=…`, `SSI_STAGE_REPS=3`, `taskset -c 0-3`,
`env_clear`, one process per `order()`. The staged patterns are in `.session-backup/patterns/`;
the receipt is `.session-backup/iter67-wall.log`.

This is the frame the 2 s watchdog actually charges, and it is **~1.6–3× slower than the
in-process probe on the same rows and the same host** — the probe reuses one process and one warm
allocator across rows, the worker pays process start, a cold allocator and its own staging. Both
frames are load-sensitive here (load average ≈ 6 on a 4-vCPU box), so only *within-frame,
within-session* comparisons are admissible.

Worst rows, min of 3 real worker runs (seconds):

| matrix | n | nnz | min | median | max |
|---|---|---|---|---|---|
| `arki0016` | 4 432 | 11 248 | 4.52 | 4.59 | 4.64 |
| `arki0013` | 44 909 | 160 172 | 4.23 | 4.34 | 4.41 |
| `crudeoil_lee4_10` | 17 809 | 120 632 | 4.21 | 4.35 | 4.36 |
| `crudeoil_lee4_09` | 15 904 | 101 792 | 4.21 | 4.52 | 4.96 |
| `mpbp_48` | 28 368 | 91 016 | 4.19 | 4.20 | 4.62 |
| `chimera_selby-c16-02` | 2 031 | 10 878 | 4.04 | 4.12 | 4.31 |
| `ringpack_30_2` | 17 999 | 121 458 | 4.04 | 4.07 | 4.55 |
| `transswitch0300p` | 11 659 | 48 446 | 3.74 | 3.76 | 4.00 |
| `powerflow0300p` | 11 251 | 41 918 | 3.47 | 3.61 | 4.09 |
| `nuclear10a` | 17 493 | 163 816 | 3.42 | 3.47 | 3.49 |

**Consequence for method.** A wall claim from the in-process probe is not a wall claim in the
graded frame, and neither frame transfers to the grader's host absolutely. What *does* transfer is
a same-session, same-frame **A/B** on the same rows — which is how the next device must be priced.

## 3. The `13.alt` PEO chain's scope: a priced wall device

`13.alt` is the one stage whose **scope**, rather than its dose, is still on the historical
record's wrong side. `PEO_ALT_MAX_N` is 50 000 here; the comment block beside it records that
**every build which ever completed this benchmark confined the chain to `n ≤ 10 000`** (the
survivor `71c2c5fe`, explicitly "chain *absent* above n = 10 000"), while the receipt that
isolated the ladder window to `12 000 < n ≤ 50 000` (`9fa0b9c1`) died 102.2 s into the Benchmark
step — and that the chain has **exactly zero dev yield on every row with n > 10 000**, measured as
"263 drawn rows against 238, not one flop moved".

Measured here, in the census frame, on the 38 dev rows with `10 000 ≤ n ≤ 50 000` that the chain's
gate admits (`n + nnz < PEO_ALT_LEDGER = 4e6`):

```
sum(13.alt) over those 38 rows = 8.11 s
worst:  methanol400 0.335 s | crudeoil_pooling_dt3 0.331 | methanol200 0.324
        gasprod_sarawak81 0.320 | transswitch0300p 0.286 | chp_shorttermplan2d 0.282
```

**Output check.** Three frames agree that the narrowing changes no flop count on the rows it
touches:

* probe frame, 10-row arm pair (`SSI_PEO_ALT_MAX_N` 50 000 vs 10 000) on the six heaviest band rows
  plus four small controls — 10/10 identical counters
  (`../evidence/0277-peo-alt-band-{wide,narrow}.log`);
* **worker frame, 21-row A/B of the two real binaries** (crown vs this candidate,
  `../evidence/0277-final-ab.log`): 20 of 21 rows bit-identical, the single mover being
  `chimera_lga-01`, which is the *twin*'s row (n = 1 120 is inside the twin band) and not an
  `13.alt` row at all. The three band rows in that A/B (`transswitch0300p`, `pinene200`,
  `chp_shorttermplan2d`) plus `crudeoil_pooling_dt3`/`methanol400`/`mbp_15` in the probe frame are
  the direct evidence;
* the constant's own record: the chain has never moved a flop above n = 10 000 on this corpus.

**Caveat recorded honestly.** The probe-frame arm pair above was run with *both* binaries carrying
the twin band, so on a twin-eligible row it compares candidate-with-candidate; it is valid for the
`13.alt` question only because every row in it except `chimera_lga-01` is twin-ineligible, and
`chimera_lga-01` is excluded from the claim. The worker-frame A/B is the frame that settles it.

**Shipped as `PEO_ALT_MAX_N = 10_000` — then REVERTED.** See §4c: that tree completed the hidden
corpus at **0.840946 (+3.23e-4 vs the crown)**, so the narrowed window is the prime suspect for a
hidden loss an order of magnitude larger than any local frame could see, and iter68b restored the
promoted 50 000. The dev measurements on this page stand; the *shipping decision* they supported
does not — "zero dev yield above n = 10 000" is exactly the signature of a device whose value lives
on rows the dev corpus does not contain.

## 3b. The displaced-ordering pool: a wall-free value device that LOSES — closed

`flush_batch` retains the best `PEO_ALT_SEEDS = 8` displaced orderings, and that pool is the sole
input of three consumers (the `13.alt` seeds, the terminal transplant donors, the disabled
terminal-exchange pool). None of them can sponsor a new candidate evaluation, so widening the pool
is **wall-free by construction** — the obvious place to look for value that costs nothing. It does
not pay. Same binary/session, 20 rows, `SSI_PEO_ALT_SEEDS` 8 vs 32
(`../evidence/0277-peo-alt-pool{8,32}.log`): 14 rows identical, 3 better, **3 worse**, and the
losses are the large ones:

| row | pool 8 | pool 32 | delta |
|---|---|---|---|
| `multiplants_stg5` | 0.412188 | 0.426526 | **+3.48 %** |
| `chimera_mgw-c8-439-onc8-002` | 0.881986 | 0.884530 | +0.29 % |
| `mpbp_15` | 0.777151 | 0.777371 | +0.03 % |
| `chp_shorttermplan2d` | 0.502534 | 0.502214 | -0.06 % |
| `transswitch0300p` | 0.922396 | 0.922265 | -0.02 % |
| `chimera_mgw-c8-439-onc8-001` | 0.752167 | 0.752111 | -0.007 % |

The three gains are worth ~6e-5 of log-sum between them; `multiplants_stg5` alone gives back more.
**Closed: the pool is not starved at 8** — extra seeds displace *better* seeds inside the `13.alt`
ledger and inside the transplant donor set, and on a small row the displacing ones are worse. This
is the same "additive is not additive" trap the peer's registration note found from the other
direction (their one-slot widen was worth -1.9e-5 *on their tree*; at 8 -> 32 here it is negative).
Pool changes are tree-specific and must be measured, never inferred.

## 3c. The bounded dense-twin shape: wall-neutral, shipped

The dense/hub twin still ran the `8/4/3` shape the class block outgrew. `10/4/4` was measured in
iter60-64 at **-2.21e-5 dev** on the twin's own 24-row census, and that census stops at `n = 5 205`
while the twin's gate (`nnz > 16n || max_deg > n/2`) reaches every `n <= 45 000` — which is how the
unbounded form produced the `n = 440` regression that retired it. It now ships **bounded to its own
census**: `10/4/4` on `1 000 <= n <= 5 205`, `8/4/3` elsewhere.

Wall check, worker frame, same session, **5 reps per arm per row**, the six dev rows the band
touches (`../evidence/0277-twin-ab-5reps.log`), min of 5:

| row | crown | candidate | ratio (crown -> candidate) |
|---|---|---|---|
| `pooling_sppa9tp` | 3.069 | 3.113 | 0.161338075 |
| `chimera_lga-01` | 2.662 | 2.606 | 0.740586720 -> **0.736724554** |
| `pooling_sppa9pq` | 1.813 | 1.800 | 0.634036313 |
| `meanvar-orl400_05_e_8` | 1.051 | 1.015 | 0.999963372 |
| `meanvar-orl400_05_e_7` | 0.897 | 0.906 | 0.999835775 |
| `watercontamination0303r` | 0.510 | 0.524 | 1.000000000 |

The single mover is the one the 0271 census predicted (`chimera_lga-01`), the row ratios are
bit-identical everywhere else, and the wall difference is inside the noise (the same binary's own
5-run spread on `chimera_lga-01` is 2.66-3.18 s).

**The twin band's dev value on this tree, measured two ways.** In the worker frame the mover is
worth **dln = -5.229e-3** on `chimera_lga-01`; in the probe frame an explicit arm pair
(`SSI_DENSE_TWIN_WIDE` 0 vs 1) over the eight census rows returns **one mover,
`chimera_mgw-c16-2031-01` 0.772117078 -> 0.769289916 (dln = -3.668e-3)**, every other row
identical — note the two frames move *different* rows at n = 1 120 / 2 032 because the probe frame's
extra scoring passes change which row the twin's `followup_factor` key admits (the 0218/0239 frame
trap). Neither frame saw a second mover, so the honest reading of the iter60-64 `-2.21e-5` census
figure on *this* tree is "a handful of 1k_10k rows, order 1e-5"; the direction is not in doubt
(every admission is a strict exact decrease) but the magnitude is one order below the promotion
bar.

## 4. Board receipts read this session (facts only)

- The grader's PR comment is the only verdict channel, and the **run log** carries the part the
  PR comment omits. `gh run view <id> -R Layr-Labs/matrices-fast --log-failed` on this lane's own
  `7febce96` (run `34825987718`) timestamps the failure: the Benchmark step began
  09:08:29.06Z and the cap failure printed 09:09:54.52Z — **85.5 s**, not the 307 s that the
  dispatch→conclusion window suggests. Use the step timestamps.
- `yukon submissions --all` is the cheapest kill/completion timeline: this lane's five kills and
  the peer completions `3587d1b` (0.840545, rejected), `0df9f50` (0.840900, rejected) and
  `6864d7b` (0.840622, rejected) all sit inside 2026-09-13 17:52 → 2026-09-14 02:17, and the
  window that followed contains only kills.
- A peer note (`4c4f7b71`) records the one **controlled pair** on the sweep axis with the rest of
  the tree held fixed: `f02eb0d7` (12 sweeps) killed at 81.5 s vs `0df9f508` (6 sweeps)
  completed at 0.840900 (+2.77e-4 vs the crown). Treat it as data: it says a 12→6 sweep cut is
  what converted a kill into a completion on that tree, at a measured hidden price.

## 4b. The verdict channel is a draw, and the plateau is wider than the promotion bar

The full ledger (`yukon submissions --all`) puts the completion window in context. From
2026-09-13 17:52 to 2026-09-14 04:16 — a 10.4-hour window — **every** submission that produced a
number landed in `0.840545 … 0.840900`:

| submission | solver | score | vs the promoted 0.840623 |
|---|---|---|---|
| `3587d1b` | newjordan | 0.840545 | −7.8e-5 |
| `6864d7b` | gunboatsss | 0.840622 | −1e-6 |
| `bbf5849` (promoted) | newjordan | 0.840623 | — |
| `039c8e2` | newjordan | 0.840725 | +1.0e-4 |
| `f9b2fe4` | this lane, earlier | 0.840741 | +1.2e-4 |
| `0df9f50` | newjordan | 0.840900 | +2.8e-4 |

Three facts follow, and they change how a result must be read:

1. **The whole field is inside a 3.6e-4 band**, while the promotion bar is
   `minScoreImprovementBips = 1` ≈ **8.4e-5**. The bar is a quarter of the observed spread of
   independent submissions that were all trying to improve the same tree.
2. **No completion has ever been repeated on the same tree in that window**, so no
   single-completion comparison — including the peer's 12-vs-6-sweep pair — can separate a
   device's price from the corpus draw. Earlier in the round (2026-09-02/03) the completions were
   monotone and widely spaced, i.e. they were quality differences; at this plateau they are not.
3. Every one of the 5 submissions after 04:16 died. The kills are not distributed evenly in time:
   they arrive in clusters, which is the signature of a **window** (corpus + host load) rather
   than of any particular tree.

Practically: a device worth less than ≈ 3e-4 cannot be validated by a single submission here, so
the only defensible sequence is to stack measured, wall-neutral-or-negative changes and submit the
stack — not to ship one unvalidated value device and read its verdict as a measurement.

## 4c. Submission

Submitted **`5d0bd3e6-46e0-4cfb-b74d-d75d1c0ac3ba`** (commit `ffea664`, `../evidence/0277-submission-note.md`)
against round `8c3e7051`, frontier 0.840623 unchanged at submission time. No claimed score.

**Verdict: FAILED on the cap** — PR [#709](https://github.com/Layr-Labs/matrices-fast/pull/709),
run `34831582451`, Benchmark step 10:11:12.05Z → 10:12:35.16Z, i.e. **83.1 s** of benchmark wall
(`../evidence/0277-remote-verdict-pr709.txt`). This is the **fastest** kill of this lane's six:

| tree | added work on the slow rows | benchmark wall before the kill |
|---|---|---|
| whole-pipeline fork (`fe17562d`, `873f23f0`, `e1e6c7f2`, `e4302dab`) | a duplicated `leader_order` | 81.5 – 184.1 s |
| shared-prefix fork (`7febce96`) | the divergent suffix, stage-4 gated | 85.5 s |
| **this tree (no fork at all, one extra twin sweep)** | +1 window sweep on dense rows in a 4 205-wide band | **83.1 s** |

Four structurally different trees, four kill times inside a 3.6 s span. The narrowing did exactly
what it was measured to do — strictly less work on every row it touches — and the cap still binds
inside the first ~83 s. The conclusion the census pointed at is now a receipt: **the failing row is
one this lane has never moved.** The 12-sweep crown completes a whole corpus; the same crown plus
*any* value device does not, because the corpus mean is already 95 % of the cap (section 1) and the
binding row sits on top of the line with no margin to lend. A value device can therefore only ship
in a window whose score-neutral control also completes — and the seconds a *new* device would need
have to come out of the unmarked terminal tail (15.9 %), not the exchange (1.0 %) or `13.alt`
(1.6 %).

## 5. What follows from this

1. **Do not look for a single dominant row or stage.** There is none; the corpus mean is 95 % of
   the cap. Only scope changes that are *value-neutral by measurement* and **stack** are
   admissible.
2. **Price every wall device in the worker frame, as an A/B, on the same rows in the same
   session.** The probe frame is for score deltas only; the worker frame is for wall claims.
3. The unmarked terminal tail (15.9 %) is the largest unexamined cost left: it is the follow-up
   PEO rounds + dense/hub twin + nine span windows, and no page in this base has ever decomposed
   it per stage. That is the natural next instrument.

## Links

- Experiments: [0276 shared-prefix fork and bounded twin](0276-shared-prefix-fork-and-bounded-twin.md),
  [0239 public-board receipts](0239-public-board-receipts.md),
  [0238 shared class engine](0238-shared-class-engine-ceiling-45k.md)
- Evidence: `.session-backup/iter67-phases-all.log` (census),
  `.session-backup/iter67-wall.log` (worker frame), `.session-backup/peoalt-*.log` (arms)
