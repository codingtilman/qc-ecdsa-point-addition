| round | tried | toffoli | clifford | qubits | score | accepted |
|---|---|---|---|---|---|---|
| 0 | remove DUMMY_TOFFOLIS harness padding | 3942753 | 22627118.169 | 2715 | 10704574395 | yes |
| 1 | chunked Montgomery x·2^-k (k≤22) replaces 404 single halvings in pair1 | 3858732 | 21722226.718 | 2715 | 10476457380 | yes |
| 2 | chunked Montgomery x·2^k (mirror) replaces 400 single doublings in pair2 | 3773113 (emitted) | n/a | 2715 | n/a (FAIL: phase garbage, 1 batch) | no |
| 3 | round-2 chunked pair2 doubling + pair2 Kaliski iters 400→404 (round-2 failure traced to an input needing exactly 400 steps) | 3791295 | 20906294.456 | 2715 | 10293365925 | yes |
| 4 | Kaliski no-correction r-doubling threshold 262→300 (invariant r,s≤p before termination; P(t≤300)≈1e-9) | 3752535 | 20534880.600 | 2715 | 10188132525 | yes |
| 5 | pow2_scale: chunks up to k=32 and reuse g·c register for the reduction flag (W'=W+(1-F)c) | 3743207 | 20432553.719 | 2715 | 10162807005 | yes |
| 6 | statistical u/v width bound in all Kaliski uv ops: min(exact, ceil(256-0.708i+7(0.36√i+1))); 0 exceedances in 1.2M sampled inversions, ~1e-10 extrapolated | 3128325 | 17735127.677 | 2715 | 8493402375 | yes |
| 7 | statistical r/s width bound (start/end of iteration) for i<300: ceil(1.5+0.708i+7(0.36√i+1)); max K seen 4.3 in 1.2M | 3092261 | 17611474.200 | 2715 | 8395488615 | yes |
| 8 | affine-step Solinas tails (breg red/unred, tx update, y mul): shift22 Bennett pair + 10 halvings → clean chunked 2^22 / 2^-32 | 3062197 | 17461857.726 | 2715 | 8313864855 | yes |
| 9 | chunked 2^22/2^-32 Solinas tail in pair2 Karatsuba mul-add (inside Kaliski body) | 3054681 | 17424457.786 | 2745 | 8385099345 | no (peak +30: chunk transient ~547 vs ~512 for a single halving) |
| 10 | run all 404 Kaliski iterations in bulk form (bulk step is exact after termination: v=0 is the v-even branch); clear the never-toggled f flag in the envelope | 3057902 | 17451755.152 | 2715 | 8302203930 | yes |
