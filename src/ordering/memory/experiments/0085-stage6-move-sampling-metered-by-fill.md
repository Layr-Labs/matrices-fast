# Stage-6 move sampling, metered by fill rather than by the gate

## Baseline

Promoted tip `4d86414` (submission `e5310ad6`, alternate-seed chains, hidden
**0.859573**). Local dev on this host, worker rebuilt from that tree:
**0.826784**, fill 0.937368 (lt_1k 0.889730 / 1k_10k 0.863292 / gt_10k
0.752193). All numbers below are pinned to two cores under bubblewrap and read
as deltas against an arm measured in the same interleaved run.

## The reading that started this: dev bips translate much better than the log says

`0084` moved dev by 4.11 bips and hidden by **2.61 relative bips**
(0.859834 -> 0.859573), a translation of **0.64**. The three ledger steps before
it translated at 0.10 / 0.15 / 0.15. The difference is not noise and it is not
the family: those three moved `gt_10k` alone, while `0084` moved four buckets'
worth of rows. So the required dev margin for a promotion is **not** a constant
of this benchmark - it is a function of where in the bucket ladder the gain
lands, and a change that moves `lt_1k` and `1k_10k` is worth measuring even at
one or two dev bips.

## Finding 1: stage 6 is not at a local optimum, it is at a local optimum of 1536 fixed tuples

`cutoff_paired_swap_refine` draws 512 four-position tuples at seed `0x917ad73`
and `cutoff_plateau_refine` draws 1024 two-position tuples at `0xa839d37`, both
with iteration counts that do not scale with `n`, on the gate
`12 <= n <= 300 && nnz <= 3000` (92 dev rows). The already-passing
`cutoff_differential` test reports `wins_vs_leader=0`: a second application of
those same streams improves nothing. That is local optimality **with respect to
those 1536 position tuples only**, and because the iteration count IS the length
of one xorshift stream, "more iterations" and "another seed" are the same
experiment.

Replacing the streams instead of repeating them, best-of, from `leader_order`'s
own output (a post-hoc probe sees exactly the incumbent and exactly the shipped
admission test, so it returns the shipped score rather than an estimate):

| independent stream sets | winning rows | dev bips |
|---:|---:|---:|
| 1 | 0 | 0.00 |
| 2 | 1 | 0.04 |
| 4 | 3 | 0.09 |
| 8 | 4 | 0.26 |
| 32 | 4 | 0.31 |
| 64 | +1 | +0.07 |
| 128 | +1 | +0.03 |
| 256 | 0 | 0.00 |

Shipped in production (before stage 7, so a better `best_perm` re-enters the PEO
chain) the same 32 sets are worth **0.86 dev bips**, three times the post-hoc
figure, because the chain compounds them.

## Finding 2: the stage's cost is set by fill, and no input gate bounds it

This is the part that would have failed hidden validation, and the dev corpus
does not show it. One `SmallScore` pass clones `n * words` u64s and then ORs a
`words`-wide row into every factor entry, so it costs `words * (n + fill)` word
operations. **`nnz` does not appear.** A sparse pattern whose elimination fills
in completely costs exactly what a dense one costs, and `fill` is bounded only
by `n(n+1)/2`, which no gate on `(n, nnz)` can lower.

Measured directly: dropping the `nnz <= 3000` half of the gate (arguing, wrongly,
that `n <= 300` makes the work an absolute constant) admits `graphpart_clique-70`
(n=280, nnz=14910) and takes that row to **1.771 s** against a 2 s per-matrix cap
and a 1.369 s corpus maximum. It scored +0.18 bips. It was discarded.

The same arithmetic condemns the version that keeps the gate: at `n = 300` with
`nnz = 3000`, a pattern that fills in completely is ~30x the corpus's densest
in-gate row, and 32 unmetered sets on it run for seconds. **The gate is not the
bound; the fill is.** So the set count is solved from an explicit op budget:

