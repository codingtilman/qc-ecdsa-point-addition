# Run 3 (nov02) rounds

| round | tried | toffoli | clifford | qubits | score | accepted | exact/probabilistic (failure prob.) |
|---|---|---|---|---|---|---|---|
| 0 | baseline (harness padding removed) | 3,942,753 | 22627118.169 | 2715 | 10,704,574,395 | yes (baseline) | exact apart from pre-existing approximations (see notes) |
| 1 | 22-bit Montgomery-style halve/double for the 404 halvings / 400 doublings | 3,755,663 | 20635056.952 | 2715 | 10,196,625,045 | yes | probabilistic: wrong when v < M·c, ≤ 2^-201 per 22-bit round (~2^-194 per shot); concrete failing input v = 1 |
| 2 | same k-bit rounds (low-qubit) in the multiplier Solinas reductions | 3,705,485 | 20226877.139 | 2715 | 10,060,391,775 | yes | probabilistic: same flaw as round 1, ≤ 2^-201 per round |
| 3 | lazy-frame Kaliski: drop step-9 (u,v_w) swap-back, recover relative swap via r0&(sigma\|add) | 3,339,205 | 19483639.996 | 2715 | 9,065,941,575 | yes | exact |
| 4 | lazy (r,s) frame too: persistent sigma qubit, no step-9 cswaps | 3,057,097 | 18914613.250 | 2715 | 8,300,018,355 | yes | exact |
| 5 | half-cost squaring via controlled add-subtract rows + closed-form correction | 2,993,349 | 18894493.853 | 2715 | 8,126,942,535 | yes | exact |
| 6 | peak cut: carry-only sparse-constant adder, carry-free 3h Karatsuba z1 add, tmp-free Kaliski step 4 | 3,002,135 | 24436291.697 | 2588 | 7,769,525,380 | yes | exact |
| 7 | in-place Karatsuba half sums (no x_sum/y_sum registers) | 3,002,135 | 24433233.335 | 2462 | 7,391,256,370 | yes | exact |
| 8 | lazy Kaliski history, release provably-zero (u,v_w) top bits, drop step 0 / termination flag, schoolbook body multiplies | 3,044,210 | 24980691.397 | 2314 | 7,044,301,940 | yes | exact (post-termination state (1,0) evolves identically without the flag; bitlen(u)+bitlen(v) ≤ 2n−i holds for all inputs) |
| 9 | r no-reduction threshold 262→300, u/v width 2n−i−1, late history allocation | 3,003,114 | 24819063.587 | 2312 | 6,943,199,568 | yes (best) | probabilistic: threshold fails if an inversion terminates in ≤ 300 steps; 0/480k samples ≤ 310 (95% upper bound < 6e-6 per inversion, likely far smaller). Width trim and late allocation are exact |
| 10 | skip iteration-0 comparator; r threshold 310 | 2,991,886 | 24772810.095 | 2312 | — (validation failed) | no (crash) | probabilistic (threshold 310; comparator skip exact). Failed on phase garbage in 2 shots caused by the pre-existing pair2 400-iteration cap (k = 402), not by the change |
| exact | exact-only variant of round 9 (branch autoresearch/nov02-exact, a22604f): borrow-kept k-bit rounds, threshold 255 | 3,085,082 | 25166115.262 | 2312 | 7,132,709,584 | n/a (separate branch) | exact (unit-tested on adversarial inputs, incl. v = 1) |

Toffoli, Clifford and qubits are from `results.tsv` (average executed gates per shot). Score = round(Toffoli) × qubits.

"Exact" means correct for every input whose two inversions terminate within the iteration caps. These pre-existing approximations from the original code remain in every row:
- the Kaliski iteration caps (pair1 404, pair2 400) miss about 6e-6 and 2.3e-5 of random inversions (measured over 480k samples), so roughly 1 in 4–5 circuit changes fails validation by bad luck;
- the original single-bit modular halve/double (used for r at iterations ≥ the threshold) is wrong on a ~2^-223 slice of inputs;
- the baseline's r threshold of 262 already sits past the proven bound of 255.
