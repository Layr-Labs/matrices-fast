# 0060: Reverse the terminal four/five cleanup schedule

Date: 2026-09-04 (local). Parent: public promoted frontier ea67ff8.

Hypothesis: running four-pivot descent before five-pivot descent might reach a
better local minimum without increasing the existing two-round work budget.
Only the final cleanup call order changed; gates and acceptance stayed intact.

Verification on Mac Studio, public dev corpus (300 matrices):

- Sandboxed worker build: `bash scripts/local-candidate-build.sh`.
- Trusted scoring: `cargo run --release --offline --locked`.
- Fresh parent: OK, flops 0.843978, fill 0.943905.
- Reversed schedule: OK, flops 0.843979, fill 0.943906.
- Rejected locally; no submission. Restored exact parent mod.rs and rgreedy.rs.

Earlier attempted baselines reused a stale worker and reported 0.845469. Those
are INVALID evidence for the new parent. Touching the synced source files forced
Cargo to recompile; the valid baseline log explicitly reports compilation.
Future rsync transfers must verify a genuine worker rebuild before scoring.

Parent mod.rs SHA256:
fdfaa5a4937bd6d4e6513f940b86f6f6dbd9b91f363d0eb991742c5b40030984

Parent rgreedy.rs SHA256:
921d68ef4b8bbf5e8732b6a9719feb8a89398f0022b61b595896c8501ea1799f

Candidate mod.rs SHA256:
89fe76eb211542ab7f140e9a5bd2cdcd52e3d03d605fa0b990d8a1a7ec60a995

Local evidence logs: /private/tmp/matrices-fresh-frontier-baseline.log and
/private/tmp/matrices-reversed-cleanup.log. Original pre-frontier ordering backup:
/private/tmp/src-ordering-snapshot-20260904-232955.tar.gz. Existing unrelated
dirty files were preserved. No claim of hidden-eval timing safety or new rank.

Spark assisted investigation; the main agent reviewed and executed this final
experiment. Next work should target an algorithmic improvement, not this schedule.
