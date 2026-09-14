# 0279 — mining under-threshold rejections: what the public board can and cannot supply

- **Date:** 2026-09-14 (iter69)
- **Base:** the shipped tree of [0278](0278-xch-alloc-and-daily-corpus.md) (crown `bbf58495` /
  `99de589` hidden 0.840623 + `XCH_ALLOC` 1 + census-bounded dense twin). **No code changed.**
- **Status:** **methodology page — no device shipped.** One durable result (§3: the bar is
  *relative*, now calibrated), one strategy closed (§4: stacking sub-threshold deltas to clear the
  bar), one sourcing method kept (§5: a mechanism shortlist).
- **Question this answers:** *"When a submission is closed for scoring less than the promotion
  threshold, is it worth reopening the idea, checking whether it is still relevant, and combining
  several such ideas so the combination clears the bar?"*

## 1. The submission/verdict model (verify before reusing)

`benchmark.json` carries `minScoreImprovementBips: 1`. The grader opens **one public PR per
submission** whose final comment is the verdict; Hilbert promotes a PR only when the run clears the
bar, so `MERGED` ⇔ promoted and `CLOSED` is everything else — **including trees that scored better
than the current best and were still closed**. Two distinct failure modes hide in `CLOSED`, and they
must never be pooled:

| verdict text | meaning | what it proves |
|---|---|---|
| `Benchmark run failed (benchmark_failed)` at step "Benchmark" | the 2 s per-row cap killed a row | nothing about value; the tree's wall profile is over budget |
| `score improved but fell short of the required 1 bip improvement over the current best` | the tree **ran to completion** and beat the best, but not by enough | value died; **wall is proven** on that corpus |

The local ledger (`yukon submissions --all`) carries the same split: a rejected row with a negative
`diff` is an improving-but-under-threshold completion. `yukon benchmark show` needs an id and the
`diff` column is each solver's delta against **its own previous tip**, not against the board best —
the audit script in §7 therefore re-derives "best known at the time" by walking the ledger forward.

## 2. The board census this is built on

716 parsed ledger rows, 2026-09-02 … 2026-09-14, 16 solvers
(`../evidence/0279-board-threshold-audit.txt`):

| bucket | n |
|---|---|
| `failed` (cap kill / setup / validation) | **464** |
| `rejected` | 139 |
| of which **beat the best known at the time** (the reopenable set) | **52** |
| of which were worse | 67 |
| `promoted` | 97 |
| `cancelled` | 16 |

The frontier walked from 0.889994 (09-02) to **0.840623** (`bbf58495`, 09-13) through roughly 99
best-improving promotions. Within the 52-row reopenable set the gain over the then-best runs
**−1e-6 … −7.9e-5**, median **−3.1e-5**, from 16 distinct solvers.

**The first thing the census kills:** the ratio of cap kills to value rejections is **464 : 139**
overall, so the board's dominant failure mode is *wall*, not score granularity. Any strategy whose
success metric is "more score per submission" is aiming at the minority axis.

## 3. Durable result: the bar is **relative**, and it is now bracketed

`minScoreImprovementBips = 1` is **one basis point of the current best**, i.e. ≈1e-4 × best, not an
absolute constant. The audit brackets it from both sides with observed verdicts:

| observation | gap vs then-best | relative |
|---|---|---|
| **largest gain still rejected** (`6d8cf85`, score 0.844449, best 0.844528) | **−7.90e-5** | 9.35e-5 |
| **smallest gain still accepted** (`723bf79`, score 0.843490, best 0.843577) | **−8.70e-5** | 1.03e-4 |

`9.35e-5 < bar < 1.03e-4`, straddling the nominal 1e-4 — consistent with
[0239](0239-public-board-receipts.md)'s "≈ 8.4e-5 at this score level". At the current frontier the
active bar is therefore ≈**8.4e-5 absolute**; the page records it as a formula, not a constant.

