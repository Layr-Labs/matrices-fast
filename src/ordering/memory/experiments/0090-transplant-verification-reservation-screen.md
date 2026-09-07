# Reserving verification work in a 100k transplant ledger

Control: exact production `996e8d6`, eight retained donors in their production
order. Test-only source: `c9812ce`. This is a terminal scoring probe and makes
no production change. It compares the original donor-loop truncation with a
policy that reserves a verification score before trying another donor.

Both arms use widths 4096/512/128/32, minimum block size four, `n >= 16`, a
nonempty donor pool, and `4(n + nnz) <= 100000`. The original loop abandons
the width when another donor cannot fit, discarding any unverified partial
gains. The reservation arm stops the donor loop and verifies those gains.

| policy | paid rows | winning rows | dev score | absolute dev bips |
|---|---:|---:|---:|---:|
| original truncation | 196 | 13 | 0.8265378797435275 | 0.20340032 |
| reserve verification | 196 | 26 | 0.8264650805206859 | 0.93139255 |
| reservation, only below raw AMD | 152 | 26 | 0.8264650805206859 | 0.93139255 |

Baseline: 0.826558219775327. All 300 AMD/incumbent count pairs match
[0089](0089-production-core-stage6-screen.md) exactly. No `gt_10k` row changes;
the reservation increment over the original is in `1k_10k`. The below-AMD
condition removes 44 paid rows without losing a winner on this corpus.

The predeclared below-AMD screen required at least 15 winners, three absolute
dev bips, and fewer admitted rows. Reservation meets the first and third but
fails the score floor. **This 100k treatment is not implemented or submitted.**

The sweep measures 152.253 ms total / 2.394 ms maximum for the original and
137.510 ms total / 2.143 ms maximum for reservation, including setup and
verification. These same-pass exploratory timings are cache/order sensitive;
they are not interleaved production qualification. The ledger counts scorer
calls, not every rank/sort/allocation operation. Maximum charged units stay
below 100k in both arms. A below-AMD score is not evidence of runtime slack.

All assembled proposals are bijections and satisfy exact per-block contribution
checks. The full screen passed in 153.15 s. A larger budget or different donor
selection would be a new experiment, not a validated extension of this result.
