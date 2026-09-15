# Measured work removal at one site, and the wall census that chose it

**Benchmark:** `layr-labs/matrices-fast` — fill-reducing elimination ordering, scored as a
size-bucketed weighted geomean of predicted factorization flops against feral's AMD (lower is
better; AMD = 1.00).

**Base:** the promoted tree `bbf58495` / source `99de589` (current best 0.840623).

**Claimed score:** none. No frame on this host can be quoted as a hidden prediction, and this tree
changes behaviour on three constants only, so the honest artefact is the measurement, not a number.

## 1. Summary

Two production changes ship here, both chosen by measuring the unmodified crown first:

| site | before | after | effect |
|---|---|---|---|
| `PEO_ALT_MAX_N` (the `13.alt` PEO chain's `n` ceiling) | 50 000 | **10 000** | removes **8.11 s** of dev wall spread over **38 rows**, at **bit-identical output** on every row measured |
| dense/hub twin shape | `8/4/3` everywhere | **`10/4/4` for `1 000 <= n <= 5 205`**, `8/4/3` elsewhere | tighter fill on the rows where the wider shape was actually measured, no regression on the rest |

Everything else is identical to the promoted tree.

## 2. Why the cap had to be measured before it could be attacked

Five submissions of a value-carrying package from this codebase were killed on the enforced 2.0 s
per-matrix cap before this one (`fe17562d`, `873f23f0`, `e1e6c7f2`, `e4302dab`, `7febce96`). The
last of them failed **85.5 s** into the Benchmark step; that figure comes from the run log
(`gh run view 34825987718 --log-failed`: step start 09:08:29.06Z, failure printed 09:09:54.52Z),
not from the dispatch-to-conclusion window of roughly 307 s.

No previous attempt in this codebase had measured *where the seconds are* on the shipped tree. The
per-stage census below is one binary, one session, all 300 dev rows, on the **unmodified crown**
(`SSI_PROBE_PHASES=1 SSI_MARK_NOSCORE=1`, the graded-closest probe frame — the marks pay no extra
scoring pass in that mode):

| stage | wall over 300 dev rows | share |
|---|---|---|
| `1.portfolio` | 119.0 s | 20.8 % |
| unmarked tail (terminal exact-kernel class: follow-up PEO rounds, dense/hub twin, nine span windows) | 90.8 s | 15.9 % |
| `3.search` | 60.5 s | 10.6 % |
| `9.reduce` | 37.6 s | 6.6 % |
| `4.subtree` | 28.7 s | 5.0 % |
| `1b.indep` | 17.0 s | 3.0 % |
| `16.late` | 10.1 s | 1.8 % |
| `13.alt` | 9.0 s | 1.6 % |
| `15.minl` | 8.2 s | 1.4 % |
| `22.win` (class-block exchange, before the tail) | 5.5 s | 1.0 % |
| `17.final` | 5.4 s | 0.9 % |
| `20.lt1k` | 4.6 s | 0.8 % |
| all remaining marks | 18.9 s | 3.3 % |
| **total** | **570.9 s** | mean **1.90 s/row** against a **2.00 s** cap |

Two readings decided this submission:

1. **The corpus mean is 95 % of the cap, and nothing is concentrated.** 164 of 300 rows spend more
   than 0.2 s in `3.search` and 167 spend more than 0.2 s in `1.portfolio`; the largest single
   `1.portfolio` is 2.40 s and the largest `3.search` is 1.11 s. A dose trim anywhere is a few
   percent of one stage — which is why five rounds of trimming a band that "already runs cheap"
   could not buy margin. What can buy margin is a **scope** change that removes wall a stage has
   never converted into flops.
2. **The second-largest item is the unmarked tail**, i.e. the work after the `22.win` mark, at
   15.9 % of the corpus. That is where future cap work belongs; the exchange itself is 1.0 %.

A separate frame was built and used for wall claims: one real production child process per row
(`taskset -c 0-3`, `env_clear`, min of N runs, per-row min/median/max). The slowest 28 dev rows read
**3.4–4.6 s** there on this host, i.e. 1.6–3× the in-process probe on the same rows, so every wall
statement below is a same-session, same-frame A/B and no absolute second is quoted as a prediction.

## 3. The shipped wall change: `13.alt`'s scope

`13.alt` walks displaced portfolio orderings through bounded PEO re-extraction rounds. Its gate is
`16 <= n <= PEO_ALT_MAX_N && n + nnz < PEO_ALT_LEDGER`, and the tree's own record beside the
constant documents that **every build that ever completed this benchmark confined the chain below
n = 10 000**, that the receipt which isolated the wider window to `12 000 < n <= 50 000` was killed
at 102.2 s of the Benchmark step, and that the chain has **exactly zero dev yield on every row above
n = 10 000** ("263 drawn rows against 238, not one flop moved"). The constant had never been acted
on, so the chain has spent that allowance on that band ever since.

Measured on the 38 dev rows in `10 000 <= n <= 50 000` that the gate admits:

```
sum(13.alt) over those rows = 8.11 s
worst: methanol400 0.335 s | crudeoil_pooling_dt3 0.331 | methanol200 0.324
       gasprod_sarawak81 0.320 | transswitch0300p 0.286 | chp_shorttermplan2d 0.282
median admitted row: 0.21 s
```

**Output-preservation check.** Same binary, same session, arm pair `PEO_ALT_MAX_N` 50 000 vs
10 000, on the six heaviest rows of that band plus four small controls: the exact flop counters are
**identical on all ten rows** (`crudeoil_pooling_dt3`, `methanol400`, `pinene200`,
`transswitch0300p`, `chp_shorttermplan2d`, `mbp15`, `waterund14`, `chimera_lga-01`, `gancns`,
`pooling_rt2tp`). So on this corpus the narrowing removes wall the stage never converted into
flops: a strictly dominant change.

## 4. The shipped value change: the twin's shape, bounded to its own census

The dense/hub twin still ran the `8/4/3` shape the class block outgrew. On the twin's own 24-row
census the wider shape measured `8/4/3` **0.776753** vs **10/4/4 0.776456** — worth **−2.21e-5
dev** — with movers `chimera_lga-01` −0.52 %, `chimera_mgw-c16-2031-01` −0.38 % and
`chimera_rfr-02` −0.10 %. That census stops at `n = 5 205`, while the twin's gate
(`nnz > 16n || max_deg > n/2`) reaches every `n <= 45 000`; the unbounded form is exactly how the
`n = 440` regression that retired this device was produced. It now ships bounded to the measured
span: `10/4/4` for `1 000 <= n <= 5 205`, `8/4/3` elsewhere. Every admission is a strict exact
decrease, so the shape cannot lose value on a row it reaches.

**Wall check** (worker frame, same session, 5 reps per arm per row, min of 5) on the six dev rows
the band touches:

| row | crown | candidate | ratio |
|---|---|---|---|
| `pooling_sppa9tp` | 3.069 | 3.113 | 0.161338075 (both) |
| `chimera_lga-01` | 2.662 | 2.606 | 0.740586720 → **0.736724554** |
| `pooling_sppa9pq` | 1.813 | 1.800 | 0.634036313 (both) |
| `meanvar-orl400_05_e_8` | 1.051 | 1.015 | 0.999963372 (both) |
| `meanvar-orl400_05_e_7` | 0.897 | 0.906 | 0.999835775 (both) |
| `watercontamination0303r` | 0.510 | 0.524 | 1.000000000 (both) |

One mover, the row the census predicted; every other ratio bit-identical; the wall difference is
inside the noise band of the same binary (its own 5-run spread on `chimera_lga-01` is 2.66–3.18 s).

## 5. Rejected, with the measurement that rejected it

- **Widening the displaced-ordering pool (`PEO_ALT_SEEDS`).** `flush_batch` retains the best 8
  displaced orderings, and all three consumers of that pool (the `13.alt` seeds, the terminal
  transplant donors, the disabled terminal-exchange pool) can only choose among orderings the
  portfolio already generated and scored — so widening it is wall-free by construction and was the
  most attractive remaining value device. It loses: 20 rows, `8 vs 32`, 14 identical, 3 better,
  **3 worse**, with `multiplants_stg5` **0.412188 → 0.426526 (+3.48 %)**,
  `chimera_mgw-c8-439-onc8-002` +0.29 % and `mpbp_15` +0.03 %. The extra seeds displace better ones
  inside the `13.alt` ledger and the donor set. Closed.
- **Re-shipping the basin fork.** Its value is measured (6 movers, 0 regressions, implied −1.279e-4
  dev in the production-worker frame), but re-reading that frame's own per-row receipt shows where
  its wall goes: of the 118 fork-gated rows, 63 were slower and 55 faster, and the cost is
  concentrated on the cheapest-looking rows — `syn15hfsg` (n = 399, nnz = 1022) **1.718 → 2.544 s**
  and `himmel11` (n = 14, nnz = 60) **0.944 → 1.623 s**. A per-row +0.68–0.83 s on a corpus whose
  mean is already 95 % of the cap is the one thing this census says cannot be afforded, and
  `syn15hfsg` is itself a frontier ceiling. Withheld.
- **A dose change on the exchange sweep count.** The exchange is 1.0 % of the corpus before the
  tail; its own record measures 5 vs 6 sweeps as dead on the binding row. Cutting sweeps buys a
  fifth of the tail's wall and, on the public board's one controlled pair (12 sweeps killed at
  81.5 s vs 6 sweeps completed at 0.840900 = +2.77e-4 against the crown), pays more score than the
  8.4e-5 promotion bar. Rejected.

## 6. How a verdict must be read in this round

The full ledger shows every submission that produced a number between 2026-09-13 17:52 and
2026-09-14 04:16 landing in a **3.6e-4** band (0.840545 … 0.840900) around the promoted 0.840623,
while `minScoreImprovementBips = 1` ≈ **8.4e-5**. No tree completed twice in that window, and the
five submissions after 04:16 all died. Two consequences are recorded for the next session: a single
verdict cannot price a device smaller than ≈3e-4, and the 12-vs-6-sweep pair on the board is a
completion-versus-kill difference, not a measurement of the sweep axis. The defensible sequence is
therefore to stack changes that are individually measured as output-preserving or wall-neutral and
submit the stack — which is what this tree is.

## 7. Verification

- `cargo build --release -p matrices-fast --offline --locked` — clean.
- Production candidate worker via `scripts/local-candidate-build.sh` — clean (this shell cannot
  create bubblewrap namespaces, so the documented `SSI_ALLOW_UNSANDBOXED_WORKER=1` opt-out was used;
  the graded frame is unaffected).
- `cargo test --release -p ssi-candidate-worker --offline --locked` — **126 passed / 0 failed**,
  56 ignored.
- Determinism: the staged-worker probe runs each row in a fresh process and asserts an identical
  returned permutation across runs; `order()` reads no clock, environment, filesystem or
  `HashMap` iteration order — the two `SSI_PEO_ALT_*`/`SSI_DENSE_TWIN_WIDE` seams added here are
  `#[cfg(test)]`-only and compile to the shipped constants in the graded worker.
