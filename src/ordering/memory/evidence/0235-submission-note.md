# The exchange's own ledger, priced on the tree that ships it — plus the "dead rows" the lane never audited

**Model:** deepseek-v4-flash
`model=deepseek-v4-flash harness=angelX`; the club display label is a separate UI artifact and is
not the wire model.
**Harness:** angelX
test-only probe frame (`#[cfg(test)]`, never compiled into the graded worker) and the repo's own
sandboxed harness for the official receipt.

## 1. Where this lane stood

The lane owns the frontier: `a34c109a` promoted at hidden **0.841366** (the boundary device there
was the terminal class-block exchange window `8/4/3 -> 12/4/5`). The public ledger (`yukon
submissions --all`, 656 receipts) puts the practical acceptance boundary at roughly **-8.7e-5**:
`723bf79` at -0.0000870 promoted while -0.0000790, -0.000077, -0.000073 were rejected. So a
submission has to be worth about a bip against the *hidden* frame, and this lane's clean
dev→hidden transfer ratios on this base have measured 1.16x and 1.55x.

This iteration did no new heuristic design first. It audited two things the lane had never looked
at, and shipped the one device that survived the audit.

## 2. New audit A — the ratio-1.0000 rows ("dead rows") are a class, and it is closed to cheap rules

42 of the 300 public dev rows score a ratio of **exactly 1.0000** against AMD: the pipeline ships
AMD's ordering untouched. Four of them are in the gt_10k bucket (weight 0.40), among them
`supplychainr1_053050` (n=16640), `emfl100_5_5` (n=21925), `squfl030-150` (n=13680) and `kissing2`
(n=20772); the rest are mostly in lt_1k / 1k_10k. Several are *slow* rows (0.4-0.9 s of `order()`)
that return no value at all.

I audited 15 of them (n = 601 ... 21 925) outside the tree, in Python, against the **exact**
metric the grader ranks (`SCORE = sum_j c_j^2`, c_j = nnz of column j of L, diagonal included).
The metric implementation is calibrated against the probe's own record: a plain minimum-degree
elimination in Python reproduces the probe's recorded AMD value to the last digit on 13 of these
15 rows, and is *worse* than AMD on `hydroenergy1`, so the frame is sensitive, not degenerate.

Constructions tried, all structure-only, all compared to the recorded AMD value:

| construction | result on the 15 dead rows |
|---|---|
| exact-greedy **min-fill** (the tree's own primitive) | worse by **+0.45 % to +107x** |
| plain min-degree (reproduces AMD) | equal on 13, +5.8 % on `hydroenergy1` |
| 8 jittered min-degree **restarts** | never better; equal or +7.2 % |
| lexicographic **(degree, exact fill)** greedy | equal on 13, +1.5 % on `hydroenergy1` |
| **BFS / RCM / DFS / peripheral level orders** | worse by **+2.4x to +4500x** |
| reversed orders | worse by **+1000x to +1.7e6x** |

The lower bound the tree itself certifies (`SCORE >= n + 3E + 2T`, equality iff fill-free) is
**19.5 % - 95 % below** the shipped value on these rows, so "provably optimal" is *not* available
here — the rows carry real, unavoidable fill. But the class is closed to every cheap structure-only
rule in this family, and the pipeline's own exact machinery (which does cover n <= 12 000, including
`squfl015-060` at n=2775 and `squfl030-150` at n=13680) finds nothing there either. Conclusion:
the 14 % of the corpus that scores 1.0000 is not a reachable headroom; it is a floor. I did **not**
ship anything for it.

## 3. New audit B — the exchange is not a fixpoint, and its other sites are empty

The one device that is validated on the hidden frame is the class-block exchange window. I added
two test-only seams and measured its *unpriced* axes and sites (one binary, one session,
`taskset -c 0-3`, 300 dev rows):

* **The dense/hub site** (`nnz > 16n || max_deg > n/2`) still calls the old `8/4/3` shape. Widening
  it to `12/5/5` changes **0 of 16** dense rows' output and only costs +0.05 s on the worst row.
* **Pool-seeded exchange**: re-running the exact window descent from the displaced orderings the
  pipeline already retains (`runner_up`) wins **0 of 8** rows probed. Pure cost.
* **Post-tail re-application**: the move is *not* a fixpoint — re-applying it once to whatever the
  terminal ladder + fanout + banked walks left improves `arki0016` (835794 -> 835661), but the win
  is 0.016 % on one row and costs ~0.09 s per pass on every eligible row. Rejected on value/time.

Both seams default to the shipped behaviour in a production build (`cfg(not(test))`), so the graded
worker is unchanged by them.

## 4. What is actually shipped: the exchange ledger 512M -> 1G and its sweeps 4 -> 5

Neither half of this pair was ever priced *on this tree*, and the pair has never been submitted
without the 13-width sparse-span schedule that died remotely three times. In-frame, one binary, one
session, 300 dev rows:

| arm | dev probe SCORE | worst `order()` |
|---|---|---|
| P — as promoted (`a34c109a`): 12/4/5, 512M | 0.791694 | 1.358 s |
| L1G — ledger 1 GiB | 0.791647 (-4.7e-5) | 1.366 s |
| L2G — ledger 2 GiB | 0.791641 (-5.3e-5) | 1.430 s |
| **L55 — ledger 1 GiB + sweeps 4 -> 5** | **0.791616 (-7.8e-5)** | 1.398 s |

The ledger step alone moves 5 rows with **0 worse** (`crudeoil_lee4_06` -0.52 %, `powerflow0300p`
-0.21 %); the fifth sweep adds 8 more movers (`crudeoil_lee2_06` -0.70 %, `chimera_selby-c16-01`
-0.39 %) and costs back 0.053 % on `rsyn0810m02hfsg` and 0.034 % on `glider400`. 6 and 8 sweeps
measured identical (1e-6) on the same base, so 5 is the saturation point. The added time is spent
on *exactly* the rows that move (`chimera_selby-c16-02` +0.040 s, `crudeoil_lee4_06` +0.069 s) and
not on the dense rows the same ledger is shared with (`graphpart_clique-70` +0.001 s, `qapw`
-0.004 s). Every install site keeps its strict exact decrease, so a larger ledger can only trade
time for value on the row it fires on.

**Official local receipt (repo's own sandboxed harness, `yukon run`, production build):**
**300/300 OK, 0.791399 / 0.924065**, buckets lt_1k 0.8873 / 1k_10k 0.8373 / gt_10k 0.6851
(`results.tsv:1789279537`). The promoted tree's own official local receipt was 0.791478, i.e. this
device is **-7.9e-5 in the graded (non-probe) frame**, matching the probe frame's -7.8e-5.

## 5. Correctness and honesty

* Only `src/ordering/` is touched. No scorer, purity gate, corpus, test or build-infrastructure
  change; the candidate builds and runs through the repo's own sandboxed harness.
* Every new code path is `#[cfg(test)]`-gated with production defaults equal to the previous
  promoted instance, except the two constants shipped here.
* Not claimed: acceptance. The hidden frame's cap margin is not observable from here; the pair is
  deliberately shipped *without* the span-schedule extension whose three remote kills may or may
  not have been caused by the ledger half. Two dev rows pay a small regression (+0.053 %, +0.034 %)
  for the eleven that improve. If the hidden delta lands short of the board's acceptance boundary in
  the opposite direction, the next bat is the ledger step alone (5 movers / 0 worse).
