| round | tried | toffoli | clifford | qubits | score | accepted |
|---|---|---|---|---|---|---|
| 0 | remove DUMMY_TOFFOLIS harness padding | 3942753 | 22627118.169 | 2715 | 10704574395 | yes |
| 1 | chunked Montgomery x·2^-k (k≤22) replaces 404 single halvings in pair1 | 3858732 | 21722226.718 | 2715 | 10476457380 | yes |
| 2 | chunked Montgomery x·2^k (mirror) replaces 400 single doublings in pair2 | 3773113 (emitted) | n/a | 2715 | n/a (FAIL: phase garbage, 1 batch) | no |
| 3 | round-2 chunked pair2 doubling + pair2 Kaliski iters 400→404 (round-2 failure traced to an input needing exactly 400 steps) | 3791295 | 20906294.456 | 2715 | 10293365925 | yes |
| 4 | Kaliski no-correction r-doubling threshold 262→300 (invariant r,s≤p before termination; P(t≤300)≈1e-9) | 3752535 | 20534880.600 | 2715 | 10188132525 | yes |
| 5 | pow2_scale: chunks up to k=32 and reuse g·c register for the reduction flag (W'=W+(1-F)c) | 3743207 | 20432553.719 | 2715 | 10162807005 | yes |