```
per_set = words * (n + fill) * (512 + 1024)          // word operations
sets    = clamp(CUTOFF_STREAM_UNITS / per_set, 1, 32)
CUTOFF_STREAM_UNITS = 150_000_000
```

Set 0 is `(0x917ad73, 0xa839d37)` started from the incoming permutation, i.e.
bit-for-bit the shipped pass, and `sets >= 1` always - so the floor is the
incumbent's own cost and the ADDED work is at most `CUTOFF_STREAM_UNITS` word
operations no matter what the input looks like. Calibration over the 92 in-gate
rows: one word operation costs 0.54-1.04 ns here (median 0.88), so the budget is
130-160 ms on this host and ~300 ms on the slowest host on record. Metered, the
stage costs 5.2 s corpus-wide with a **152 ms maximum row**, and scores 0.826637
against 0.826634 unmetered - the meter costs 0.03 bips and removes a
seconds-long tail.

## Finding 3: the alternate-seed pool wants 8, not 4, and 16 adds nothing

`PEO_ALT_SEEDS` retains the k best DISTINCT displaced scores, which is monotone
in k: a larger pool's first four entries are exactly the shipped pool's, in the
same order, so a larger k re-runs the shipped chains unchanged and then spends
whatever the shared 4M-unit `PEO_ALT_LEDGER` has left. Score is monotone
non-worsening and the per-matrix work envelope is the SAME ledger.

| pool | dev | movers vs pool 4 | corpus total | corpus maximum |
|---:|---:|---:|---:|---:|
| 4 (shipped) | 0.826784 | - | 140.7 s | 1.365 s |
| 8 | 0.826723 | 4 | 144.3 s | 1.372 s |
| 16 | 0.826723 | 4 | 148.7 s | 1.402 s |
| 32 | 0.826703 | 5 | 154.1 s | 1.414 s |

Eight takes the knee: `sonet21v6` 0.9839 -> 0.9650, `mpbp_15` 0.8094 -> 0.8052,
`syn40hfsg`, `p_ball_10b_5p_4d_m`. Thirty-two buys 0.20 bips more for +49 ms on
the corpus maximum, which is the tail rule's whole tolerance, and was discarded.
The retention itself is now cheaper than before: an entry whose score is already
held, or which loses to a full pool's worst, is dropped again by the very next
dedup, so deciding that BEFORE cloning replaces an O(n) copy plus a sort with an
O(pool) scan of u64 keys, bit-identically; and a matrix outside the chain gate
retains nothing at all.

Address space, which `RULES.md:109` caps at 4 GiB per matrix and explicitly does
not apply to local runs, is the one term local evidence cannot see. Four seeds at
the chain gate's corner (`n + nnz < 4M`) cost 128 MB; eight would cost 256 MB, so
the pool is clamped to 16M retained entries, which holds eight seeds inside the
incumbent's own 128 MB and never falls below four anywhere the gate admits.

## Finding 4: two provably score-neutral items in the terminal chain

- **Pre-symbolic entry refusal on the above-gate branch.** Each round costs
  `5(n + nnz) + Lnnz` against a 2.5M ledger and `Lnnz >= n >= 16`, so
  `5(n + nnz) >= 2_500_000` already implies the first round is refused - after
  the branch has paid `permute_pattern`, `EliminationTree::from_pattern` and
  `column_counts_gnp`. Deciding it from `(n, nnz)` alone is score-identical by
  arithmetic. Measured: `acopf_case9241pegase_qcqp` **-76.8 ms**, `faclay75`
  -40.1, `gabriel10` -28.7.
- **Two redundant `is_bijection` calls** in `peo_extract::candidates_bounded`.
  `mcs_peo` sets `visited[v]` before pushing `v` and never pushes a visited
  vertex, and every pushed value comes from `incumbent` (already validated) or
  from `adj` (built only from vertices `< n`), so `len() == n` already certifies
  a bijection. Two allocations and two random O(n) passes per round removed.

