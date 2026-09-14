# Production-core stage-6 screen

Control: exact promoted `996e8d6e24672636ec94ef5795cf77dccafe062f`, with eight
alternate seeds, promoted capture paths, and robust-AMD giant gates. The
failed transplant and local hygiene/refactor are absent. Only `cfg(test)`
instrumentation is added, preserved in source commit `ec25101`.

The probe captures the core ordering production selects before splicing,
applies the same 512/1024 move streams under a 10M starting-fill estimate,
and compares fixed-prefix-plus-polished-core flops with the final production
ordering. The screen was set at six final winning rows before measurement.

| measurement | result |
|---|---:|
| original rows with eligible captured cores | 117 |
| eligible outside the raw stage-6 gate | 77 |
| original rows receiving proposals | 88 |
| paid outside the raw gate | 48 |
| captured cores receiving proposals | 190 |
| captured cores improved | 98 |
| final winning rows | **2** |
| production score | 0.826558219775327 |
| hypothetical splice score | 0.8265508817825582 |
| absolute dev gain | **0.07337993 bips** |

Winners: `ex1265a`, 3750 → 3736 flops, and `p_ball_10b_5p_4d_m`,
13292 → 13288. Both lie in `lt_1k`. The other buckets are unchanged.
Most improved cores lose to other orderings already found by production.
The treatment fails the six-row screen and is not implemented in production.

The hook captures K=3 and extra reduction depths. Ten million estimated units
per capture is not a per-original-row allowance: 13 rows exceed 10M total,
up to 18,742,272; at most four captures are paid per row. Starting fill is
not a maximum over proposed orderings, so this estimate is not a proved work
bound. The single pinned sweep reports 509.883 ms total added core work and
17.159 ms maximum including setup/checks; this is not a timing qualification.

The exact control geomeans are 0.8897304931511659 / 0.8630890157200626 /
0.751780917784896 over 147/108/45 rows. The probe's SmallScore results are
checked against the symbolic scorer. It passed all 300 rows in 152.29 s.

This closes the specified polish screen, not every core portfolio. The offline
`blend718` surrogate win is chiefly a portfolio improvement, and this 10M
screen pays no proposal sets on that row. Post-hoc splicing also does not
predict every changed mid-pipeline trajectory.
