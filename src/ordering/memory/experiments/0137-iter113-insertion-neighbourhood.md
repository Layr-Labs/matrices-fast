# 0137 — iter113: insertion neighbourhood + exact convergence exit (terminal SmallScore)

- **Date:** 2026-09-08
- **Base:** `fbef927` (submission `767130f`, rcwrightiii / Grok 4.6, hidden
  **0.848883**), local **0.804873** reproduced exactly on this box.
- **Result:** local **0.804768** (−1.05 bip). 6 movers / 0 worse, all `lt_1k`.
  `1k_10k` and `gt_10k` bit-identical. 71/71 tests.
- **Prior context:** this session's `c617a8eb` (iter111) FAILED the hidden cap by
  putting extra chained rebuild work on gt_10k — see [0136](0136-iter111-gain-conditioned-chain.md).
  Everything here is confined to `n <= 1_000`.

## Two independent findings

### 1. The restart chain converges provably, and the crown paid past it

The crown's terminal SmallScore block ran **four blind restarts** of
`plateau(paired_swap(...))`, chaining `cand` and scoring only the final one.

Both refine passes carry **fixed RNG seeds** (`0x917ad73`, `0xa839d37`), so each
is a deterministic function of its input. Therefore: *the moment a restart
returns its own input unchanged, every later restart is byte-identical.* The
chain has provably converged and the remaining restarts are pure waste. Exiting
there is exact, not heuristic.

Alone this is score-neutral (0.804873, unchanged) — but it is a large **time
refund** on exactly the rows that were nearest the cap (see timings below), and
that refund is what pays for finding 2.

### 2. Insertion is a neighbourhood the crown could not reach

`paired_swap` and `plateau` are both *transposition* neighbourhoods: they
exchange two positions and leave every other pivot's index fixed. An elimination
order is sequence-sensitive, so relocating one pivot across a span shifts its
relationship with every pivot it crosses at once. Those permutations are simply
not in the transposition neighbourhood, which is why this is worth its own pass
rather than more restarts of the existing two.

`cutoff_insertion_refine`: fixed seed `0x5deece6d`, lift position `a`, re-seat at
`b`. **Neutral acceptance matters enormously** — strict-improvement-only scored
0.804857 (−0.16 bip); keeping score-neutral relocations scored 0.804671
(−2.02 bip) at the same iteration count, i.e. ~12x the gain. Same reason
`cutoff_plateau_refine` takes `neutral`.

## The cap trap, and why n and nnz are both unusable as gates

The unbudgeted insertion pass is a **cap killer**:

| INSERTION_ITERS | dev score | WORST order() |
|---|---|---|
| 1024 | 0.804857 | **3.29 s** |
| 2048 | 0.804671 | **4.95 s** |
| 4096 | 0.804647 | **7.80 s** |
| 8192 | 0.804552 | **11.77 s** |

Even the weakest version would have FAILED. The offender is
`maxcsp-langford-3-11` (n=660, nnz=29646).

Critically, **neither `n` nor `nnz` predicts the cost**. At nearly the same `n`:

- `qapw` n=705 **nnz=87496** → +0.21 s (near-complete graph, `flops_bounded`
  cuts off early)
- `maxcsp-langford-3-11` n=660 **nnz=29646** → +4.07 s

A 20x cost spread with the *denser* row being the cheap one. Any `(n, nnz)` gate
tuned on dev would have been tuned on noise.

**Fix: count the actual work.** `SmallScore::flops_bounded_work` charges the
caller for the word-operations it performs; the walk stops when
`INSERTION_WORK_BUDGET` is spent. Fail-closed, structure-independent,
deterministic.

| work budget | dev score | WORST |
|---|---|---|
| 6M (**ships**) | 0.804768 | 0.887 s |
| 12M | 0.804765 | 0.925 s |
| 24M | 0.804764 | **2.06 s** |

Note the **cliff** between 12M and 24M: one row starts converting many samples
and worst-case doubles for ~0 score. Budget tuning near that edge is precisely
what has been killing submissions all day. 6M sits an order of magnitude below
it and buys all but 0.004 bip of what 24M does.

## Timing — the package is FASTER than the crown where it matters

Interleaved crown/candidate, `SSI_PROBE_REPEAT=5`, two pairs:

| row | crown | candidate |
|---|---|---|
| `maxcsp-langford-3-11` | 1.350 / 1.371 s | **0.894 / 0.888 s** (−35%) |
| `qapw` | 1.301 / 1.308 s | **0.648 / 0.648 s** (−50%) |
| `ndcc13` | 0.731 / 0.734 s | **0.546 / 0.537 s** (−26%) |
| `multiplants_stg5` | 0.766 / 0.768 s | 0.842 / 0.840 s (+10%) |
| `crudeoil_lee4_09` | 0.848 / 0.853 s | 0.850 / 0.910 s (noise) |
| `faclay75` | 0.359 / 0.369 s | 0.369 / 0.362 s (noise) |

`crudeoil_lee4_09` (n=15904) and `faclay75` (n=272878) cannot be touched by
`n <= 1_000` code, so their spread fixes the noise floor at ~7%.

The convergence exit refunds more than the budgeted insertion pass spends, so
the crown's two slowest lt_1k rows both get materially cheaper. **The crown's
`maxcsp` at 1.35 s isolated was closer to the 2 s cap than anything else in
lt_1k; this package cuts it to 0.89 s.**

## Movers (6 / 0 worse)

`korcns` 0.9155→0.9113, `pooling_adhya4tp` 0.7667→0.7652, `sporttournament18`
0.7754→0.7700, `wastewater05m1` 0.6879→0.6843, `waterund11` 0.7261→0.6992,
`waterund14` 0.3620→0.3615. All n between 98 and 333.

Honest caveat: 6 movers is narrow, and it is a pure `lt_1k` move — the crown's
own note flags that such moves "can go either way" on the hidden set. The reason
to ship anyway is the timing profile, not the breadth: this is the first package
in this session that *reduces* the parent's worst case.

## What did NOT work

- Insertion with strict-improvement-only acceptance: −0.16 bip vs −2.02 bip
  neutral. Do not drop `neutral`.
- Early-exit restart loop alone: exactly score-neutral. It is a time mechanism,
  not a score mechanism; its value is that it funds the insertion pass.
- Raising `INSERTION_ITERS` without a work budget: fails the cap at every count
  tested, including the lowest.

## Next

- The remaining insertion gain is locked behind expensive rows (`maxcsp`,
  `multiplants_stg5`, `ndcc13`): ungated they give −2.02 bip, budgeted −1.05.
  Recovering it needs a *cheaper scorer*, not a bigger budget — an incremental
  flop delta for a single relocation instead of a full `flops_bounded` rescore.
  That is the highest-value open lead here.
- Do NOT widen `n` past 1000, and do NOT raise the work budget toward 24M.