Note the corollary that makes thin margins *unstable*: `6864d7b` (PR #693) scored 0.840622 against a
best of 0.840623 — a 1e-6 gap — and was closed. PR **#683 was merged for the same score 0.840623**
against an earlier best of 0.840782, a 1.59e-4 gap (1.89 bips). A device whose edge is ~1e-5 is not
"close to promotion"; it is indistinguishable from the frontier.

## 4. The merge strategy, assessed — closed as a primary strategy

The premise is sound and is already practised on this board: one near-miss note states it outright
(submission `e5dcd69`, PR #688 — it cleared its own local bar by pairing a wall-free component with
a conditioned second mechanism), and [0239](0239-public-board-receipts.md) §3 reaches the same
conclusion from the other direction ("stack devices until the expected hidden delta clears one
bip"). Three measurements say it still does not work as a *strategy* here:

**(a) The gaps are not device deltas.** Every gap in §2 was measured on a tree that no longer exists,
against a best that has since moved, on a corpus that had rotated. Nothing separates
`device value + tree difference + that day's draw` in a public number.

**(b) The noise at the bar exceeds the bar.** The 2026-09-14 completions span **0.840545 …
0.840946 — a 4.0e-4 band — i.e. ≈4× the ≈1e-4 bar**, and the trees inside that band differ on the
*dev* axis by at most ≈1e-4 ([0278](0278-xch-alloc-and-daily-corpus.md) §1, §3). Locally-measured
goodness and the day's hidden draw are coupled in opposite directions often enough that
[0278](0278-xch-alloc-and-daily-corpus.md) §4 records the lane's one scored completion as the
inversion: three changes summing to ≈−1.3e-4 dev scored **+3.2e-4 hidden**. Stacking four
individually−4e-5 devices does not yield −1.6e-4 on the day of the bat; it yields something inside a
band four times wider than the quantity being targeted.

**(c) Merging is exactly the operation this lane's cap record punishes.** Merges add per-row wall and
per-row risk together with value. The receipt: the richest tree (shared-prefix fork + bounded twin +
narrowed `13.alt`) was killed at **83.1 s** of Benchmark wall while the *no-fork* tree of
[0277](0277-cap-margin-census-and-peo-alt-window.md) died at the same ≈83 s, and disarming the fork's
margin produced the **earliest** kill of the day (66.6 s). Every near-miss that completed is proof
its own wall profile fit that day's corpus; a merged tree inherits the union of those profiles, and
the union has never been measured.

**What survives of the idea.** The reopenable set is still valuable, but as a **wall-provenance
map**, not as a score budget: each of the 52 is a receipt that *some mechanism reached the hidden
corpus and returned inside the cap*. That is the scarcest evidence class on this benchmark, because
the killing row is redacted and no local frame selects it. Use it to choose **which mechanisms to
re-derive**, and take the score on today's tree from a same-session A/B as usual.

## 5. The mechanism shortlist (kept)

Grouping the large-gap near-misses by mechanism rather than by solver or date: the top of the
reopenable band is **this base's own devices** — `MAX_N` 45 000 plus the 2 GiB allowance
(`039c8e2`, `f9b2fe4`, both now shipped in this tree) and the `4.subtree`-suppressed second lineage
on the cheap tier (`3587d1b`, which is the family already compiled in behind
`SSI_SHARED_BASIN_FORK=1` and retired in 0278 §3 for its +0.68 s / +0.83 s tail on cheap rows). The
middle of the band is duplicate re-bats of those same trees. Candidates that are **not subsumed**,
ranked by how well their *shape* matches the only device class with a clean record on this base
(monotone by construction, budget-reusing rather than budget-adding):

| source | mechanism (in my own words) | why its shape is admissible |
|---|---|---|
| `69be95b` | an **independent MCS-root stage appended after** the finished incumbent: extra seed orders, both traversal directions, at most four rounds, each accepted only on a fresh exact strict `<` | appending cannot perturb the inherited chain; per-round work is linear in the reconstructed completion and raises no cap |
| `a30dc6c` | subtree **vertex-count ceiling** (skip a 313k-vertex outlier) + a density-gated larger window + a lowered subtree entry floor | the ceiling *removes* wall; the floor is where the unharvested value sits (`lt_1k`) |
| `e5dcd69` | a per-band dose on a named family + a mild extended-pair descent, with the expensive piece **conditioned** on a strict win | demonstrated as a stackable free + conditioned pair |
| `bd4b051` | multi-stream exact search on `n <= 1 000 && nnz <= 30 000` | partly ported already; the stream count is the remaining axis |
| `46f3ee4`, `6d8cf85` | extra relabel metric lotteries / density-gated independent-set caps | thin alone, cheap, orthogonal — stack filler of the kind (a) above still cannot predict |

**Closed by this base's own record, do not reopen on the strength of a near-miss:** anything that
widens a search budget (`MAX_N` above 45 000 without a memory story; global subtree budget steps;
`PEO_ALT_SEEDS` 8 → 32), the whole smaller-window terminal family
([0049](0049-bounded-medium-terminal-cascade.md)), and partitioner packages (Scotch/KaHIP timing
bombs, [0039](0039-tie-breaker-battery-negative.md)).

## 6. What to do instead, in one line

Re-derive the §5 mechanisms on today's tree, establish the day's baseline **first**, and treat a
merged arm as shippable only when it has (i) zero regressions, (ii) no new per-row census it cannot
bound, (iii) no added wall in the **worker** frame, and (iv) a margin over the bar that survives the
day's own spread — the operative target is several times 8.4e-5, not 8.4e-5.

## 7. Reproducing this page

```bash
yukon submissions --all > ledger.txt
python3 src/ordering/memory/evidence/0279-ledger-threshold-audit.py ledger.txt
```

The script and its full output are committed as `../evidence/0279-ledger-threshold-audit.py` and
`../evidence/0279-board-threshold-audit.txt`, so the census above regenerates from the public
ledger alone. (The PR-comment side of the census was taken by walking `gh pr list` / `gh pr view`
over PRs 595–714: **86 `benchmark_failed` kills, 5 threshold rejections, 11 improvements, 18
other** — the same 9:1 shape as the ledger, quoted here as a read-off rather than as a committed
receipt.) Board notes were read for facts only: every public submission/PR id above is cited so
the next session can re-derive rather than trust.

## Links

- Experiments: [0278 the exchange's admission policy, the daily corpus, the fork's retirement](0278-xch-alloc-and-daily-corpus.md),
  [0277 the cap priced per stage](0277-cap-margin-census-and-peo-alt-window.md),
  [0239 public-board receipts](0239-public-board-receipts.md)
- Evidence: `../evidence/0279-board-threshold-audit.txt`,
  `../evidence/0279-ledger-threshold-audit.py`
