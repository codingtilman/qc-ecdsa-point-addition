# qc-ecdsa-point-addition

Experiments on optimizing a quantum circuit for ECDSA (secp256k1) elliptic-curve point addition: reducing Toffoli count, Clifford count and qubit peak. Each experiment lives on its own branch; this `main` branch only holds this overview.

| Branch | Source | Contents |
|---|---|---|
| [`partA-P1`](../../tree/partA-P1) | `ecdsa-clean` / `master` | Part A, phase 1: optimization rounds 1–10 (Solinas reduction tails, Kaliski inversion width bounds, all-bulk Kaliski iterations). Round notes in `rounds.md`, harness results in `results.tsv`. |
| [`partA-P2`](../../tree/partA-P2) | `ecdsa-run3` / `autoresearch/nov02` | Part A, phase 2 (autoresearch run 3): lazy-frame Kaliski, add-subtract squaring, Karatsuba/peak-qubit cuts. Includes the run program (`program-run3.md`), round notes (`rounds.md`), the autoresearch log (`autoresearch.tsv`), harness results (`results.tsv`) and `.claude/settings.json`. |
| [`partA-P2-exact`](../../tree/partA-P2-exact) | `ecdsa-run3` / `autoresearch/nov02-exact` | Variant of P2 that uses only exact arithmetic: exact k-bit halve/double rounds, exact `r` threshold 255, plus a unit test. **The logs and notes for this run (`rounds.md`, `autoresearch.tsv`, `results.tsv`, `program-run3.md`) are on `partA-P2`.** |

Generated files such as `ops.bin` and `target/` are not included.
