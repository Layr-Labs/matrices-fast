# 0205 — the orphaned four-stream engine as a terminal fan-out

**Status:** shipped, submitted `f647df4d-a58c-41fc-986e-2fb6640a0ad4` (validating).
**Dev:** probe control `0.792166` → `0.791896` (**−2.70e-4**); official sandboxed
harness 300/300, `0.791896 / fill 0.924419`, buckets 0.887431 / 0.838452 / 0.685328.

## The property

`rgreedy::search_par_specs` runs up to four INDEPENDENT streams concurrently and
merges by a strict `(flops, source index)` argmin. Its output is therefore a
pure function of the four `(rng, params, budget)` specs — **the thread count
cannot change it** — and its own doc says why it exists: on the grader's 4
vCPUs four streams cost the wall time of one and buy 4x the search. `grep` of
the production path (mod.rs / probe.rs / the draw) finds **no caller**: every
shipped spend device is a sequential single-stream `rgreedy::search`. That is
the wall time the pipeline paid four times over and collected once.

## Pricing (census seam, today's tip, 37-row engine-affordable class)

| config | wins | dev Δ | corpus add | worst row |
|---|---|---|---|---|
| 1 x 3e7 | 1 | −2.8e-6 | 0.6 s | 0.964 |
| 1 x 3e8 | 6 | −4.5e-5 | 3.4 s | 1.022 |
| 1 x 1.2e9 | 12 | −9.0e-5 | 16.1 s | 1.532 |
| 1 x 2e9 | 14 | −1.6e-4 | 30.6 s | 2.180 (over cap) |
| 4 seeds x 5e8, sequential | 12 | −8.75e-5 | 14.5 s | 1.414 |
| 4 seeds x 5e8, parallel | 12 | −8.75e-5 | **4.0 s** | 1.033 |

The last two rows are one experiment: the same four streams, same merge, same
output — a quarter of the wall clock.

## Shipped shape

One round, 4 streams x 2e9, `Params::DEFAULT`, seeds `0x9E37…`, `0xD1B5…`,
`0xA24B…`, `0x9FB2…`, strict-accept, gated
`n <= 12_000 && (n > 6_000 || nnz > 30_000) && best_flops <= LADDER_FILL_BOUND`
(the census envelope — **35** dev rows fire, not the 263 FANGATE lines, which
count rows inside the terminal window — crossed with the fence's complement).

Measured firing cost: +18.7 s corpus, worst dev row 1.120 → 1.565 s. Movers:
`rsyn0830m04m` 2.46%, `powerflow0300p` 1.44%, `crudeoil_lee2_06` 0.68%, plus
~0.15% on three `qspp_*` rows and small gains on `rsyn0840m04m`, `mpbp_15`,
`pooling_sppa9tp`, `crudeoil_lee4_06`, `mpbp_34`, `qap`, `rsyn0820m04m`.

## Rejected in the same session

* `[5e8,1e9,2e9]` escalation with early stop: stacks +0.03/+0.21/+0.43 s on the
  rows that improve → worst dev row **2.051 s** (past the cap) and, because the
  rounds re-seed and resample the trajectory, SCORE 0.791949 is *worse* than
  the single 2e9 round's 0.791896. Dominated on both axes.
* Every ratio-1.0000 row (`emfl100_3_3`, `squfl020-150`, `squfl015-080persp`,
  `knp5-43/44`, `qapw`, `polygon75`, `watercontamination0303r`, `autocorr_*`)
  bought zero flops from a 2e9 walk and appears as pure price (up to +1.2 s on
  one row): a longer walk on a row no ordering can move is cap exposure only.

## Next

Convert the residual sequential spends to the primitive (the rung ladder's
2e8+1e8x3 is 5e8 ops paid four times in wall clock), and re-price the draw and
the mid-engine stream against this build with `SSI_ENGINE_FANOUT=0`.
