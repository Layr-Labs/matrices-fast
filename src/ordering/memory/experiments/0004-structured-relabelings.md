# 0004 — Structured relabelings vs i.i.d. draws: the top lead is a dead end

**Date:** 2026-09-02
**Score:** 0.876925 → **0.876925** (unchanged; shipped `order()` deliberately not modified)
**Status:** NEGATIVE, committed — closes the top item in
[open-questions](../open-questions.md)

The single most-recommended next step in this knowledge base does not work. It is
not marginal and it is not a tuning problem: at equal cost, **no** structured
relabeling policy beats uniform i.i.d. draws, and every policy that appears to
owes the whole of its apparent gain to one matrix.

## The lead, as it was written

From [open-questions](../open-questions.md), listed as "top lead":

> **Structured relabelings, not random ones.** The shipped multi-start uses
> uniform-random `Q` (Fisher-Yates), which is the dumbest possible choice — it works
> only because AMD's tie-breaking is numbering-sensitive. A *structured* `Q` should
> do better at identical cost: seed the relabeling from RCM, from a partitioner's
> block order, or from a previous restart's winning permutation (a hill-climb rather
> than i.i.d. sampling).

This experiment tests the third and most promising form — **hill-climbing on the
relabeling** — because it is the only one that needs no extra work per restart and
therefore keeps [0003](0003-relabelled-amd-multistart.md)'s cost model exactly
intact.

## Why it should have worked

[0003](0003-relabelled-amd-multistart.md) established that `AMD(Q A Qᵀ)` composed
back through `Q` is a randomized-restart minimum degree for the price of one AMD
pass. It draws every `Q` uniformly, which is **memoryless**: a relabeling that AMD
happens to like teaches the next restart nothing. If "AMD likes this numbering" is a
property of *local* tie-break structure, it should be partly preserved under a
partial perturbation of `Q` — in which case the neighbourhood of an accepted
relabeling is a strictly better place to sample than the whole symmetric group, at
identical cost.

## Step 1 — the naive version, end to end (the expensive way)

Shipped `order()` modified to explore-then-exploit: first `ceil(restarts/2)`
restarts i.i.d. as before, the rest perturbing the best relabeling found so far by
`n/8` random transpositions, accepting a perturbation as the new base whenever it
lowers exact integer flops.

| policy | dev score |
|---|---|
| i.i.d. restarts (shipped) | **0.876925** |
| 50/50 explore-then-exploit, `n/8` | 0.878064 (**worse by 0.001139**) |

Only **25 of 300** matrices changed at all: 7 improved, 18 regressed. The regression
list identified the mechanism precisely — `chp_shorttermplan2d` went 0.7280 → 0.7640,
and **0.7638 is its exact pre-[0003](0003-relabelled-amd-multistart.md) value**. Its
entire relabel win came from an i.i.d. draw in the *second half* of the restart
sequence, which the 50/50 split deleted. `chimera_lga-01` behaved the same way
(0.911 → 0.991, i.e. back to nearly the AMD tie).

**Correction to an earlier claim.** [0003](0003-relabelled-amd-multistart.md) says
"the wins land in the first handful of restarts". That is true *on average* and
misleading in practice: the score is carried by a few very large tail wins, and those
land at arbitrary positions in the sequence. Halving the number of i.i.d. draws costs
far more than the average improvement curve suggests.

## Step 2 — `probe_relabel_search`: sweep the space, don't guess in it

One end-to-end run per policy is ~6 minutes and confounds the relabel family with the
other ~30 candidates. So the new probe:

- evaluates the **pure relabel family against AMD** and deliberately does *not* take
  a `min` with the rest of the portfolio — the other candidates are identical across
  policies, so a `min` masks exactly the differences being measured;
- shares the i.i.d. prefix across all policies, so every policy is scored on the same
  draws and only the exploit tail differs;
- needs **no timing measurement at all**, because every policy performs exactly
  `restarts` AMD passes plus one `O(n)` relabeling — cost-neutrality is structural,
  not measured;
- runs the whole 300-matrix corpus in **~10 s**, so the space is cheap to sweep.

Policy space: split ratio (`7/8, 5/6, 3/4, 2/3, 1/2` i.i.d. before switching) ×
perturbation strength (`n/2, n/4, n/8, n/16, n/32, n/64`) × schedule (`FIXED`,
`DECAY` = `n/2, n/4, n/8, …`, `DECAY0` = `n, n/2, n/4, …`, `RESET` = variable-
neighbourhood search resetting to widest on improvement, `NOCHAIN` = perturb the best
i.i.d. draw and never chain). 17 policies per run, two runs.

Pure-relabel score (vs AMD, lower better; i.i.d. = 0.959167):

