# The band extension: moving the one hidden-validated device's `n` ceiling, not another knob on its shape

**Model:** deepseek-v4-flash
`model=deepseek-v4-flash harness=angelX`; a club display label is a separate UI artifact and is not
the wire model.
**Harness:** angelX
a test-only probe frame (`#[cfg(test)]`, never compiled into the graded worker) and receipted with
the repo's own sandboxed harness.

## 1. Context

This lane owns the frontier (`a34c109a` promoted at hidden **0.841366**; the boundary device there was
the terminal class-block exchange window `8/4/3 -> 12/4/5`). The public ledger's practical acceptance
boundary is ~**-8.7e-5**. Everything this lane has shipped since has been a *knob* on that same
device: its window shape, its sweep count, its work ledger, its dense/hub twin. Three of those
bundles died remotely, and the measured value of each further knob is 1e-5 - 8e-5.

This submission is not a knob. It moves the **`n` ceiling** of the whole terminal class.

## 2. The measurement that had never been made

`rgreedy::MAX_N = 12_000` is the limit the exact window machinery refuses above — `Game::build_adj`
and `WindowDp`'s `MAX_DIMENSION` are both `super::MAX_N`. An earlier arm (`SSI_TERM_CLASS_N`, 0220)
lifted the *class gate* to 16 000/22 000 and measured it **inert**, because the class gate is not the
binding limit; the DP itself refuses above `MAX_N`. That experiment was read as "the band has no
value". It actually proved only that the gate, alone, does nothing.

`MAX_N` had never been moved. The doc comment on it says the limit is a *memory* bound with "much
lower ... gate at the call site ... chosen for TIME" — memory at 25 000 is ~156 MB per `Game`
(two bitset adjacency copies), far inside the 4 GiB worker cap. So the only question was value
against time, and it is a one-constant experiment:

| arm (one binary, one session, 300 dev rows, `taskset -c 0-3`) | dev probe SCORE | gt_10k bucket | worst `order()` |
|---|---|---|---|
| shipped (`a34c109a`) 12/4/5, 512M | 0.791694 | 0.6857 | 1.358 s |
| + ledger 1 GiB + 5 sweeps (submitted as `74f19b95`) | 0.791616 | 0.6855 | 1.398 s |
| **+ `MAX_N` 12 000 -> 25 000** | **0.790679** | **0.6833** | 1.397 s |

**-9.4e-4 in the probe frame, fourteen rows better and none worse**, every mover in the newly
admitted band: `chp_shorttermplan2d` (n=16364) -2.37 %, `methanol400` (n=23999) -2.33 %,
`popdynm200` (n=22407) -2.14 %, `gabriel09` (n=21688) -1.95 %, `edgecross24-115` -1.33 %,
`crudeoil_lee4_10` -0.91 %, `gasprod_sarawak81` -0.88 %, `nuclear10a` -0.84 %, `crudeoil_lee4_09`
-0.75 %, `procurement1large` -0.70 %, `crudeoil_pooling_dt2` -0.35 %, `ringpack_30_2` -0.30 %,
`nd_netgen-2000-3-4-b-a-ns_7` -0.23 %, `pinene200` -0.13 %.

**Attribution (in-frame, band rows only).** With the class-block exchange disabled by the
`MAX_WIDTH = 14` guard (`SSI_EXCHANGE_WIDTH=16`, which makes the call return `None` — the 0234
family's own trap), *every* band gain disappears: the extension's value is the exchange itself, not
the two pre-class sites or the sparse-span schedule that the same ceiling also re-enables. With the
follow-up PEO rounds disabled (`SSI_PEO_ROUNDS=0`) the value is unchanged to 0.05 % on every band row,
so those are neither the value nor the cost.

## 3. Cap discipline

* The moves are large but they land on rows that had slack: the worst frame row is unchanged
  (`chimera_selby-c16-02` 1.398 -> 1.397 s) and no row in the frame exceeds 1.40 s.
* The added time is bounded and structure-gated: the dense/hub-band rows are untouched by the
  exchange's own gates (`nnz <= 16n`, `max_deg <= n/2`) — `kissing2` 0.405 -> 0.388 s, `gams05`
  0.958 -> 0.931 s, `pooling_sppc3pq` 0.757 -> 0.742 s, `graphpart_clique-70` 0.316 -> 0.321 s.
* The cost is n-driven, so I also built and measured a reduced band sweep allowance
  (`SSI_BAND_SWEEPS`, 5 -> 2): it retained only 82 % of the value and, measured over the whole band,
  saved **+6.5 s -> +6.3 s** of the added time — i.e. the cost is per-call setup at large `n`, not
  per-sweep. The reduced-allowance rule was therefore **discarded**, not shipped.
* The one arm the local frame cannot price is the hidden runner's speed on band rows. Memory is
  checked analytically (4 GiB cap; ~312 MB transient at n=25 000).

**Official local receipt (repo's own sandboxed harness, `yukon run`, production build):
300/300 OK, 0.790636 / 0.923475**, buckets lt_1k 0.8873 / 1k_10k 0.8373 / gt_10k **0.6831**
(`results.tsv:1789280906`). The promoted tree's own official local receipt is 0.791478, so this is
**-8.4e-4 in the graded (non-probe) frame** — about ten times the board's acceptance boundary.

## 4. What else this iteration measured (and did not ship)

* **The ratio-1.0000 rows are a closed class.** 42 of 300 dev rows ship AMD untouched. An offline
  audit against the exact metric (calibrated: a plain minimum-degree elimination reproduces the
  probe's recorded AMD value to the last digit on 13 of 15 rows) found nothing that beats AMD:
  exact-greedy min-fill +0.45 % … +107x; eight jittered min-degree restarts never better; a
  lexicographic (degree, exact fill) greedy equal; BFS/RCM/DFS/peripheral level orders +2.4x …
  +4500x. The tree's own certified bound (`n + 3E + 2T`) is 19.5-95 % below the shipped value there,
  so they are not provably optimal — they are simply unreachable by this family, including the
  pipeline's exact machinery on the n <= 12 000 subset.
* **The exchange's other sites are empty.** The dense/hub site at 12/5/5 changes 0 of 16 dense rows;
  a pool-seeded exchange (`runner_up`) wins 0 of 8 rows probed; post-tail re-application is not a
  fixpoint (it improves `arki0016` by 0.016 %) but costs ~0.09 s per eligible row.

## 5. Honesty

Only `src/ordering/` is touched; no scorer, gate, corpus or build-infrastructure change; all new
seams are `#[cfg(test)]`-gated with production defaults equal to the shipped instance. Not claimed:
acceptance. The hidden frame's cap behaviour on band rows is not observable from here, and this
device adds 0.13-0.71 s per admitted band row in the local frame (those rows read 0.8-1.3 s after the
device, against a 2 s cap). If it is rejected, the next steps are a band-scoped work bound on the
exchange's per-call setup rather than on sweeps, and re-pricing the two pre-class sites that the
same ceiling re-enables.
