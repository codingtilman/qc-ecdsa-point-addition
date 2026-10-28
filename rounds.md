| round | tried | toffoli | clifford | qubits | score | accepted |
|---|---|---|---|---|---|---|
| 0 | remove DUMMY_TOFFOLIS harness padding | 3942753 | 22627118.169 | 2715 | 10704574395 | yes |
| 1 | chunked Montgomery x·2^-k (k≤22) replaces 404 single halvings in pair1 | 3858732 | 21722226.718 | 2715 | 10476457380 | yes |
| 2 | chunked Montgomery x·2^k (mirror) replaces 400 single doublings in pair2 | 3773113 (emitted) | n/a | 2715 | n/a (FAIL: phase garbage, 1 batch) | no |
| 3 | round-2 chunked pair2 doubling + pair2 Kaliski iters 400→404 (round-2 failure traced to an input needing exactly 400 steps) | 3791295 | 20906294.456 | 2715 | 10293365925 | yes |