## Result

Full 300-matrix trusted run, baseline then candidate:

| | `4d86414` | candidate |
|---|---:|---:|
| weighted flop geomean | 0.826784 | **0.826637** |
| fill tiebreak | 0.937368 | 0.937346 |
| lt_1k | 0.889730 | **0.889444** |
| 1k_10k | 0.863292 | **0.863089** |
| gt_10k | 0.752193 | 0.752193 |

**1.47 dev bips, zero per-row regressions, 68 tests** — and `failed` hidden
validation, see below. `lt_1k` moves for the first time in five promotions - it is not saturated, it was simply never aimed
at: no `lt_1k` row is ever charged by either PEO ledger, so no allowance,
currency or round cap in that family could ever have moved it.

Interleaved pinned A/B, two trials per arm, score bit-identical at 0.826637 in
both: median corpus maximum 1.3689 -> 1.3838 s (**+14.9 ms** against a +50 ms
tolerance at 21.7 sigma), no row that was below the base tail crosses it, worst
per-row increase 153.7 ms (`korcns`, inside the 175 ms declared budget), corpus
total +8.9 s of which the refusal returns 0.6 s.

## Hidden outcome: the full candidate FAILED, and what that isolates

Submission `6ce0721` carried all four findings and came back **`failed`** — no score, no instance,
no category. The local evidence was clean: every hard gate passed, the median corpus maximum moved
+14.9 ms against a +50 ms tolerance at 21.7 sigma, 68 tests, purity clean, determinism bit-identical
across trials.

The statistic the tail gate did **not** look at, and which is the obvious suspect: the candidate
raised **104 of 300 rows by more than 25 ms, with a 153.7 ms maximum**. None of those rows is near
the cap on the dev corpus, so every gate passed. The hidden corpus is disjoint and rotated, so a
hidden row that already sits near 2 s and happens to be inside the stage-6 gate — or to have unspent
alternate-seed ledger — receives the same increase with no margin left. **A broad, large,
many-row increase is not made tail-safe by the dev corpus's own tail row being untouched.**

The multi-set stage-6 change is therefore withdrawn. It is the change that produced essentially all
of the 104 rows and all of the 153.7 ms; the alternate-seed pool deepening spends only ledger the
incumbent already permits, and the two hygiene items strictly *return* time. Submission `6ce0721x`
re-runs the candidate **without** the stage-6 sets as a bisection: a `rejected` there attributes the
`failed` to stage 6 and leaves the pool and hygiene safe to build on; a second `failed` means the
realized-time increase from spending an unchanged ledger is itself enough, which would be a much
larger result.

If the multi-set stage is ever rebuilt, `CUTOFF_STREAM_UNITS` needs to be an order of magnitude
smaller — 10-20M units, i.e. 10-20 ms per row rather than 130-160 — and the acceptance statistic
needs to be **the whole distribution of per-row increases**, not the maximum row's.

## What this closes

- **Stage-6 seed lottery**: open at 32 sets, closed above 128 - sets 129-256 win
  nothing. The remaining `n <= 300` headroom is not in this move class.
- **Alternate-seed pool depth**: closed at 8. Sixteen is worth exactly zero and
  thirty-two is worth 0.20 bips at the tail rule's whole tolerance.
- **"Absolute constant" arguments from a `(n, nnz)` gate**: not sound for
  anything whose cost is driven by the FACTOR. Check whether the quantity that
  sets the cost is an input or an output before calling a gate a bound.

## Next

`0084`'s own next step - seeds from the stages that assign `best_perm` directly
rather than through `consider` - is untested here, but this round's pool sweep is
evidence against it: 8x more seeds bought 0.20 bips, so the pool is not starved
for entries. Test it as a diversity change (different lineages), not as a
quantity change, and keep the shipped seeds first so the result stays monotone.
