# 0101 — METIS densify n<3k

**Score:** 0.806243 → **0.805951** (−2.92 bip)
**Worst:** 1.109 s (`crudeoil_lee1_07`)
**Change:** Under `part_extra2 && METIS_VAR`, when `n < 3_000`: extra `nd_to_amd_switch` ∈ {50,150,300,800} + `max_imbalance=0.15`. Tip EXTRA_METRICS kept (2 specs × α{10,5,1} on n<10k). Medium tickets tip-identical.

**Finding:** nuclear25a (n=1942) 0.635→0.561 is this METIS densify, not MinFill / EXTRA densify.

**Submit:** no — worst at parent ceiling 1.05–1.10s.