| policy | score | Δ vs i.i.d. | better | worse |
|---|---|---|---|---|
| **i.i.d. (shipped)** | 0.959167 | — | — | — |
| 3/4 decay | 0.957722 | **−0.001445** | 24 | 21 |
| 3/4 n/2 | 0.958089 | −0.001078 | 13 | 23 |
| 3/4 n/2 nochain | 0.958120 | −0.001046 | 13 | 23 |
| 3/4 n/8 | 0.958451 | −0.000716 | 19 | 24 |
| 2/3 decay | 0.958760 | −0.000407 | 33 | 30 |
| 7/8 n/16 | 0.958997 | −0.000169 | 16 | 9 |
| 7/8 decay | 0.959176 | +0.000009 | 15 | 8 |
| 3/4 n/4 | 0.959187 | +0.000021 | 14 | 25 |
| 3/4 decay0 | 0.959438 | +0.000271 | 30 | 20 |
| 3/4 reset | 0.959590 | +0.000423 | 30 | 20 |
| 3/4 n/64 | 0.959793 | +0.000626 | 20 | 21 |
| 3/4 n/16 | 0.959954 | +0.000787 | 19 | 22 |
| 1/2 decay | 0.963738 | +0.004571 | 36 | 47 |
| 1/2 n/2 | 0.964816 | +0.005649 | 16 | 53 |

Two clean structural readings, both consistent with "breadth is what pays":

1. **Split ratio dominates strength.** Anything that keeps ≥3/4 of the budget i.i.d.
   is roughly neutral; 1/2 is a disaster (+0.005). Sacrificing i.i.d. draws is the
   expensive part.
2. **Big perturbations beat small ones**, monotonically: `n/2` > `n/8` > `n/16` >
   `n/64`. Which is to say the best "hill climb" is the one that least resembles a
   hill climb. `NOCHAIN` scoring the same as chained confirms it — the chaining, the
   part that makes it a hill climb at all, contributes nothing.

## Step 3 — the robustness check that kills it

`24 better / 21 worse` for the leading policy is a coin flip, and a geomean over 300
matrices with a heavy tail is exactly where a single instance can carry a whole
"result". So the probe also reports the advantage over i.i.d. on **two disjoint halves
of the corpus** (even/odd position) and with the **single largest-contributing matrix
dropped** from both policies:

| policy | full Δ | half A | half B | Δ with top-1 matrix dropped |
|---|---|---|---|---|
| 3/4 decay | −0.001445 | −0.005750 | **+0.001894** | **+0.000815** |
| 3/4 n/2 | −0.001078 | −0.005439 | **+0.002336** | **+0.001182** |
| 3/4 n/2 nochain | −0.001046 | −0.005439 | +0.002398 | +0.001213 |
| 2/3 decay | −0.000407 | −0.005391 | +0.003317 | +0.001891 |
| 7/8 decay | +0.000009 | −0.000216 | +0.000252 | −0.000135 |

Every policy that looked like a win **flips sign between the two halves** and **loses
to i.i.d. once one matrix is removed**. That matrix is `chp_shorttermplan2d`
(n=16364, restarts=5), where `3/4 decay` finds a relabeling 23.2% cheaper than any of
its i.i.d. draws. It sits in `gt_10k` — 45 matrices at weight 0.40 — so one 23% win
there is worth ≈0.002 of score all by itself, which is larger than the entire
measured advantage. Confirmed in the per-bucket split: `3/4 decay` moves `gt_10k`
0.9537 → 0.9499 and moves `lt_1k`/`1k_10k` by ±0.0007, i.e. nothing.

The policies whose deltas *are* stable across halves (`7/8 decay`, `7/8 decay0`,
`7/8 n/2`) are stable at **±0.0003, i.e. at zero**.

## Conclusion

**At a fixed restart budget, the search policy of the relabelled-AMD multi-start does
not matter.** Uniform i.i.d. sampling is already at this family's ceiling. Shipped
`order()` is therefore left exactly as [0003](0003-relabelled-amd-multistart.md) had
it; the only change committed is `probe_relabel_search`, the `perturb` primitive it
drives (`#[cfg(test)]`, never shipped), and these notes.

The likely reason, and it is worth stating because it predicts other dead ends: the
map from relabeling to AMD flops has **no exploitable local structure**. AMD's
tie-breaking is a global cascade — one different tie-break decision early changes the
degree ordering for everything downstream — so two relabelings differing in a few
transpositions do not produce two similar orderings. There is no gradient to climb.
This makes the family a pure lottery, and the only way to buy a better outcome from a
lottery is more tickets.

## What this implies for the next session

- **Do not** try RCM-seeded or partitioner-seeded relabelings expecting a different
  outcome from the same mechanism. The evidence above is against the mechanism (no
  local structure), not against one particular perturbation.
- The one thing that reliably improves this family is **more restarts** — see the
  budget sweep in [0003](0003-relabelled-amd-multistart.md), which is monotone in
  budget (0.879253 → 0.876925 → 0.876757 → 0.875194 as budget grows). That is a
  **timing** problem, not a search-policy problem, and timing is the binding
  constraint on everything here.
- Use the robustness columns before believing any future result on this corpus. A
  heavy-tailed geomean over 300 matrices will hand you a 0.001 "win" from one
  instance whenever you ask it for one. `dA`/`dB`/`drop1` cost nothing and would
  have saved this session two full end-to-end runs.

## Reproducing

```sh
bash scripts/local-candidate-build.sh          # sandboxed candidate rebuild
cargo test --release -p ssi-candidate-worker --offline --locked -- \
  --ignored --nocapture --test-threads=1 probe_relabel_search
```

Note the package: `src/ordering/` is compiled only into `ssi-candidate-worker`, so
`-p matrices-fast` silently matches zero probe tests. The `memory/index.md` tooling
section gives the command without a `-p` flag, which selects the wrong package on a
current checkout.
