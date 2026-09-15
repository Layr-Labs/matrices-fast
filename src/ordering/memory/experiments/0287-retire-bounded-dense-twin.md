# 0287 — retire the bounded dense twin after fork-off still caps

- **Date:** 2026-09-15 (iter79)
- **Base:** iter78 `938598a` / submission `782a26d9` / PR #744.
- **Change:** dense/hub exact-window schedule returns from the bounded
  `10/3/4` arm to the promoted `8/4/3` shape at every size.
- **Public score:** derived from complete fork-off records and the measured
  dense schedule arms: `0.790255990203 -> 0.790253862267` (**-2.128e-6**).
- **Status:** submitted as `26f00f80` / PR #745 and cap-failed after about
  84.360 seconds of Benchmark execution.

## Hidden attribution after the fork endpoint

Iter78 compiled the shared-basin fork out of production. It was the strongest
causal cap test available: the fork had been the only device that could add a
second terminal suffix on small rows, and every composition carrying it had
failed. The exact fork-off tree was submitted as
`782a26d9-ea6a-4b85-936f-39f329526da9` / PR #744.

It still failed the two-second hidden cap. Benchmark began at
`14:36:39.432027Z` and reported the kill at `14:38:04.113147Z`, about **84.681
seconds** later. Iter77, with the fork still admitted below an 80%-of-AMD
checkpoint, failed at 84.509 seconds. Removing the fork did not materially move
the hidden failure position, so the binding row on this daily corpus does not
use that device.

The remaining composed search device absent from the last completing iter72
tree is the dense/hub twin. Its production shape is wider than the promoted
source only for `1,000 <= n <= 5,205`: width 10, three sweeps, step 4, versus
the promoted width 8, four sweeps, step 3. Width dominates the exact subset DP
cost; a ten-vertex component has four times as many subsets as an eight-vertex
component before accounting for the sweep count.

## Existing schedule curve

The dense site was measured in one binary through `SSI_DENSE_W`,
`SSI_DENSE_S`, and `SSI_DENSE_T`. The three public movers returned:

| schedule | c16 Chimera | `chimera_rfr-02` | `chimera_lga-01` |
|---|---:|---:|---:|
| 10/4/4 | 2,560,253 | 2,487,272 | 492,146 |
| 10/3/4, iter78 | 2,560,265 | 2,501,621 | 494,718 |
| 8/4/3, promoted | 2,570,031 | **2,489,836** | 494,718 |

Returning to 8/4/3 gives back 9,766 flops on the c16 row, improves the rfr row
by 11,785 flops, and leaves the lga row unchanged relative to iter78. The two
changed rows are in the same `1k_10k` score bucket. Their opposing log-ratio
deltas nearly cancel, with the rfr improvement slightly larger.

## Exact public score

Iter78's complete fork-off arm contains all 300 returned counts and scores
`0.790255990203`. Replacing only the two changed dense records with their
measured 8/4/3 values gives:

```
lt_1k    count=147   geomean=0.887274030819
1k_10k   count=108   geomean=0.837322865453
gt_10k   count=45    geomean=0.682186983464
SCORE = 0.790253862267
```

Thus retiring the wider twin is not a score sacrifice on the current public
composition. It improves the aggregate by **2.127936e-6** while removing the
width-10 exponential schedule. No other public row changed between the dense
arms in the prior full census, and the fork is already off, so there is no
overlap between the retired devices.

The candidate remains `+6.33e-5` behind the promoted source's public provenance
`0.790190562749`; that difference is mainly the public value of the now-retired
fork. This iteration is still cap-first. Its purpose is to restore the two
search topologies absent from the last completing tree while preserving all
output-invariant representation improvements.

## Implementation

The dense/hub site keeps its structural admission:

```
nnz > 16*n || max_degree > n/2
```

It also keeps the exact score check, bijection check, shared exchange ledger,
follow-up fill gate, and PEO rounds. Only the default tuple changes. Production
now uses `(8, 4, 3)` at every size. Test builds retain `SSI_DENSE_W`,
`SSI_DENSE_S`, and `SSI_DENSE_T`, so the retired width-10 arms remain
reproducible without changing submitted behavior.

The obsolete `DENSE_TWIN_WIDE_MIN_N` and `DENSE_TWIN_WIDE_MAX_N` constants and
the dimension branch are removed. This makes the submitted control flow match
the promoted dense site directly and ensures no hidden row can enter the
width-10 DP through this block.

## Final verification

The production candidate was rebuilt through `scripts/local-candidate-build.sh`
with the repository's explicit local unsandboxed-worker opt-out, and the trusted
parent release build completed offline and locked. The complete release suite
then passed:

```
test result: ok. 127 passed; 0 failed; 56 ignored
```

A final focused run from that exact release test binary reproduced the promoted
8/4/3 records: c16 Chimera **2,570,031**, `chimera_rfr-02` **2,489,836**, and
`chimera_lga-01` **494,718**. `git diff --check` also passed. These are the same
records used in the exact full-corpus score derivation above.

## Hidden receipt

The exact tree was submitted as
`26f00f80-4907-404f-a9e0-d3f4bf508c53`, opening PR #745 and workflow
`34984482002`. Benchmark began at `14:55:14.1192976Z` and reported the hidden
two-second cap failure at `14:56:38.4794833Z`, about **84.360 seconds** later.
That again matches iter77 and iter78. Removing both newly composed search
devices has therefore left the binding failure unchanged.

An exact source comparison with the promoted base then exposed global residue
from the retired fork: the complete suffix remained wrapped in the fork's
closure and used converted scorer/runner-up ownership on every row even when
the production admission function returned constant false. Iter80 removes that
scaffolding and restores the promoted direct suffix control flow; see
[0288](0288-restore-direct-suffix-control-flow.md).

## What remains different from the completing topology

Iter79 retains several changes that either come from the promoted PR #719 tree
or remove output-invariant work:

- the one-image sparse-pristine representation and its one-GiB resource law;
- first-reset reuse of the adjacency already built from CSR;
- retention of one sparse mutable image between sequential exact-window calls;
- `XCH_ALLOC=1`, which skips an unfundable component and continues the window;
- the resource-law class admission and restored `PEO_ALT_MAX_N=50,000`.

The first three reduce allocation, reconstruction, or first-touch work without
changing logical charges or returned counts. `XCH_ALLOC=1` has one small public
mover and completed in another lane's score-neutral hidden tree. The resource
law itself completed in iter72. If iter79 still fails at the same hidden
position, neither newly composed search device owns the row; the next work must
return to global pipeline cost or a shared terminal stage rather than further
fork/twin gating.

## Evidence

- `.session-backup/iter77-fork-off-full.log`
- `.session-backup/iter76-dense-s3-full.log`
- dense schedule curve in
  [0284](0284-cap-trim-dense-sweep-and-fork-margin.md)
- iter78 hidden receipt: PR #744 / workflow `34982397976`
- iter79 hidden receipt: PR #745 / workflow `34984482002`

## Links

- [0286 retire the basin fork](0286-retire-basin-fork.md)
- [0284 dense sweep and fork margin](0284-cap-trim-dense-sweep-and-fork-margin.md)
- [0276 shared-prefix fork and bounded twin](0276-shared-prefix-fork-and-bounded-twin.md)
- [0288 restore direct suffix control flow](0288-restore-direct-suffix-control-flow.md)
