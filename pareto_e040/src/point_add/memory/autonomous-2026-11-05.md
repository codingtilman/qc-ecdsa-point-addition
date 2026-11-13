# Autonomous optimization checkpoint — 2026-11-05 UTC

## Preserved baseline

- Parent: `667ce8d4d8774be3edde6a8c9777bb04a1876bf8`
- Q: 1,243
- Mean executed Toffoli: 721,156.073; rounded: 721,156
- Score: 896,396,908
- Official local validation: 9,024/9,024; classical/phase/ancilla failures 0/0/0
- `ops.bin` SHA-256: `1c542d401ededcbceaed6596172aaa35d29a8a9c6c20274ee16e81f9a74fcebb`
- Original checkout had only the evaluator-appended `results.tsv` modification; experiments use branch `research/autonomous-20261105` in a separate worktree.

## Resource profile

The build's phase report gives expected executed Toffoli by major phase:

- divide: 340,393.0, peak Q=1,243
- multiply: 345,228.2, peak Q=1,243
- square: 33,888.0, peak Q=516
- coordinate subtraction shells: 317.5 each

The first global peak is op 1,473,360 in `divide`. An observational peak census reproduced the baseline hash and found these 1,243 live-wire owners:

- 256 target-y wires (`mod.rs:266`)
- 193 surviving target-x wires (`mod.rs:265`)
- 256 payload copy wires (`leapfrog.rs:6867`)
- 142 carry/fold wires (`leapfrog.rs:2243`)
- 108 tick-trio/record wires (`leapfrog.rs:4262`)
- 96 + 96 seed-multiplication wires (`seed_mult.rs:16-17`)
- 37 + 36 sign/tick wires (`leapfrog.rs:4291`, `leapfrog.rs:4243`)
- 11 + 10 + 1 A2 record-field wires (`a2_rails.rs:240,248,251`)
- 1 split carry (`leapfrog.rs:1307`)

Both walks reach Q=1,243, so a width improvement must remove a live wire at every binding site rather than only the first divide peak.

Major traced Toffoli components include 198,720 T in 735 `chunked_add` sites, 17,953.5 T in 2,021 `cmp_lt_phase` sites, 117,901 T in forward Y15 payload bodies, and 175,320.5 T in reverse A2 ticks. WL_CODEC costs about 845 T over both walks.

The ordinary WL_CODEC legal domain has `26^4 * 2^12 = 1,871,773,696` states. It exceeds `2^30` and fits `2^31`, proving that a standalone exact 31-to-30 wire codec is impossible without extra correlated side information or a smaller reachable-domain proof.

## Ranked exact-preserving backlog

1. Peak-lifetime surgery across both walks. Target an A2 record/read scratch or recomputable carry live at all Q=1,243 sites. Estimated delta Q=-1, delta T=0..+300, score improvement 721,156..348,556. Correctness requires no intervening read, clean release/rebuild, phase equality, and independent validation of any planner changes. Main failure: a second peak remains or the Q=1,242 planner cliff changes approximate repairs.
2. Resynthesize `a2_code_read` / erase on its legal alphabet. Estimated delta Q=0 (possibly -1 transient), delta T=-40..-160, score improvement 49,720..198,880. Correctness requires exhaustive legal-state, measurement-branch, phase, and cleanup checks. Main failure: the three-Toffoli read is already locally minimal.
3. Resynthesize WL_CODEC commit/pop without changing its 32-to-31 interface. Estimated delta Q=0, delta T=-16..-64, score improvement 19,888..79,552. Correctness requires exhaustive replay of 456,976 high-word states, phase-coin cancellation, cleanup, and integrated paired evaluation. Main failure: current network is Toffoli-minimal.
4. Extend exact product holds/loans. Baseline found 333 repeated-product candidates but placed only five after 11,565 duplicate loans. Estimated delta Q=0, delta T=-1..-10. Major acceptance requires an independent exact-expression proof rather than relying only on the pass's 128-bit hash.

## Experiment E001 — post-veto exact rescan

- Timestamp: 2026-11-05 UTC
- Parent: `667ce8d4d8774be3edde6a8c9777bb04a1876bf8`
- Candidate revision: uncommitted isolated patch; reverted after decision
- Hypothesis: `last_veto` may expose new identities not seen by the earlier proof passes.
- Change: after `last_veto`, rerun `simplify_products`, `affine_simplify`, and `truth_simplify` before `hold_pass`.
- Affected function: `lane_generic_policy` in `src/point_add/mod.rs`
- Cheap correctness: all added passes are existing exact, measurement-free passes.
- Result: emitted 12,338,173 ops, byte-identical to parent.
- Q/T/score: 1,243 / 721,156.073 / 896,396,908 (inherited because stream is identical)
- Delta Q/T/score: 0 / 0 / 0
- Ops hash: `1c542d401ededcbceaed6596172aaa35d29a8a9c6c20274ee16e81f9a74fcebb`
- Correctness evidence: exact operation-stream identity to the verified baseline.
- Official validation: not rerun; baseline's verified stream is identical.
- Runtime: about 7 minutes including incremental release compilation; about 5 minutes 40 seconds generation.
- Compute: one CPU-heavy build, peak RSS about 4.5 GiB.
- Decision: `REJECTED` — no resource change.
- Next: focused literature/GitHub checkpoint, then the smallest locally exhaustive A2 read/erase experiment.

## Disproven / closed directions

- Post-`last_veto` product/affine/truth rescan: byte-identical, no savings.
- Standalone exact WL_CODEC 31-to-30 compression over the documented legal domain: information-theoretically impossible.

## Focused external checkpoint

The review was intentionally limited to ideas that map to a measurable local experiment.

- Long et al., *Efficient Record-and-Replay Arithmetic for Quantum Elliptic-Curve Point Addition* (arXiv:2609.28882) formalizes the same comparison-free ping-pong and fused replay family already present here. Its isolated fused inverse-replay cell trades one workspace qubit for 53 static Toffolis, and its measurement-based cleanup is already represented in this tree. It supports the local carry/replay focus but does not supply an unimplemented drop-in improvement. <https://arxiv.org/abs/2609.28882>
- Luo et al., *Space-Efficient Point Addition* (arXiv:2607.13816) uses length registers and shared packed EEA workspaces. The public implementation comparison reports about 69.5M Toffolis and 838 qubits for the point-add circuit, so it is not competitive with 721k by the product metric. Its register-sharing principle is already reflected in this tree's shrinking rails, loans, and record packing. <https://arxiv.org/abs/2607.13816>
- Gu et al., *Discovering state-of-the-art quantum circuits with large language models* (arXiv:2610.22367) introduces algebraic branch decoding and predetermined sharing between shrinking integer rows and growing coefficient rows. The public certified implementation's best product point is about 4.0M Toffolis and 935 qubits, still far worse than the incumbent. The potentially transferable local idea is to test whether surviving arithmetic state provides side information for A2 history; this could evade the standalone 31-to-30 codec bound, but only after a conditioned reachable-state census and exact invariant proof. <https://arxiv.org/abs/2610.22367> and <https://github.com/andigu/uncompute/tree/main/tasks/modinv-point-add/circuits/divsteps>
- Oumarou et al., *Halving the width of Toffoli based constant modular addition to n+3 qubits* is not aimed at the binding phase here: coordinate/constant shells cost only hundreds of expected T and do not set Q=1,243. <https://arxiv.org/abs/2102.03615>
- Meuli et al., *Reversible Pebbling Game for Quantum Memory Management* suggests SAT scheduling for local space/time tradeoffs. This implementation already has extensive hand loans, recomputation, and cleanup, and the generic lifetime pass placed only five holds. A whole-stream pebbling rewrite is therefore deprioritized; a bounded DAG around one binding A2/carry interval remains a later feasibility probe. <https://arxiv.org/abs/1904.02121>
- Meuli et al., *The Role of Multiplicative Complexity in Compiling Low T-count Oracle Circuits* supports treating a local legal-alphabet Boolean network as an AND/XOR synthesis problem. This directly motivates exhaustive/symbolic resynthesis of `a2_code_read` and its erase rather than an arithmetic-stack rewrite. <https://arxiv.org/abs/1908.01609>

Decision: retain the existing algorithm. Next execute a local A2 read/erase truth-table and synthesis audit. Keep algebraic side-information decoding and bounded SAT pebbling behind it in the backlog.

## Experiment E002 — A2 joint read multiplicative complexity

- Timestamp: 2026-11-05 UTC
- Parent: `667ce8d4d8774be3edde6a8c9777bb04a1876bf8`
- Candidate revision: analysis-only; no submitted-source change
- Hypothesis: jointly synthesize `(k3,k2)` from the five-bit A2 block word with two ANDs instead of the current three-AND interface.
- Affected functions if successful: `a2_code_read` and `a2_code_read_erase` in `src/point_add/a2_rails.rs`.
- Exact domain derivation: frozen `EMAX=5` implies `k3 => !k2`; the per-rail family rule permits at most two `k3` events in a three-position block. Thus the legal alphabet has `2^3 + 3*2^2 + 3*2 = 26` states. A classical transcription of `a2_code_commit` maps these injectively to 26 five-bit stored words, and the existing decoder returns every original logical state.
- Decisive test: exhaustively enumerate all XOR-AND networks with a first AND of two affine input forms, a second AND of two affine forms of inputs plus the first product, and arbitrary affine output forms. Truth functions were represented as 26-bit vectors; 684 distinct first-product functions were checked for each of the three positions.
- Result: no two-AND network computes both `(k3,k2)` at position 1, 2, or 3 on the complete legal alphabet.
- Q/T/score: unchanged at 1,243 / 721,156.073 / 896,396,908; no circuit candidate was built after the lower-bound search failed.
- Delta Q/T/score: 0 / 0 / 0
- Correctness evidence: all 26 encoder outputs are distinct; existing decode round-trips all 26; synthesis search is exhaustive for two fresh-AND straight-line networks with free affine transformations.
- Official validation: not applicable; no operation-stream change.
- Runtime/compute: under one second on one CPU; negligible.
- Decision: `REJECTED` — the current three-AND value interface is locally multiplicative-complexity minimal in the searched model.
- Main caveat: this does not rule out savings obtained by fusing the decoder into a particular consumer or by carrying correlated side information.
- Next: test whether a second exact hold/loan fixed-point pass exposes a small stream-level saving; in parallel retain WL codec resynthesis as a higher-cost direction.

## Experiment E003 — second hold/loan pass

- Timestamp: 2026-11-05 UTC
- Parent: `667ce8d4d8774be3edde6a8c9777bb04a1876bf8`
- Candidate revision: uncommitted isolated patch; reverted after decision
- Hypothesis: the first `hold_pass` rewrites may expose another reusable nonlinear product or a width-saving loan.
- Change: invoke `hold_pass::hold_pass` a second time in `lane_generic_policy`.
- First pass: 11,565 duplicate loans, 333 product candidates, five holds, one relocation move.
- Second pass: 3,946 duplicate loans, 328 product candidates, zero holds, zero relocation moves.
- Result: Q remained 1,243 and no Toffoli-bearing rewrite was placed. The stream grew from 12,338,173 to 12,348,210 emitted operations due only to additional Clifford duplicate-loan moves; hash changed to `94b8447d2d4a44d416aa5d2cde7574a3d1279ca52de8dcb43c136c53273f2e34`.
- Q/T/score: 1,243 / structurally unchanged / structurally unchanged. The `HEO_TOTAL expected=720,108.2` figure initially looked lower than the official mean but is the same pre-pass estimate in the baseline build log; it is not a candidate improvement.
- Delta Q/T/score: 0 / 0 / 0
- Correctness: not advanced to simulation because there is no product-metric mechanism; changing the operation hash would merely redraw the official input corpus.
- Runtime/compute: about 7 minutes generation plus 1 minute 23 seconds incremental release compilation; CPU-heavy, peak memory comparable to baseline.
- Decision: `REJECTED` — more Clifford operations, no logical-width or executed-Toffoli improvement.
- Next: rerun the exact conditional cleanup after `last_veto`, the fixed-point step not covered by E001.

## Experiment E004 — post-veto conditional cleanup

- Timestamp: 2026-11-05 UTC
- Parent: `667ce8d4d8774be3edde6a8c9777bb04a1876bf8`
- Candidate revision: uncommitted isolated patch; reverted after decision
- Hypothesis: `last_veto` may expose conditional AND/phase simplifications not covered by E001's product/affine/truth rescan.
- Change: rerun `cond_pass::final_cap_cleanup` immediately after `last_veto` and before `hold_pass`.
- Result: no added pass diagnostics or rewrite; 12,338,173 emitted operations and byte-identical baseline stream.
- Q/T/score: 1,243 / 721,156.073 / 896,396,908 (inherited from identical verified stream)
- Delta Q/T/score: 0 / 0 / 0
- Ops hash: `1c542d401ededcbceaed6596172aaa35d29a8a9c6c20274ee16e81f9a74fcebb`
- Correctness evidence: exact operation-stream identity to the verified baseline.
- Official validation: not rerun; verified baseline stream is identical.
- Runtime/compute: about 7 minutes generation plus 1 minute 20 seconds incremental release compilation.
- Decision: `REJECTED` — the conditional cleanup is already at a fixed point across `last_veto`.
- Next: GPU feasibility checkpoint requested by the user, then return to peak-lifetime or consumer-fused A2 work.

## GPU feasibility checkpoint

- Timestamp: 2026-11-05 UTC
- Local hardware: one idle NVIDIA GeForce RTX 4090, 24,564 MiB VRAM, compute capability family `sm_89`; driver 595.91.07 reports CUDA 13.2.
- Compiler: `nvcc` 12.8.93 is installed.
- Reviewed source: `jieyilong/ecdsafail_gpu_toolkit` at its 2026-11-05 main branch, cloned read-only for analysis under `/workspace/project/ecdsa-research/gpu_toolkit`.
- Transferable mechanisms: one block per work item with 128 shot lanes; block-wide early rejection; grid-stride work assignment; one process pinned per GPU; Montgomery batch inversion of Jacobian Z values and affine-add denominators; larger fixed-base comb tables; exact CPU/GPU known-case agreement gates; separate exact-obligation manifests; and end-to-end timing rather than kernel-only timing.
- Applicability finding: the supplied CUDA kernels replay later dialog-GCD and TrailMix/jump-GCD filters and derive hash-selected nonce corpora. They do not implement this baseline's 124-tick A2 record/replay circuit or an independent random-input point-add equivalence check. Using them directly would answer the wrong correctness question and overemphasize official nonce islands, contrary to this research protocol.
- Throughput evidence from the toolkit: GPU scan acceleration is large for its specialized GCD predicate, while exact full-circuit CPU validation remains a separate stage; its own documentation warns that unsafe fused filters can miss known-clean cases and requires known-case smoke tests.
- Decision: do not build a new GPU verifier yet. Current bottleneck is discovering a resource-positive candidate, not evaluating a queue of promising candidates; E001-E004 were rejected before independent large-sample simulation. A faithful A2/secp256k1 GPU replay would be material infrastructure with no candidate to amortize it over.
- Integration trigger: once an exact structural candidate changes the stream with a positive Q×T mechanism, first implement a small CPU random-input reference and paired baseline/candidate checker outside the repo, then port only the dominating arithmetic constraints to CUDA. Require exact CPU/GPU agreement on 10,000 identical inputs including separate value/phase/ancilla classifications; benchmark preprocessing, transfers, launches and output reduction. Scale to 100,000 and then 1,000,000 only if the candidate remains promising and wall-clock speedup is meaningful. The unchanged official CPU evaluator remains final authority.
- No harness, evaluator, Cargo file, dependency, submitted source, or scoring path was modified for this checkpoint.

## Experiment E005 — 1,242-qubit frozen walk cap

- Timestamp: 2026-11-05T15:34:46Z
- Parent: `667ce8d4d8774be3edde6a8c9777bb04a1876bf8`
- Candidate revision: `7bf27013a5d93dfa81931d1758d789f281db0aba` on `research/autonomous-20261105`
- Hypothesis: reducing `FROZEN_WALK_CAP` from 1,243 to 1,242 may trade fewer than about 580 mean executed Toffolis for one qubit and improve the product.
- Change: in `src/point_add/mod.rs`, set the `WL_CODEC` frozen walk cap to 1,242; no other implementation change.
- Resource build: Q=1,242; `HEO_PHASE divide expected=340,596.5`, `multiply expected=345,538.2`; pre-pass `HEO_TOTAL expected=720,621.8` (baseline 720,108.2, delta +513.6); 12,351,360 emitted operations.
- Operation stream SHA-256: `35035a0782191276aaa75d15fe5165d97a02337b65b1ca088ffbe8df34eb5f6a`.
- Protected evaluator measurement: mean executed Toffoli 721,685.317, rounded 721,685; Q=1,242. Counterfactual score if correct: `1242 * 721685 = 896,332,770`, 64,138 below baseline.
- Official validity: failed on its hash-derived corpus. The separate-process official CPU evaluator tested 9,024 shots and found 12 classical mismatches, phase garbage in 6 batches, and zero ancilla-garbage batches. First mismatch was shot 403. This prevents promotion to official-valid-best but is not, by itself, evidence of a worse global error rate than the already approximate baseline.
- Local-test diagnosis: the standalone divide/multiply and integrated point-add source tests abort during construction at forward tick 111 because their legacy setup does not reproduce all frozen submission-only room settings. This is a test-setup limitation, not positive correctness evidence; no test source or harness was modified to work around it.
- Official evaluation artifact: evaluator-appended `results.tsv` row timestamp 1793892875, commit label `667ce8d4`, Q=1,242, T=721,685.317, `FAIL`. This generated row was not manually edited.
- Runtime/compute: operation generation about 7 minutes; official evaluation 24 seconds wall clock; CPU-only. The GPU was not used because the specialized external toolkit does not replay this A2 circuit and the protected CPU evaluator was already fast.
- Independent paired evaluation: a scratch CPU checker loaded the preserved baseline and candidate operation streams, generated one fixed independent scalar corpus, used the same measurement-coin seed, and compared both to the secp256k1 reference without changing the protected evaluator.
  - At 10,000 inputs, baseline and candidate each had 16 overall failing inputs (0.160%); the failing sets were identical. Value failures were identical at 15; phase failures were 8 baseline versus 11 candidate; ancilla failures were zero. Overall baseline-only/candidate-only counts were 0/0.
  - At 100,000 inputs, baseline had 204 overall failures (0.204%, Wilson 95% CI 0.1779%–0.2339%) and candidate had 207 (0.207%, CI 0.1807%–0.2371%). Paired baseline-only/candidate-only counts were 10/13, so the estimated overall change is +0.003 percentage points with paired normal 95% CI -0.0064 to +0.0124 points.
  - Value failures were exactly the same 181 inputs (0.181%) for both streams, with 0/0 discordant value failures. Phase failures were 106 baseline versus 128 candidate, with 49/71 baseline-only/candidate-only; phase-rate change +0.022 points, paired 95% CI about +0.0005 to +0.0435 points. Ancilla failures were 0/0.
  - Common-input mean executed T was 721,392.895 baseline versus 721,911.256 candidate. Using rounded common-input T, products were 896,691,499 and 896,613,462: candidate improvement 78,037. Using the official per-stream measurements gives improvement 64,138. Both comparisons favor the candidate.
- Main finding: lowering the cap changes phase-error behavior modestly but did not change value failures on 100,000 paired independent inputs. The overall error-rate increase is small and statistically inconclusive at this sample size, while Q×T is structurally and empirically lower. Do not pursue nonce tuning or hash-selected input repair.
- Decision: `RESEARCH_PROMISING` — approximate Pareto candidate with lower Q×T, independently measured error near the baseline, and unresolved official validity.
- Next: snapshot the one-line candidate and its operation stream; keep the official-valid incumbent separate, then resume exact-preserving and approximate frontier experiments.

## Five-experiment checkpoint

- Attempts: 5 total; verified improvements 0; research-promising 1; rejected 4; inconclusive 0.
- Official-valid-best: unchanged baseline, Q=1,243, mean T=721,156.073, score=896,396,908, stream hash `1c542d401ededcbceaed6596172aaa35d29a8a9c6c20274ee16e81f9a74fcebb`.
- Research-best / approximate Pareto incumbent: E005, Q=1,242, official-draw mean T=721,685.317, product 896,332,770 (64,138 lower), with 100K independent overall failure 0.207% versus paired baseline 0.204%. It is not official-valid.
- Closed mechanisms: post-veto exact rescans (fixed point), second hold pass (Clifford-only growth), two-AND A2 joint decoder (exhaustively impossible on the legal alphabet), and standalone 31-to-30 WL codec (capacity bound). Raw cap reduction to 1,242 remains open only as the E005 approximate Pareto point; it failed official validity but passed the independent resource/error tradeoff gate.
- Highest-priority remaining direction: exact peak-lifetime surgery or consumer-fused A2 simplification that does not change the approximate planner/cap, followed by Q-neutral WL codec Toffoli resynthesis.

## Experiment E006 — ordinary WL codec reachable-support Toffoli screen

- Timestamp: 2026-11-05 UTC
- Parent: verified baseline `667ce8d4d8774be3edde6a8c9777bb04a1876bf8`; E005 cap patch restored before this experiment
- Candidate revision: analysis-only; scratch script `/workspace/project/ecdsa-research/scratch/wl_codec_screen.py`
- Hypothesis: a generated `ORDINARY_COMMIT` or `ORDINARY_POP` CCX may have an impossible control conjunction, or its conjunction may equal an affine expression of other live wires, on the exact 26^4 legal block domain.
- Method: bit-sliced exhaustive propagation of all 456,976 legal ordinary-block inputs through the Rust operation lists. HMR resets the measured basis wire; conditional Z/CZ/CCZ operations do not change basis values. At every CCX, test the control product for zero and membership in the complete affine span of the other current wires plus constant 1.
- Correctness of the model: the scratch model reproduced the complete commit/pop basis-state round trip for all 456,976 inputs, including clean temporary wires.
- Result: zero constant-control or affine-control CCX candidates in either direction.
- Q/T/score and deltas: unchanged baseline; no source candidate was built.
- Runtime/compute: about 10 seconds, one CPU, negligible memory relative to a circuit build.
- Decision: `REJECTED` — no exact local Toffoli-to-Clifford rewrite exists in this class for the ordinary codec.
- Caveat: this does not prove global Toffoli minimality or cover nonlinear resynthesis with a different intermediate network, the smaller prefix codec, or correlations between adjacent ordinary blocks beyond the documented Cartesian legal domain.
- Next: inspect the smaller prefix codec with its exact reachable domain, then return to a bounded peak-lifetime target if that screen is also empty.

## Experiment E007 — prefix WL codec reachable-support Toffoli screen

- Timestamp: 2026-11-05 UTC
- Parent: verified baseline; analysis-only continuation of E006
- Hypothesis: the shortened prefix block may expose a constant or affine CCX control product absent from ordinary blocks.
- Exact domain: tid0 has only A positions 2 and 3 because tick 0 is not an A tick. Each position has the three legal `(k3,k2)` values `(0,0)`, `(0,1)`, `(1,0)`, giving 9 distinct encoded four-bit words. Paired with tid1's 26 ordinary words, the prefix domain has 234 states.
- Method/result: exhaustively propagate all 234 states through `PREFIX_COMMIT` and `PREFIX_POP`; the model round-tripped every state with clean temporaries. At each CCX, test its control conjunction against zero and the complete affine span of all other live wires. Zero candidates were found.
- Q/T/score and deltas: unchanged baseline; no submitted-source change.
- Runtime/compute: included in the approximately 10-second E006/E007 scratch run; negligible.
- Decision: `REJECTED` — neither generated codec direction has an exact support-only Toffoli-to-Clifford rewrite of the screened class.
- Next: bounded peak-lifetime analysis around the shared Q=1,243 reverse-A2 additions, looking specifically for a wire that can be released/recomputed without lowering the global walk cap.

## Experiment E008 — 1,241-qubit frozen walk cap

- Timestamp: 2026-11-05 UTC
- Parent: `22e237b5` (research history plus restored official-valid cap); implementation delta from verified baseline is the single frozen-cap change 1,243 to 1,241.
- Hypothesis: the second qubit may cost less than the approximately 1,161-T two-qubit break-even and extend the approximate score/error frontier.
- Source change: set the `WL_CODEC` `FROZEN_WALK_CAP` to 1,241 in `src/point_add/mod.rs`.
- Build/resource evidence: Q=1,241; divide expected T 340,795.5; multiply 345,867.5; pre-pass total expected T 721,150.0; 12,364,837 emitted operations. Operation-stream SHA-256 `0e048d08820f3db1229ab419c12a88837c5c87d2817bf6d61170d16406c4f238`.
- Protected CPU evaluator: mean executed T 722,204.213, rounded 722,204; product `1241 * 722204 = 896,255,164`, which is 141,744 below the verified baseline. Official hash-derived validity failed: 23 classical mismatches, phase garbage in 16 batches, zero ancilla-garbage batches over 9,024 shots. Evaluator-appended `results.tsv` row timestamp 1793894547; the generated row was not manually edited.
- Independent paired 10K diagnostic: baseline/candidate value failures 15/15 with identical failing inputs; phase 8/7; ancilla 0/0; overall failures 16/17 with baseline-only/candidate-only 1/2. Common-input T 724,627.529/725,663.653.
- Independent paired 100K evaluation:
  - Baseline overall failures 204/100,000 = 0.204% (Wilson 95% CI 0.1779%–0.2339%); candidate 205/100,000 = 0.205% (CI 0.1788%–0.2350%).
  - Overall baseline-only/candidate-only counts 11/12. Paired error-rate change +0.001 percentage points; paired normal 95% CI -0.0084 to +0.0104 points.
  - Value failures were exactly the same 181 inputs for both streams, with zero discordance. Phase failures were 106 baseline versus 121 candidate, with baseline-only/candidate-only phase counts 50/65; change +0.015 points, paired 95% CI about -0.0060 to +0.0360. Ancilla failures were 0/0.
  - Mean common-input T 721,392.895 baseline versus 722,437.626 candidate. Products using rounded common-input T: 896,691,499 versus 896,545,558, improvement 145,941.
- Comparison with E005: on the identical 100K corpus E008 has lower product, fewer overall failures (205 vs 207), and fewer phase failures (121 vs 128). It empirically dominates E005 at this resolution, though both snapshots are retained.
- Runtime/compute: build about 7 minutes plus incremental compilation; official evaluation 23 seconds; paired 10K about 37 seconds and paired 100K about 334 seconds after compilation. CPU-only; no GPU port was justified because the validated checker completed the decision in minutes.
- Decision: `RESEARCH_PROMISING` and current research-best approximate candidate. It is not official-valid and is not promoted to official-valid-best.
- Next: snapshot source and operation stream, restore the official-valid cap, then screen cap 1,240 only if its predicted/actual T remains inside the three-qubit break-even; continue exact-preserving local synthesis in parallel when a concrete mechanism is available.

## Eight-experiment checkpoint

- Attempts: 8 total; verified improvements 0; research-promising snapshots 2; rejected 6; inconclusive 0.
- Official-valid-best: unchanged verified baseline, Q=1,243, mean T=721,156.073, score=896,396,908.
- Research-best approximate point: E008, Q=1,241, mean T=722,204.213 on its official resource draw, product 896,255,164, improvement 141,744; independent overall error 0.205% on 100K versus paired baseline 0.204%.
- Approximate frontier: E008 currently dominates E005 on the common 100K sample. E005 remains a useful intermediate snapshot and replication point.
- Exact frontier: unchanged baseline. E006/E007 closed support-only Toffoli simplifications in both ordinary and prefix WL codecs; more ambitious nonlinear resynthesis or consumer fusion remains open.

## Experiment E009 — 1,240-qubit frozen walk cap

- Timestamp: 2026-11-05 UTC
- Parent: E008 snapshot `8f7e1e4fd6f797da42d56daf88d865ab78e555a6`; implementation delta from verified baseline is only the frozen cap 1,243 to 1,240.
- Hypothesis: a third qubit reduction may remain below the approximately 1,744-T three-qubit product break-even.
- Build/resource evidence: Q=1,240; divide expected T 340,981.0; multiply 346,195.0; pre-pass total expected T 721,663.0; 12,379,988 emitted operations. Base-stream SHA-256 `fd8b16e7b4aee4243961ac9395ae0a2788ac44a43944d7b149d42f779984396e`.
- Protected CPU evaluator: mean executed T 722,726.777, rounded 722,727; product `1240 * 722727 = 896,181,480`, 215,428 below verified baseline and 73,684 below E008. Base nonce official validation failed with 20 classical mismatches, phase garbage in 11 batches, and zero ancilla garbage over 9,024 shots. Evaluator-appended row timestamp 1793895544; not manually edited.
- Independent paired 10K diagnostic: baseline/candidate value failures 15/15, exactly identical inputs; phase 8/9; ancilla 0/0; overall failures 16/16 with no overall discordance. Common-input T 724,627.529/726,193.386.
- Independent paired 100K evaluation:
  - Baseline overall 204/100,000 = 0.204%; candidate 199/100,000 = 0.199% (Wilson 95% CI 0.1732%–0.2286%).
  - Overall baseline-only/candidate-only counts 14/9; paired change -0.005 percentage points, paired normal 95% CI -0.0144 to +0.0044 points.
  - Value failures were exactly the same 181 inputs, zero discordance. Phase 106 baseline versus 107 candidate, baseline-only/candidate-only phase 55/56; change +0.001 points, 95% CI about -0.0196 to +0.0216. Ancilla 0/0.
  - Common-input mean T 721,392.895/722,948.395. Rounded common-input products 896,691,499/896,455,520, improvement 235,979.
- Frontier decision: `RESEARCH_PROMISING`, research-best approximate candidate. It empirically dominates E008 and E005 on the common 100K sample: lower product and no observed increase in value or overall error.
- Official status: not official-valid at the base nonce. Per updated user authorization, local nonce search is allowed and will be recorded separately; a clean local nonce would not establish global correctness or competition-rule permission.
- Runtime/compute: build about 7 minutes plus compilation; official evaluation 24 seconds; paired 10K about 36 seconds and paired 100K about 336 seconds after compilation; CPU-only.
- Next: snapshot the exact base stream and source, investigate official nonce-search rules, then benchmark a bounded nonce screen. Preserve three distinct incumbents: research-best Q×T/error (E009), best locally clean-nonce candidate if found, and previously confirmed leaderboard-valid baseline.

## Local nonce-search policy and competition-rules checkpoint

- Timestamp: 2026-11-05 UTC
- User authorization: local clean-nonce research is permitted, including GPU-assisted screening, but leaderboard submission, publication, and submission APIs remain explicitly prohibited until separately authorized.
- Official rule evidence: the ECDSA.fail Terms say the validity inputs are derived from the submitted circuit hash so they “cannot be predicted or tuned to in advance,” while also allowing ordinary retries on one's own account subject to the rules. The repository README describes the same hash-derived test-set mechanism. Sources: <https://www.ecdsa.fail/terms> and <https://github.com/Layr-Labs/ecdsafail-challenge/blob/main/README.md>.
- Compliance finding: deliberately screening identity-tail nonces to select a clean hash-derived corpus is in unresolved tension with the plain-language anti-tuning statement. Local experimentation is therefore being kept strictly separate from competition-use permission. A locally clean nonce will be recorded as local-validation evidence only and will not be submitted or described as leaderboard-valid.
- Third-party evidence: `jieyilong/ecdsafail_gpu_toolkit` and its island-hunting notes advocate GPU nonce filtering, but they are not official rules and their supplied CUDA predicates target earlier Dialog/TrailMix-style GCD architectures, not this Skywalk/A2 circuit. Sources: <https://github.com/jieyilong/ecdsafail_gpu_toolkit> and <https://github.com/jieyilong/ecdsafail_skills/blob/main/ecdsafail-island-hunting/SKILL.md>.
- Technical decision: use the architecture-matched in-tree research test `leapfrog_official_lambda` for a bounded CPU screen. It changes only the identity-tail X-pair targets, rehashes the exact E009 operation stream, derives the corresponding 9,024 inputs, and executes the circuit simulator. The external GPU toolkit will not be used as an oracle for E009. A GPU port remains deferred unless this exact full-circuit screening is proven to be a material bottleneck and a CPU/GPU known-case agreement gate is first established.
- Confirmation rule: if a clean nonce is found, freeze it in the submitted implementation, rebuild the operation stream, confirm the hash, and run the unmodified official CPU evaluator. No submission API will be invoked.
- Separate incumbents before nonce screening: research-best is E009; locally validated candidate is none; previously confirmed leaderboard-valid baseline remains commit `667ce8d4d8774be3edde6a8c9777bb04a1876bf8` with Q=1,243, mean T=721,156.073, score 896,396,908, and hash `1c542d401ededcbceaed6596172aaa35d29a8a9c6c20274ee16e81f9a74fcebb`.

## Experiment E009-N — bounded local nonce screen

- Timestamp: 2026-11-05 UTC
- Candidate: E009 commit `ac7c1c128c4c85e871ed15a9c7d0ae275c43dc36`, base hash `fd8b16e7b4aee4243961ac9395ae0a2788ac44a43944d7b149d42f779984396e`.
- Method: architecture-matched `leapfrog_official_lambda` replay. It regenerated the exact E009 stream once, rewrote only the 48 identity-tail X;X target pairs for each nonce, derived each stream's exact 9,024-shot Fiat-Shamir corpus, and ran the same bit-sliced circuit simulator.
- Nonces screened: 3,458,839,657 through 3,458,839,664 inclusive.
- Total failure counts: 17, 21, 26, 18, 23, 27, 7, 17; mean 19.50 failures/draw. The best near miss was nonce 3,458,839,663 with six wrong-value shots, four phase-failing shots, and seven union failures. No clean nonce was found.
- Runtime: 514.14 seconds test time including one full circuit generation/optimization pass; per-nonce exact simulation after setup was roughly 15–30 seconds.
- Decision: `INCONCLUSIVE` for clean-nonce existence and `REJECTED` as a scalable search inner loop. Preserve the near miss as a regression case. Do not widen hash-derived screening while the competition-rule anti-tuning language remains unresolved.
- Local-validation incumbent: still none. This screen is local research only and was not submitted.

## GPU integration checkpoint — exact Skywalk/A2 replay

- Timestamp: 2026-11-05 UTC
- Hardware: one NVIDIA GeForce RTX 4090, compute capability 8.9, 24,564 MiB VRAM; driver 595.91.07 advertises CUDA 13.2; `nvcc` 12.8.93.
- Toolkit source: `jieyilong/ecdsafail_gpu_toolkit` commit `2fdcec6e755b34cdf7c11b9e36e9abf03701e6ec`, cloned outside the challenge repository at `/workspace/project/ecdsa-research/gpu_toolkit`.
- Original-toolkit execution: its production `gpu_island2.cu` compiled successfully as native sm_89 plus PTX using the toolkit's `runtime/build_kernel.sh`; binary SHA-256 `4d55f49690357316ddd0ac11aac40b51ef4338931d09a82a7607f80949ec09ae`. Executing it without a state correctly failed closed (`cannot open /tmp/gpu_state.bin`). The wrapper install/dump workflow was not run because it would copy obsolete helpers into protected `src/bin/` and expects the removed Dialog API.
- Compatibility audit: the supplied filters implement Dialog GCD, TrailMix-thin, or a hard-coded ludicrous jump-GCD schedule. None models the current 124-tick Skywalk/A2 operation stream. They are not used as E009 or baseline correctness oracles.
- Reused components: auto-detected native CUDA build flow; CUDA launch/error/timing structure; byte-exact SHAKE256/Keccak-f1600 implementation and its CPU cross-check pattern; block-level independent-work scheduling; compact immutable GPU-state serialization; full cold-start timing discipline. The toolkit's secp256k1 field/Jacobian/comb and batch-inversion code was inspected but is not yet used by the replay v1 because trusted CPU input/reference generation remains outside the kernel.
- Architecture-matched implementation: `/workspace/project/ecdsa-research/gpu_skywalk/` contains an external Rust state packer and CUDA generic replay kernel. It decodes an immutable `ops.bin`, preserves the 64-shot bit-slice, all nested classical conditions (observed maximum depth three), conditional swap, HMR/R semantics, phase, ancilla cleanup and executed CCX/CCZ accounting. Each batch is one CUDA block with qubit/classical state in shared memory. Per-batch SHAKE states are serialized by a CPU implementation copied from and checked against the toolkit and squeezed on-device, avoiding multi-gigabyte RNG transfers.
- Exact operation census: E009 has Q=1,240, 2,571 classical bits, 12,379,988 serialized ops, 12,378,960 executable ops after annotations, 788,665 HMR/R words per batch and condition depth three. Baseline has Q=1,243, 2,566 bits, 12,338,173 serialized ops, 12,337,145 executable ops and 783,303 HMR/R words per batch.
- Boundary agreement: E009 batch sizes 1, 63, 64 and 65 all had zero CPU/GPU disagreement in value masks, phase masks, ancilla masks and per-batch Toffoli totals. These cover partial masks and a batch boundary.
- E009 10K: CPU and GPU exactly agreed on 15 value failures, 9 phase failures, zero ancilla failures and total T=7,261,933,862 (mean 726,193.386). GPU kernel 3.345 s; cold GPU process including 0.388 s file load, 0.212 s context and transfers took 4.043 s. CPU replay took 13.485 s: 4.03x kernel-only or 3.34x cold-process replay speedup.
- Baseline 10K: exact agreement on 15 value, 8 phase, zero ancilla and total T=7,246,275,295 (mean 724,627.529). GPU kernel 3.355 s and cold total 4.049 s versus CPU replay 14.596 s: 4.35x kernel-only or 3.61x cold-process replay speedup.
- E009 100K: exact agreement on all masks and batch T totals; 181 value failures, 107 phase failures, zero ancilla failures and total T=72,294,839,506 (mean 722,948.395). CPU replay 129.332 s; GPU kernel 17.866 s and cold total 18.742 s. Sustained replay speedup is 7.24x kernel-only and 6.90x including state-file load, GPU context, allocation, H2D/D2H and teardown.
- End-to-end limitation: deterministic CPU secp256k1 input/reference generation took 112.084 s at 100K, and the current packer also serializes per-batch initial/expected state. Thus replay is no longer the dominant wall-clock component at large N. Porting the already-inspected comb/batch-inversion input generator is the next GPU-throughput enhancement only if repeated 100K/1M evaluations become the research bottleneck.
- Correctness scope: this is full forward op-stream replay for independent corpora, including value/phase/ancilla/T, not forward/reverse identity and not official hash-derived input generation. The unmodified official CPU evaluator remains the final scoring and official-validity authority.
- Workflow decision: `VALIDATED_ACCELERATOR`. Use the GPU replay after CPU manifest generation for 10K diagnostics and 100K promising-candidate checks; retain periodic CPU/GPU exact agreement gates. Do not use obsolete Dialog/TrailMix predicates for Skywalk/A2 and do not treat any partial future filter as full validation.

## Experiment E010 — 1,239-qubit frozen walk cap

- Timestamp: 2026-11-05 UTC
- Parent: E009 commit `ac7c1c128c4c85e871ed15a9c7d0ae275c43dc36`.
- Candidate revision: `dd903d13d2fc93394a527729d0f796e6e7f1d65f`; implementation delta from the verified baseline is only the frozen cap 1,243 to 1,239.
- Hypothesis: a fourth qubit reduction may cost less than the rounded-T break-even of 723,484 at Q=1,239. The prior three steps predicted about +519 mean T for this step.
- Build/resource evidence: Q=1,239; divide expected T 341,158.0; multiply 346,537.0; pre-pass total expected T 722,182.0, exactly +519 from E009; 12,392,884 emitted operations. Stream SHA-256 `273c647f6956410dd6fe64d00e8725b36aceba359a43f00fa0693f4cc93348ab` and snapshot `/workspace/project/ecdsa-research/scratch/e010-ops-273c647f6956410d.bin`.
- Protected CPU evaluator: mean executed T 723,241.094, rounded 723,241; product `1239 * 723241 = 896,095,599`, 301,309 below the verified baseline and 85,881 below E009. Its base-nonce hash-derived validation failed with 17 classical mismatches, phase garbage in 13 batches and zero ancilla garbage over 9,024 shots. First mismatch was shot 111. The evaluator appended the `results.tsv` row timestamp 1793898444; it was not manually edited.
- Independent 10K diagnostic: value failures 15, phase 8, ancilla 0, union 17. Against baseline: value discord 0; phase baseline-only/candidate-only 4/4; union 1/2. Mean T 726,721.512. The CUDA replay exactly matched every CPU value/phase/ancilla mask and per-batch T total; cold GPU replay 4.102 s.
- Independent paired 100K:
  - Baseline union failures 204/100,000 = 0.204% (Wilson 95% CI 0.1779%–0.2339%); E010 198/100,000 = 0.198% (CI 0.1723%–0.2275%).
  - Baseline-only/candidate-only union counts 18/12. Paired change -0.006 percentage points, normal 95% CI -0.0167 to +0.0047 points.
  - Value failures were exactly the same 181 inputs, with zero discordance.
  - Phase failures were 106 baseline versus 110 candidate; baseline-only/candidate-only 59/63. Phase-rate change +0.004 points, paired 95% CI about -0.0176 to +0.0256. Ancilla failures were 0/0.
  - E010 versus E009 union counts were 198 versus 199, with E009-only/E010-only 14/13; E010 empirically dominates E009 on product and observed union error at this resolution.
  - Common-input mean T baseline/E010 721,392.895/723,475.977. Rounded products 896,691,499/896,386,764; E010 improvement 304,735.
- GPU correctness/performance at 100K: CPU and GPU exactly agreed on all masks and batch T totals. CPU replay 136.726 s; CUDA kernel 17.892 s and cold replay process 18.713 s, a 7.64x kernel-only or 7.31x cold-process replay speedup for this stream.
- Runtime/compute: official build/eval roughly nine minutes including cold compilation; independent 10K pack/reference 33 s plus 4.1 s GPU; independent 100K pack/reference 284 s plus 18.7 s GPU. One RTX 4090 plus CPU reference generation.
- Decision: `RESEARCH_PROMISING` and new research-best approximate candidate. It is not official-valid at the base nonce. No local nonce screen is widened because the official anti-tuning language remains unresolved.
- Frontier: E010 dominates E009/E008/E005 on the common 100K sample. Official-valid-best remains the verified 1,243-qubit baseline; locally clean-nonce candidate remains none.
- Next: screen cap 1,238 only if the measured/predicted T remains below its rounded break-even 724,068. The observed E010 step cost +514.317 official-draw T and +527.582 common-input T, so one more step is predicted to remain product-positive but with a much thinner margin.

## Additional sprint start

- Timestamp: 2026-11-05 UTC
- Authorization: up to 60 additional hypothesis/implementation/evaluation experiments or six hours, whichever occurs first; checkpoints every ten experiments or sixty minutes. E011, already in flight when authorization arrived, is additional-sprint experiment 1.
- Preserved starting incumbents: official-valid baseline `667ce8d4d8774be3edde6a8c9777bb04a1876bf8`; research-best E010 `dd903d13d2fc93394a527729d0f796e6e7f1d65f`; locally clean-nonce candidate none.
- Restrictions remain: no leaderboard submission, publication, submission API, protected-component modification or large-scale hash-derived screening.

## Experiment E011 — 1,238-qubit frozen walk cap

- Timestamp: 2026-11-05 UTC
- Parent: E010 `dd903d13d2fc93394a527729d0f796e6e7f1d65f`; candidate source commit `4617b01498cda44c256877aa1186c90335c3767e`.
- Hypothesis: the fifth cap-reduction step remains below the 1,238-qubit rounded-T break-even of 724,068 and therefore lowers the product despite additional carry shedding/recomputation.
- Exact source change: `FROZEN_WALK_CAP` 1,239 -> 1,238; no other submitted implementation change.
- Resource evidence: Q=1,238; mean executed T=723,774.499, rounded 723,774; emitted operations 12,408,576. Product `1,238 * 723,774 = 896,032,212`, improving E010 by 63,387 and the verified baseline by 364,696 (0.040685%). Stream SHA-256 `715c84882d32b36f3830cc57b1b0e71d17b1d8adac1111c2be9fe8df150ace97`; snapshot `/workspace/project/ecdsa-research/scratch/e011-ops-715c84882d32b36f.bin`.
- Protected evaluator: failed its distinct hash-derived 9,024-shot corpus with 21 classical mismatches, phase garbage in 14 batches and zero ancilla garbage. This is official-invalid at the base nonce, not a paired global-error estimate. Evaluator-appended row timestamp 1793899640 was not manually edited.
- Independent 10K: value 15, phase 8, ancilla 0, union 16. Against baseline, value discordance 0/0, phase baseline-only/candidate-only 4/4 and union 1/1. CPU/GPU masks and batch Toffoli totals agreed exactly. Common-input mean T=727,252.259.
- Independent 100K:
  - E011 union failures 200/100,000 = 0.200% (Wilson 95% CI 0.1742%-0.2297%) versus baseline 204 = 0.204% (0.1779%-0.2339%). Baseline-only/candidate-only union counts 17/13; change -0.004 percentage points with paired normal 95% CI approximately -0.0147 to +0.0067 points.
  - Value failures were exactly the same 181 inputs, zero discordance. Phase failures were baseline/E011 106/100, baseline-only/candidate-only 64/58. Ancilla failures 0/0.
  - Versus E010, union was 198/200 with E010-only/E011-only 7/9; phase 110/100 with 51/41 discordance. E011 has lower score but E010 has the slightly lower observed union estimate, so both remain on the empirical score/error Pareto frontier at this resolution.
  - Common-input mean T=724,019.495; rounded common-input product `1,238 * 724,019 = 896,335,522`, 355,977 below the paired baseline product and 51,242 below E010.
- GPU validation/performance: exact CPU/GPU agreement on all 100K masks and per-batch T totals; GPU kernel 17.674 s and cold replay 18.459 s versus CPU simulation 146.998 s, an 8.32x kernel-only or 7.96x cold-process replay speedup. The first pack used the old repeated input-generation path (112.421 s) to preserve the already-running experiment.
- Runtime/compute: official build/evaluation completed before the independent checks; 10K pack/replay 36.9 s; first 100K pack 293.5 s plus 18.5 s GPU. One RTX 4090 plus CPU reference generation.
- Decision: `RESEARCH_PROMISING` and new score-best candidate. It is not official-valid. Preserve E010 separately on the score/error frontier.
- Next: do not immediately assume a 1,237 cap wins: the remaining product margin is thin. First test a Q-neutral, low-T error buyback (`C35_WIDEN`) on E011, then return to the raw cap frontier with measured costs.

## GPU workflow improvement — immutable paired-corpus cache

- Timestamp: 2026-11-05 UTC
- Scope: external scratch tooling only at `/workspace/project/ecdsa-research/gpu_skywalk`; no protected or submitted challenge file changed.
- Mechanism: serialize the fixed independent `(P,Q,P+Q)` suite once in a domain-bound `SKYINP1` cache and bit-slice it for each candidate. This removes repeated SHAKE input derivation, scalar multiplication and reference addition without changing the shared inputs, expected outputs, measurement coins or replay semantics.
- Validation: cached and original baseline-10K packs were byte-identical; cached and original E010-10K packs were also byte-identical despite their different operation/Q/bit/wire layouts. Both replays had zero GPU/CPU disagreement in value, phase, ancilla and T. A 100K cache extracted from the validated baseline pack has SHA-256 `4d5a3f9654313ace6c11de52eb0d4f24bf066c1226ef1a67d3e91ed443737a4f` and size about 19 MiB.
- Performance: original input/reference preparation 11.253 s at 10K and about 112 s at 100K; cached preparation 0.005 s at 10K, 1.147 s on a coldish 100K read and 0.025 s warm. Conservative cold improvement is about 98x, warm reuse about 4,480x. The remaining packer bottleneck is CPU circuit simulation and writing the large manifest.
- Decision: `VERIFIED_WORKFLOW_IMPROVEMENT`; use the cache for all later paired candidate packs and retain periodic byte-identity and CPU/GPU agreement gates.

## Experiment E012 — cap-specific boundary-compare widening

- Timestamp: 2026-11-05 UTC; additional-sprint experiment 2.
- Parent: E011 source plus recorded checkpoint `b357fbeb`; candidate commit `d8f90260`.
- Hypothesis: widening the cap-added seeded approximate boundary comparisons by two bits buys back phase errors for a small Q-neutral Toffoli cost and may add a lower-error Pareto point.
- Implementation/census: the pre-existing 21-key `C35_WIDEN` table was calibrated for another cap. Two fail-closed census builds first showed one changed comparison width and then the exact live hit vector. The final candidate targets only the 12 live E011 keys: the six boundary compares in the first adds of divide rounds 655 and 698. Each is asserted seeded and still approximate, and reads two additional bits. The unreachable divide/multiply round-661 keys were removed. This census adaptation is part of E012, not separately counted.
- Resource evidence: Q=1,238; mean executed T=723,803.414, rounded 723,803; emitted operations 12,408,618; score `1,238 * 723,803 = 896,068,114`. This is 328,794 below the verified baseline but 35,902 worse than E011. Static expected T rose about 12, while the different official corpus raised measured T by 28.915. Stream SHA-256 `8ec4f3db8bd4ce988e9a7e547907a08834479a76adff979191b16db48da56b48`; snapshot `/workspace/project/ecdsa-research/scratch/e012-ops-8ec4f3db8bd4ce98.bin`.
- Protected evaluator: failed its distinct 9,024-shot corpus with 20 classical mismatches, 11 phase-garbage batches and zero ancilla garbage. Evaluator row timestamp 1793900775; not manually edited.
- Independent 10K: value/phase/ancilla/union 15/9/0/16 versus E011 15/8/0/16; union discordance 1/1. CPU/GPU agreement exact.
- Independent 100K: value/phase/ancilla/union 181/111/0/205 (0.205%, Wilson 95% CI 0.1788%-0.2350%). Versus E011's 181/100/0/200, E011-only/E012-only union counts were 11/16, a +0.005-point change with paired 95% CI approximately -0.0154 to +0.0254 points. Versus baseline, baseline-only/E012-only union was 11/12. Common-input mean T=724,038.227.
- GPU/workflow: cached input prep 0.113 s at 100K, CPU simulation 151.361 s; GPU kernel 17.728 s and cold replay 18.552 s, with zero disagreement on all masks and batch T totals. Cache use saved about 112 s relative to E011's first pack.
- Correctness interpretation: the intended widening changes HMR consumption, so ordinary same-seed phase sets are not semantic common random numbers after the first shifted measurement. The observed error increase does not prove widening is harmful, but E012 is empirically dominated by E011 on both score and this fixed 100K union estimate. A later `C35_WIDEN_PAD` run can isolate nested targeted repairs if error attribution becomes the bottleneck.
- Decision: `REJECTED` as an incumbent; preserve commit/stream as a diagnostic. Revert the widening and continue the raw cap product frontier.
- Next: E013 tests cap 1,237 from E011. Its baseline-score break-even is rounded T=724,653, and the prior step trend predicts a small but positive product gain.

## Experiment E013 — 1,237-qubit cap and cap-frontier stop

- Timestamp: 2026-11-05 UTC; additional-sprint experiment 3.
- Parent: restored E011 implementation after E012; candidate commit `bbd3e7a3`.
- Hypothesis: a sixth raw cap reduction may remain product-positive if its T step is no larger than the preceding cap steps.
- Exact source change: `FROZEN_WALK_CAP` 1,238 -> 1,237; no other active implementation change.
- Resource evidence: Q=1,237; mean executed T=724,378.495, rounded 724,378; emitted operations 12,423,715; score `1,237 * 724,378 = 896,055,586`. This remains 341,322 below the verified baseline but is 23,374 worse than E011, establishing the observed raw-cap product minimum at Q=1,238. Stream SHA-256 `09375f198f09d13dab4ccb0d236da5b808dc012b9b5e77e8de21980828a3f7b8`; snapshot `/workspace/project/ecdsa-research/scratch/e013-ops-09375f198f09d13d.bin`.
- Protected evaluator: failed its distinct 9,024-shot corpus with 16 classical mismatches, 11 phase-garbage batches and zero ancilla garbage. Evaluator row timestamp 1793901504; not manually edited.
- Independent 10K full audit: value/phase/ancilla/union 15/8/0/18 versus E011 15/8/0/16; value discordance zero, union E011-only/E013-only 0/2. CPU/GPU agreement exact. Mean T=727,863.341.
- Independent 100K validated GPU screen: value/phase/ancilla/union 181/103/0/201 = 0.201% (Wilson 95% CI 0.1751%-0.2307%) versus E011's 0.200%. Mean T=724,615.467; common-input product 896,348,755. The v2 screen reports aggregate masks without a CPU-reference record, so no per-input discordance is claimed for this run.
- Decision: `REJECTED` as an incumbent: it is worse than E011 in score and slightly worse in observed union error. Do not test cap 1,236 as another raw-cap step; revisit only after a structural optimization changes the T/Q curve.
- Next: test an exact local one-wire/one-Toffoli recomputation trade in `a2_lt_into` at cap 1,237 to see whether saving the final lower ladder carry reduces cap-induced recomputation enough to reverse the E013 loss.

## GPU workflow improvement — GPU-only screening pack

- Timestamp: 2026-11-05 UTC; external scratch tooling only.
- Version-2 pack mode (`SKYWALK_GPU_SCREEN=1`) preserves cached inputs, expected point outputs and exact per-batch measurement SHAKE states, while skipping the approximately 150-second CPU op-stream simulation/reference-mask record. The CUDA replay directly reports value, phase, ancilla, union and executed T. Full version-1 audit remains the default and the promotion gate.
- Validation: v2 reproduced the existing full-audit E011 and E012 aggregate value/phase/ancilla/union counts and exact total/mean T at both 10K and 100K. No challenge file, protected component or scoring path changed.
- Performance: 100K v2 pack creation about 35.0 s and cold GPU replay about 18.5 s, roughly 53.6 s end to end versus about 203 s with cached inputs plus CPU simulation and GPU audit. Use v2 for ordinary screens; use v1 after format/kernel changes and before promotion.

## Closed structural hypothesis — final comparator output alias

- Read-only source audit found `a2_lt_into` already writes the mathematical top carry directly into the caller's `target`; its allocated `n-1` carries are only lower carry-ins. The proposed terminal-output alias is therefore already implemented and is closed without a redundant build.
- A distinct exact trade remains: eliminate only the final lower carry wire, inject/uninject its conjunction directly into `u[last-1]`, saving one local qubit for one additional Toffoli per invocation. An independent inline model exhaustively matched 3,378,624 basis-state cases for widths 2..8, all supported borrow/operand aliases, restored operands and clean modeled ancillas. A phase-channel argument/test is still required before promotion. This becomes E014 because it has a concrete implementation and may alter cap-1,237 planner decisions.

## Experiment E014 — recompute the final lower comparator carry

- Timestamp: 2026-11-05 UTC; additional-sprint experiment 4.
- Parent: E013 cap-1,237 candidate; candidate commit `cc5dd639`.
- Hypothesis: replacing the final lower `a2_lt_into` carry ancilla and measurement-uncompute with direct injection/recomputation saves one local live wire; at the constrained 1,237 cap, planner savings might outweigh one extra Toffoli per invocation.
- Implementation: for comparator widths at least three, allocate `n-2` rather than `n-1` lower carries. At the final lower MAJ step, target `u[last-1]` directly and apply the same CCX during reversal; widths two and all earlier carries retain the existing path. This is the exact reversible transformation covered by the 3,378,624-case basis model. The HMR/CZ measurement-uncompute being replaced is the standard channel equivalent of inverse CCX on an AND ancilla, so the replacement is also phase exact.
- Resource evidence: global Q remained 1,237; dynamic expected T rose exactly 176, showing 176 affected comparator calls and no planner recovery. Protected mean executed T=724,570.827, rounded 724,571; emitted operations 12,422,995; score `1,237 * 724,571 = 896,294,327`. This is 238,741 worse than E013 and 262,115 worse than E011, though still 102,581 below the verified baseline. Stream SHA-256 `db2aff95a3b2267d0437b64f97d79b7569088b3690be6e8f554fffbd5ba9be41`; snapshot `/workspace/project/ecdsa-research/scratch/e014-ops-db2aff95a3b2267d.bin`.
- Protected evaluator: failed its distinct corpus with 17 classical mismatches, nine phase-garbage batches and zero ancilla garbage; row timestamp 1793902113.
- Independent 10K GPU screen: value/phase/ancilla/union 15/8/0/17, mean T=728,037.979. The identical 15 value-failure count and zero ancilla failures are consistent with the exact classical transformation; a full 100K screen is unnecessary because the resource result decisively rejects the candidate.
- Decision: `REJECTED`. Local wire reduction did not reduce the measured peak or cap-induced recomputation and imposed its exact +176-T call cost. Revert and restore the E011 1,238-cap incumbent before the next structural test.
- Disproven mechanism: `a2_lt_into` terminal-output alias was already present; final-lower-carry recomputation is exact but product-negative at both the 1,237 cap and, absent a different peak binding, the 1,238 cap.

## Experiment E015 — remove multiply tick-136 bridge budget

- Timestamp: 2026-11-05 UTC; additional-sprint experiment 5.
- Parent: restored E011 cap-1,238 incumbent commit `8aded426`; candidate commit `88326a1f`.
- Hypothesis: the documented `C35B_BR_R136` trade removes a bridge budget for about -11 mean T at Q unchanged, at the cost of one extra approximate boundary compare per affected cell.
- Exact source change: `C35B_BR_R136 false -> true`.
- Result: complete no-op on the current Skywalk/A2 recipe. The HEO expected totals, post-pass statistics, emitted operation count (12,408,576), Q=1,238, mean T=723,774.499, score=896,032,212 and operation-stream SHA-256 `715c84882d32b36f3830cc57b1b0e71d17b1d8adac1111c2be9fe8df150ace97` were byte-for-byte identical to E011. The official corpus therefore also repeated E011's 21 classical, 14 phase-batch and zero ancilla failures.
- Diagnosis: the switch changes the broad C35B profile string, but the current A2 pinned planner does not consume that affected tick/profile choice. Host-environment activation is also impossible because the build clears the process environment; active experiments must be compiled/pinned into the frozen recipe.
- Decision: `REJECTED` (structural no-op). Revert without another independent replay because the operation stream is exactly E011's already audited stream.
- Next: E016 enables the existing exact `RAIL_OPT_M0=1` hosted-record path through `FROZEN_RECIPE`; it may reduce reverse-A2 shedding without changing the global cap.

## Additional-sprint one-hour checkpoint

- Timestamp: 2026-11-05 UTC; five experiments complete (E011-E015), E016 in progress; approximately five hours and 55 experiment slots remain under the authorized maximum.
- Official-valid-best: unchanged verified baseline `667ce8d4d8774be3edde6a8c9777bb04a1876bf8`, Q=1,243, T=721,156.073, score=896,396,908, 9,024/9,024 on its preserved stream.
- Score-best research candidate: E011 `4617b01498cda44c256877aa1186c90335c3767e`, Q=1,238, T=723,774.499, score=896,032,212, improvement 364,696 (0.040685%); independent 100K union error 0.200% versus baseline 0.204%.
- Empirical score/error frontier also retains E010 `dd903d13d2fc93394a527729d0f796e6e7f1d65f`, score 896,095,599 and observed 100K union error 0.198%. Neither is official-valid at its base hash. Locally clean-nonce candidate remains none.
- Completed outcomes: E011 improved score; E012 boundary widening was dominated; E013 established the raw-cap minimum at Q=1,238; E014 exact local carry recomputation was product-negative; E015 was byte-identical/no-op.
- GPU: immutable input cache validated; GPU-only v2 screen validated against full CPU/GPU packs. Current 100K screen is about 35 s pack + 18.5 s replay, with periodic v1 audits retained.
- E016 early census: hosted `m0` keeps Q=1,238 but raises pre-pass expected T by 76; finish the measurement, then reject unless post-processing unexpectedly reverses the cost.
- Ranked next queue: extend exact `A2R_SERIAL` by one tick (99); test one omitted exact `A2R_HB` tick (108); then assess `BACK_SEAM_FUSE` one leg at a time and conservative `LFQ_RAILWIN` only with independent error screening. PAD-aligned E012 attribution remains available as an error-analysis experiment.

## Experiment E016 — hosted reverse-A2 m0

- Timestamp: 2026-11-05 UTC; additional-sprint experiment 6; candidate commit `92b5e293`.
- Parent: restored E011 cap-1,238 implementation.
- Hypothesis: pinning `RAIL_OPT_M0=1` makes the reverse A2 chain's top AND host `m0`, removing a separate owner wire and possibly reducing `s1/cr/m2` shedding.
- Exact source change: add `("RAIL_OPT_M0", "1")` to `FROZEN_RECIPE`, ensuring the build uses the knob after process-environment clearing.
- Resource evidence: Q remained 1,238; pre-pass expected T rose by 76 rather than falling. Protected mean T=723,852.361, rounded 723,852; emitted operations 12,409,274; score `1,238 * 723,852 = 896,128,776`, 96,564 worse than E011. Stream SHA-256 `8269e73e2575116d37c6e36a294fdc27a2d7366039bfa5c11e0af9c9a731c0a2`; snapshot `/workspace/project/ecdsa-research/scratch/e016-ops-8269e73e2575116d.bin`.
- Protected evaluator: 22 classical mismatches, 13 phase-garbage batches, zero ancilla garbage on its distinct corpus; row timestamp 1793903148.
- Independent 10K GPU screen: value/phase/ancilla/union 15/9/0/17, mean T=727,333.300. This exact structural path preserves the observed value-failure count and ancilla cleanup; no 100K run is warranted because the score is decisively dominated.
- Decision: `REJECTED`. The hosted record removes an owner but makes the chain path more expensive under the present planner and does not reduce the measured cap. Revert.
- Next: E017 extends the exact `A2R_SERIAL` held-seed schedule by the single tick 99.

## Experiment E017 — extend reverse-A2 serial hold through tick 99

- Timestamp: 2026-11-05 UTC; additional-sprint experiment 7; candidate commit `6ea0bfad`.
- Parent: restored E011 cap-1,238 implementation.
- Hypothesis: changing `A2R_SERIAL` from `1-98` to `1-99` retains tick 99's existing serial carry chain across the fold and avoids rebuilding roughly three carries without changing the cap.
- Resource evidence: Q=1,238. Pre-pass expected T was unchanged; the stream removed 53 emitted operations but added a 0.5-weight measurement region that later simplification neutralized structurally. Protected mean T=723,782.771, rounded 723,783; score 896,043,354, 11,142 worse than E011 on its distinct resource draw. Stream SHA-256 `4064fbd2a4ef85a99dd584ed7724ae3330d8efefa09fb25dd272784b5d68cc65`; emitted operations 12,408,523; snapshot `/workspace/project/ecdsa-research/scratch/e017-ops-4064fbd2a4ef85a9.bin`.
- Protected evaluator: 20 classical mismatches, ten phase-garbage batches and zero ancilla garbage on the distinct hash corpus; row timestamp 1793903644.
- Independent 10K GPU screen: value/phase/ancilla/union 15/8/0/16, mean T=727,253.264, only +1.005 mean T versus E011 on the shared corpus. RNG-word count per batch is unchanged at 792,323. No 100K run is needed because the candidate has no T saving and no 10K error benefit.
- Decision: `REJECTED`. The held form saves operations but not executed Toffoli and is product-negative at Q unchanged. Revert.
- Next: E018 adds only tick 108 to the exact `A2R_HB` schedule; it self-gates on spare room and can save one MAJ Toffoli if active.

## Experiment E018 — add reverse-A2 HB tick 108

- Timestamp: 2026-11-05 UTC; additional-sprint experiment 8; candidate commit `5a9cc4e6`.
- Hypothesis: extend `A2R_HB` from `1-107,...` through tick 108; when two spare wires exist, retaining the chain's top AND replaces a two-AND identity with one MAJ AND for exact -1 T.
- Exact source change: `A2R_HB` range `1-107` -> `1-108` in `FROZEN_RECIPE`.
- Result: byte-identical no-op. Q=1,238, T=723,774.499, score=896,032,212, emitted operations 12,408,576 and hash `715c84882d32b36f3830cc57b1b0e71d17b1d8adac1111c2be9fe8df150ace97`, all exactly E011. The tick's runtime room gate (`room_m >= 2*k0x+2`, no chain release, early-HB enabled) is false, so adding it to the schedule cannot activate the path.
- Decision: `REJECTED` (structural no-op); no independent replay needed for an identical stream.
- Next: E019 enables all remaining omitted live HB ticks (108-109, 112-114, 118-121) in one census to close the whole self-gated schedule class efficiently.

## Experiment E019 — enable all self-gated reverse-A2 HB opportunities

- Timestamp: 2026-11-05 UTC; additional-sprint experiment 9; candidate commit `fb10fa8a`.
- Parent: restored E011 cap-1,238 implementation.
- Hypothesis: set `A2R_HB=1-121`; every newly listed tick remains guarded by exact spare-room and chain-release conditions, so any stream change identifies a safe missed one-AND optimization.
- Resource evidence: Q=1,238. Pre-pass expected T fell by 1 and the post-pass static expectation fell by 2; native Toffoli count fell by two. Protected mean T=723,774.397, rounded 723,774, so the official product remains tied with E011 at 896,032,212 because of rounding. Emitted operations 12,408,547; stream SHA-256 `4bcfed730bd6f1da82945e73a124fd691726b5a353b5b71d5d14315e8a63f4c0`; snapshot `/workspace/project/ecdsa-research/scratch/e019-ops-4bcfed730bd6f1da.bin`.
- Protected evaluator: 22 classical mismatches, 11 phase-garbage batches, zero ancilla garbage on its distinct corpus; row timestamp 1793904606. This exact scheduling rewrite's new hash/corpus is not comparable to E011's official counts.
- Independent 10K full audit: CPU/GPU exact agreement; value/phase/ancilla/union 15/11/0/17 and mean T=727,251.075, a common-input reduction of 1.184 T versus E011. Value failures remain the same count; the phase-set change follows one fewer HMR word per batch (792,322 versus 792,323), not a mathematical value change.
- Trace diagnosis: among newly enabled ticks, tick 112 is the only active opportunity. Its `m01` cost is 37 rather than the neighboring same-width cost 38. Ticks 108-109, 113-114 and 118-121 all remain self-gated/no-op.
- Decision: `RESEARCH_PROMISING` exact T reduction, tied on rounded official score. Retain as a building block because one more small T saving can cross the rounding boundary.
- Next: E020 restricts the schedule to the original set plus tick 112 only and requires the exact E019 operation hash/resources, keeping the source change minimal.

## Experiment E020 — isolate the active reverse-A2 HB tick

- Timestamp: 2026-11-05 UTC; additional-sprint experiment 10; candidate/current commit `234a6b2b`.
- Parent: E019 all-self-gated HB candidate.
- Hypothesis: only tick 112 activates the held-top-AND path; restricting the schedule to the original set plus tick 112 should reproduce E019 byte for byte.
- Exact source change: `A2R_HB` from `1-121` to `1-107,110-112,115-117`.
- Result: confirmed exact identity to E019: Q=1,238, mean T=723,774.397, rounded score 896,032,212, 12,408,547 operations and SHA-256 `4bcfed730bd6f1da82945e73a124fd691726b5a353b5b71d5d14315e8a63f4c0`. The protected result repeats E019's corpus/result because the stream is identical.
- Decision: `VERIFIED_IMPROVEMENT` over E011 in unrounded mean T and static/native Toffoli, tied after official rounding. Retain tick 112 as the current exact-enhanced implementation and integrate later savings on top of it.

## Ten-experiment checkpoint (additional sprint)

- Completed: 10 (E011-E020). Outcomes: one rounded-score improvement (E011), one exact unrounded-T improvement retained (E019/E020), five resource/correctness rejections, and three byte-identical/no-op closures. No official-valid candidate and no local clean-nonce candidate were added.
- Current exact-enhanced research-best: commit `234a6b2b`, Q=1,238, T=723,774.397, rounded score 896,032,212; stream `4bcfed73...f4c0`. Its independent 10K full audit exactly matched CPU/GPU and retained the 15 value-failure count; larger error inference remains based on E011 because the rewrite is exact and the changed HMR count shifts phase RNG positions.
- Empirical score/error frontier still retains E010 for its 0.198% observed 100K union rate and E011 for 0.200%; E020 has the same rounded score as E011 with lower unrounded T.
- Confirmed failures/closures this block: raw caps stop at 1,238; final-lower comparator carry recomputation costs +176 T; C35 boundary widening is dominated at 100K; hosted m0 costs +76 expected T; serial tick99 does not save T; HB tick108 and the C35B tick136 bridge switch are no-ops.
- Workflow: GPU cached full audit and GPU-only screen are both validated; 100K screening is about 53.6 seconds, with the official CPU evaluator retained as authority.
- Next ranked experiment: divide-only `BACK_SEAM_FUSE=1`, then multiply-only `=2`, each independently screened before any combination. Follow with conservative `LFQ_RAILWIN` only if the seam trade is not useful.

## Experiment E021 — divide-side legacy seam fusion on the active Leapfrog path

- Timestamp: 2026-11-05T19:09:10Z; additional-sprint experiment 11; transient source based on checkpoint commit `4306bb8f`.
- Hypothesis: re-enable `BACK_SEAM_FUSE=1` after `leapfrog::install_recipe()` clears the broad recipe setting, potentially removing the documented divide-side correction ladder for about 51 T at Q unchanged.
- Exact source change: set `BACK_SEAM_FUSE=1` immediately after Leapfrog recipe installation; reverted after measurement.
- Result: byte-identical no-op. Q=1,238, mean T=723,774.397, rounded score=896,032,212, 12,408,547 emitted operations and SHA-256 `4bcfed730bd6f1da82945e73a124fd691726b5a353b5b71d5d14315e8a63f4c0`, exactly E020. The protected replay consequently repeated 22 classical, 11 phase-batch and zero ancilla failures.
- Independent source diagnosis: the active `LF_SEAMS=1` path calls `leapfrog::divide_seamed`/`multiply_seamed`, with its own seed/unseed and `seam_add_reduce`. All live `BACK_SEAM_FUSE` consumers are confined to legacy HEO/classical paths, so the flag cannot affect Skywalk/A2. If ported later, the documented no-wrap support assumptions would require dedicated boundary proof/tests; ordinary 100K samples cannot resolve the rare wrap cases.
- Decision: `REJECTED` (structural no-op). No separate paired replay is needed because the operation stream is exactly the already audited E020 stream.
- Next: E022 extends the exact forward fold kept-carry schedule from tick 55 through tick 56; its local MAJ identity predicts about -1.5 mean T if the held wires fit without a planner cliff.

## Experiment E022 — extend exact forward fold kept-carry schedule through tick 56

- Timestamp: 2026-11-05 UTC; additional-sprint experiment 12; candidate source commit `963dd6da7a2a9f5355475dedd2a67a16cffd49d2`.
- Parent: E020 exact-enhanced implementation (`234a6b2b` source, checkpoint `4306bb8f`).
- Hypothesis: extending `LFG_KFWD` from `1-55` to `1-56` retains tick 56's three inner MAJ carries and avoids erase/rederive work without changing the cap.
- Resource evidence: Q=1,238; protected hash-derived mean T=723,794.908, rounded score `1,238 * 723,795 = 896,058,210`, 25,998 worse than E020 on a different corpus. Emitted operations 12,408,625; stream SHA-256 `464865e6bb7ae2fbd162e5bc0ecd78b48fe620b4b24d299803d88ea02a506bcd`; snapshot `/workspace/project/ecdsa-research/scratch/e022-ops-464865e6bb7ae2fb.bin`.
- Independent paired 10K full audit: CPU/GPU exact agreement; E022 and E020 have identical value/phase/ancilla/union failure sets (15/11/0/17, zero one-sided failures in every channel). E022 mean T=727,250.049 versus E020 727,251.075, an exact common-input saving of 1.0265 mean T (10,265 total). Pack SHA-256 `540208fda852c0175c31943f7cb599064adfb72239c9e02f90049e302108d258`.
- Protected evaluator on the distinct hash corpus: 14 classical mismatches, nine phase-garbage batches and zero ancilla garbage. These counts are not evidence of an error-rate improvement because the corpus changed.
- Decision: `RESEARCH_PROMISING` exact structural T reduction, but not the measured-score incumbent because its distinct official mean rounds higher. Retain as a composable building block; E020 remains the best measured score at 896,032,212.
- Next: E023 trims only tick 1's forward boundary-compare window from 21 to 20 bits, predicting -1.5 mean T with value and ancilla paths unchanged; screen phase error on fixed inputs.

## Experiment E023 — trim the forward boundary compare only at tick 1

- Timestamp: 2026-11-05 UTC; additional-sprint experiment 13; transient candidate over exact source commit `963dd6da`; stream SHA-256 `880bc869429197876f965c69ea34ede0ba31ca3837281841d6b9a350f6a3e7e5`.
- Hypothesis/change: change `LL_CMPF` from `0-200:1` to last-match schedule `0-200:1,1-1:0`, reducing each of tick 1's three forward boundary comparisons from 21 to 20 bits. This is phase-approximate; value and ancilla paths should be unchanged.
- Resource evidence: Q=1,238; protected mean T=723,777.784, rounded score `1,238 * 723,778 = 896,037,164`, 4,952 worse than E020. Emitted operations 12,408,332; snapshot `/workspace/project/ecdsa-research/scratch/e023-ops-880bc86942919787.bin`.
- Independent 10K full audit: CPU/GPU exact agreement; value/phase/ancilla/union 15/9/0/17. Value, ancilla and union failure sets were identical to E022; phase changed in both directions (five E022-only, three E023-only). Mean T=727,262.681, +12.632 versus E022 on identical inputs despite the lower static expectation, because the different HMR stream changes downstream executed branches.
- Independent 100K GPU screen: value/phase/ancilla/union 181/129/0/206, union error 0.206%, mean T=724,015.934. Against E011's fixed 100K counts 181/100/0/200, union rises by 0.006 percentage points; an unpaired normal approximation is about `+0.006 +/- 0.039` percentage points at 95%, so the error-rate change is inconclusive. The candidate is nevertheless dominated by E011/E020 on measured score and observed union count.
- Protected distinct-corpus result: 18 classical, nine phase-batch, zero ancilla failures; not comparable as an error-rate estimate.
- Decision: `REJECTED`; revert the compare trim and retain E022's exact tick-56 carry change only.

## Two-hour checkpoint (additional sprint)

- Elapsed wall time: 2h03m; completed 13 additional experiments (E011-E023); 47 experiment slots and about 3h57m remain under the authorized maxima.
- Official-valid-best: unchanged verified baseline `667ce8d4d8774be3edde6a8c9777bb04a1876bf8`, Q=1,243, T=721,156.073, score=896,396,908, validated 9,024/9,024 on its preserved stream.
- Best measured research score: E020 source `234a6b2b`, Q=1,238, T=723,774.397, rounded score=896,032,212, improvement 364,696 (0.040685%); not official-valid on its hash corpus.
- Exact structural building block retained: E022 source `963dd6da`, common-input T saving 1.0265 with identical 10K failure sets, but distinct-corpus score 896,058,210; therefore it is not the measured-score incumbent.
- Empirical score/error frontier remains E010 (score 896,095,599; observed 100K union 0.198%) and E011/E020 (score 896,032,212; E011 observed 100K union 0.200%). E023 at score 896,037,164 and union 0.206% is dominated.
- GPU integration remains validated: E022 and E023 full v1 CPU/GPU audits had zero disagreements in value, phase, ancilla and executed-Toffoli data; the v2 100K screen completed for E023. Official CPU evaluation remains authoritative.
- New closures: legacy `BACK_SEAM_FUSE` is unreachable on the active Leapfrog seam path; tick-56 kept carries are an exact paired T saving; tick-1 compare truncation is not useful on the current frontier.
- Next: extend the exact kept-carry schedule one more tick (E024, `LFG_KFWD=1-57`) to test the adjacent planner boundary, then perform a histogram-guided `LFQ_RAILWIN` one-bit trim only if it maps to a small, auditable repair set.

## Experiment E024 — extend exact forward fold kept-carry schedule through tick 57

- Timestamp: 2026-11-05 UTC; additional-sprint experiment 14; transient candidate over source/checkpoint `963dd6da`/`53a186bc`; stream SHA-256 `3b41245ec95229fbe9de7382a5f8b2f0c68e31d6815a00aeb877008e854ba2fa`.
- Hypothesis/change: extend `LFG_KFWD` from `1-56` to `1-57`, testing whether the adjacent tick can retain the same exact MAJ carry representation profitably.
- Resource evidence: Q=1,238; protected mean T=723,778.122, rounded score 896,037,164, 4,952 worse than E020. Emitted operations 12,408,716; snapshot `/workspace/project/ecdsa-research/scratch/e024-ops-3b41245ec95229fb.bin`. The final static expected T was one higher than E022 after downstream passes.
- Independent 10K full audit: CPU/GPU exact agreement; value/phase/ancilla/union 15/11/0/17 and every failure set identical to E022. Mean T=727,250.014, an observed saving of only 0.0346 T (346 total).
- Statistical check from the 156 full 64-shot pack batches: cluster-robust SE=0.03493, 95% interval `[-0.1036,+0.0344]`, t about -0.99, p about 0.32. The tiny observed saving is not distinguishable from zero, while protected score and final static expectation are worse.
- Protected distinct-corpus result: 23 classical, 14 phase-batch and zero ancilla failures; not comparable as an error-rate estimate.
- Decision: `INCONCLUSIVE` and product-negative on measured score; revert to `LFG_KFWD=1-56` rather than retain an unsupported extension.
- Next: E025 performs a source-level census of `LFQ_RAILWIN` and, only if a single-bit/small-site setting exists, measures that focused phase-only approximation.

## Experiment E025 — one-bit rail-erase compare trim at maximum-delta sites

- Timestamp: 2026-11-05 UTC; additional-sprint experiment 15; transient candidate over E022/checkpoint `963dd6da`/`53a186bc`; stream SHA-256 `f34835aac890860336df175a90771e9e32bf0513e56c0c532983ad98f2c561e1`.
- Census/hypothesis: a behavior-preserving `LFQ_RAILWIN=999` trace found 43 active multiply-phase repairs with delta histogram `{3:1,4:4,7:3,10:3,13:1,14:1,15:3,18:1,20:1,21:1,24:3,25:1,26:2,29:2,31:1,32:1,35:3,37:2,42:1,43:2,46:2,47:1,48:3}`. Pinning `LFQ_RAILWIN=47` therefore leaves 40 sites full and omits only bit 0 at the three delta-48 HMR-conditioned phase repairs.
- Resource evidence: Q=1,238; protected mean T=723,792.746, rounded score `1,238 * 723,793 = 896,055,734`, 23,522 worse than E020. Emitted operations 12,408,625; snapshot `/workspace/project/ecdsa-research/scratch/e025-ops-f34835aac8908603.bin`. Downstream simplification neutralized the predicted -1.5 static T: final op count and static expected T matched E022, although the stream hash differed.
- Independent 10K full audit: CPU/GPU exact agreement; value/phase/ancilla/union 15/11/0/17, with every failure mask and executed T exactly identical to E022 (mean 727,250.049).
- Independent 100K GPU screen: value/phase/ancilla/union 181/127/0/203, union error 0.203%, mean T=724,018.070. Relative to E011's same-input 181/100/0/200 and T=724,019.495, the candidate shows 1.425 lower mean T but three more union failures; the small error difference is statistically inconclusive and the protected score is worse.
- Protected distinct-corpus result: 17 classical, 12 phase-batch and zero ancilla failures; not comparable as an error-rate estimate.
- Decision: `REJECTED` as dominated/non-improving. Remove `LFQ_RAILWIN`; retain E022 only.
- Next: E026 evaluates a direction-isolated one-bit `LFA2_FWIN` reduction if source census confirms a concrete T mechanism and manageable affected scope.

## Experiment E026 — reduce the forward A2 fold window from 58 to 57 bits

- Timestamp: 2026-11-05 UTC; additional-sprint experiment 16; candidate source commit `02970ab351862ced06af4e116e49a1f14c6cf2f3`.
- Parent: retained exact E022 source `963dd6da` plus later research log checkpoint `53a186bc`.
- Hypothesis/change: set `LFA2_FWIN=57` while leaving `LFA2_RWIN=58`. This shortens 121 forward A2 folds at ticks 1-121 and the LFQ tail fold at tick 122 by one bit; it is approximate because a carry/borrow escaping the shorter window changes the classical payload.
- Resource evidence: Q=1,238; protected mean T=723,620.920, rounded score `1,238 * 723,621 = 895,842,798`. This is 189,414 better than E020 and 554,110 (0.061817%) better than the original score 896,396,908. Native Toffoli fell by 190, final static expected T by about 153.4, and emitted operations by 4,767 to 12,403,858. Stream SHA-256 `230d7ec867e5ba2a4bdee443f01707a37ce74b32ebdc1b38764c2b95bda9d554`; snapshot `/workspace/project/ecdsa-research/scratch/e026-ops-230d7ec867e5ba2a.bin`.
- Independent 10K full audit: CPU/GPU exact agreement; value/phase/ancilla/union 15/8/0/17, mean T=727,123.836. Versus E022, value and ancilla sets are identical; phase has seven E022-only and four E026-only, union has one one-sided failure each way, and mean T falls by 126.213. Versus E011, value/ancilla are identical, union adds one candidate-only failure and mean T falls by 128.423.
- Independent 100K GPU screen: value/phase/ancilla/union 187/133/0/216, union error 0.216%, mean T=723,866.541. Compared with E011's 181/100/0/200, union rises by 0.016 percentage points (about +1.44 expected failures per 9,024); the difference is smaller than the roughly 0.04 percentage-point unpaired 95% uncertainty at 100K.
- Protected distinct-corpus result: 19 classical, 13 phase-batch and zero ancilla failures; this is not a global error estimate and the candidate is not official-valid at its base hash.
- Independent 1M GPU screen: value/phase/ancilla/union 1,981/1,213/0/2,190, giving value 0.1981%, phase 0.1213% and union 0.2190%; mean T=723,633.890124. The validated deterministic input cache is 192,000,048 bytes, SHA-256 `6c769b05b596acd10bdf31e3678073af224b4994d0f6247912f67d8eefa7c721`, and its first 100K records are byte-identical to the established cache. Local pack SHA-256 `ac2a8e05c24b55a3cb8a35629ea1b4c0dc1eaa4237fe2dec675690d1159d739d`.
- GPU throughput: parallel exact CPU-reference cache generation 55.900 s; 741,947,416-byte pack build 294.155 s; RTX 4090 replay 167.447 s wall (165.647 s kernel, 1.307 s load, 0.041 s H2D). A workspace FUSE close stall invalidated the first attempted copy, so the accepted pack was regenerated and hashed entirely on local `/tmp` before replay.
- Boundary analysis: the candidate differs exactly when the fold's true signed correction leaves `[0,2^57)` (or `[0,2^55)` at the LFQ tail), i.e. a carry/borrow crosses the omitted bit. Eight independent arithmetic vectors (four in-range, four boundary escapes) passed `/workspace/project/ecdsa-research/scratch/e026_boundary_check.py`; a concrete isolated A-tick witness was also derived, though production reachability of that exact tuple is not claimed.
- Decision: `RESEARCH_PROMISING` and new measured-score incumbent. Its 1M union error estimate is 0.2190%, with zero ancilla failures; it remains separate from official validity. A same-corpus 1M preserved-baseline screen is queued for a statistically meaningful error delta.
- Next: complete the 1M screen, then test the smallest exact tick-58 kept-carry extension or the symmetric tail-only `LFQ_E3WIN=3` approximation on top of this candidate.

## Experiment E027 — narrow only the non-A LFQ tail fold window

- Timestamp: 2026-11-05 UTC; additional-sprint experiment 17; candidate source commit `0a3b8df7c0f457577618350d66a5ce7b43555ac2`.
- Parent: E026 source `02970ab3`. Hypothesis/change: add `LFQ_E3WIN=3`, changing only tick 122 from forward/reverse effective windows 55/56 to 54/55. This is a two-site classical-window approximation; all A ticks remain E026.
- Resource evidence: Q=1,238; protected mean T=723,623.571, rounded score `1,238 * 723,624 = 895,846,512`, 3,714 worse than E026 but 185,700 better than E020. Final static expected T fell by 3.0 and native Toffoli by four; emitted operations 12,403,797. Stream SHA-256 `d2e4a0277b498838ca2e44a94ad2b8eca98a809e614728ce5b1f03e7d414a3bb`; snapshot `/workspace/project/ecdsa-research/scratch/e027-ops-d2e4a0277b498838.bin`.
- Independent 10K full audit: CPU/GPU exact agreement; value/phase/ancilla/union 15/10/0/16, mean T=727,103.995. Versus E026, value and ancilla sets are identical; phase has three E026-only and five E027-only, union removes one E026 failure and adds none, and T falls by 19.8404 mean.
- Independent 100K GPU screen: value/phase/ancilla/union 187/122/0/211, union error 0.211%, mean T=723,863.945. Versus E026 on identical inputs, value/ancilla are stable, phase falls by 11, union by five, and mean T by 2.59589.
- Protected distinct-corpus result: 19 classical, eight phase-batch and zero ancilla failures; not a global error estimate.
- Decision: retain commit as an approximate Pareto candidate (slightly lower observed error and paired T, slightly worse protected score). Restore E026 source for score-first continuation; E027 is not the score incumbent.

## Experiment E028 — reduce the reverse A2 fold window from 58 to 57 bits

- Timestamp: 2026-11-05 UTC; additional-sprint experiment 18; candidate source commit `41905c8441ebba2fa2749be3b63946f9802a4ad0`.
- Parent: E026 forward-window candidate. Hypothesis/change: set `LFA2_RWIN=57`, making both main A2 directions use a 57-bit fold window. This affects 121 reverse A ticks plus the reverse LFQ tail and has the same carry/borrow escape failure mechanism as E026.
- Resource evidence: Q=1,238; protected mean T=723,486.083, rounded score `1,238 * 723,486 = 895,675,668`, 167,130 better than E026 and 721,240 (0.08046%) better than the original baseline. Emitted operations 12,399,290; stream SHA-256 `1828f6eb8b876c99b6d0d1da5514d174fbee7459afc32b11b2073f27ebf00d01`; snapshot `/workspace/project/ecdsa-research/scratch/e028-ops-1828f6eb8b876c99.bin`.
- Independent 10K full audit: CPU/GPU exact agreement; value/phase/ancilla/union 16/12/0/17, mean T=726,959.725. Versus E026 it adds one value-only failure, moves phase four out/eight in, keeps union count tied with one failure each direction, and saves 164.111 mean T.
- Independent 100K GPU screen: value/phase/ancilla/union 191/126/0/212, union error 0.212%, mean T=723,714.767; versus E026, value +4, phase -7, union -4, and T -151.774.
- Independent 1M GPU screen: value/phase/ancilla/union 2,024/1,183/0/2,218, union error 0.2218%, mean T=723,482.727916. Versus E026 on the same corpus: value +43, phase -30, union +28 and T -151.162208. The union difference is +0.0028 percentage points with an unpaired 95% interval about `[-0.0102,+0.0158]`, not significant. Pack SHA-256 `9628370d81306a163cfe70ded6c33a69b501788558f88f8c02972f94ead5680b`.
- Same-corpus preserved baseline at 1M: 1,948/1,129/0/2,118 failures (union 0.2118%), mean T=721,164.516171. E028 has +100 union failures (+0.0100 percentage points; not significant at this sample size) and sample-rounded score improvement 736,141. Baseline pack SHA-256 `293950abbc12b531539f12a2e0d4a9658cc86155881797c8cbdd4a8dbd44e1cb`.
- Protected distinct-corpus result: 22 classical, 12 phase-batch and zero ancilla failures; not a global error estimate.
- Decision: `RESEARCH_PROMISING` and new score/error Pareto incumbent, separate from official validity. Continue one bit at a time with reverse-only E029 and require the same staged screen.

## Experiment E029 — reduce the reverse A2 fold window from 57 to 56 bits

- Timestamp: 2026-11-05 UTC; additional-sprint experiment 19; candidate source commit `a5b673b7b19d8f7432c38b07c89f7ea0436935a0`.
- Parent: E028. Hypothesis/change: set only `LFA2_RWIN=56`, keeping forward at 57. This halves the reverse fold's remaining escape margin at 121 A ticks plus the LFQ tail.
- Resource evidence: Q=1,238; protected mean T=723,342.898, rounded score `1,238 * 723,343 = 895,498,634`, 177,034 better than E028 and 898,274 (0.10021%) better than the original baseline. Emitted operations 12,394,778; stream SHA-256 `fda90eaff41261fa7a0c2541a4f9aaeb7e7cd31208110a17a1aef7f54d391daf`; snapshot `/workspace/project/ecdsa-research/scratch/e029-ops-fda90eaff41261fa.bin`.
- Independent 10K full audit: CPU/GPU exact agreement; value/phase/ancilla/union 17/11/0/20, mean T=726,821.799. Versus E028 it adds one value and three union failures one-sided, moves phase seven out/six in, and saves 137.925 mean T.
- Independent 100K GPU screen: 195/129/0/220, union 0.220%, mean T=723,570.132; versus E028 value +4, phase +3, union +8, T -144.635.
- Independent 1M GPU screen: 2,081/1,250/0/2,288, union 0.2288%, mean T=723,334.623972. Versus E028, union +70 (+0.0070 percentage points, unpaired 95% interval approximately `[-0.00614,+0.02014]`) and T -148.103944. Versus preserved baseline, union +170 (+0.0170 points, about +1.534 expected failures per 9,024; interval excludes zero) while sample-rounded score improves 919,365. Pack SHA-256 `b3494f16fcceb42c965d9e31b713fce277c33e6583a428322493ce3bb75c8f86`.
- Protected distinct-corpus result: 22 classical, 13 phase-batch and zero ancilla failures; not a global error estimate.
- Decision: `RESEARCH_PROMISING` lower-score frontier point with a quantitatively resolved modest error increase; separate from official validity.

## Experiment E030 — reduce both A2 fold windows to 56 bits (in progress at checkpoint)

- Timestamp: 2026-11-05 UTC; additional-sprint experiment 20; candidate source commit `f55f2087f9526766438dbf7bad9cef82844ce272`.
- Parent: E029. Change: `LFA2_FWIN=56`, yielding symmetric forward/reverse 56-bit windows.
- Resource evidence: Q=1,238; protected mean T=723,181.424, rounded score `895,298,078`, 200,556 better than E029 and 1,098,830 (0.12259%) better than the original baseline. Emitted operations 12,390,074; stream SHA-256 `de078691b9f8c8a7224f5eef832ad914e676e3a6d01e4e8f837ae0596eb67b16`; snapshot `/workspace/project/ecdsa-research/scratch/e030-ops-de078691b9f8c8a7.bin`.
- Validation status: protected distinct corpus has 13 classical, eight phase-batch and zero ancilla failures. Fixed 10K/100K staged screening is in progress; do not classify/promote until its window-escape rate is measured.

## Twenty-experiment checkpoint (additional sprint)

- Completed: 20 (E011-E030, with E030 resource cycle complete and independent validation in progress); elapsed wall time about 3h30m, leaving 40 experiment slots and about 2h30m under the six-hour cap.
- Official-valid-best remains baseline `667ce8d4d8774be3edde6a8c9777bb04a1876bf8`, Q=1,243, T=721,156.073, score 896,396,908, 9,024/9,024 on its preserved stream.
- Best measured research score is E030 `f55f2087`, Q=1,238, T=723,181.424, score 895,298,078, improvement 1,098,830 (0.12259%), pending independent error classification. Best fully screened score point is E029 `a5b673b7`, union 0.2288% at 1M.
- Error/score frontier includes E010 (0.198% at 100K, score 896,095,599), E020/E022 exact-enhanced region (about 0.200% at 100K, score 896,032,212), E026 (0.2190% at 1M, score 895,842,798), E028 (0.2218% at 1M, score 895,675,668), and E029 (0.2288% at 1M, score 895,498,634).
- GPU integration is operational at 1M scale: parallel exact cache creation 55.9 s, pack about 294-298 s, RTX 4090 replay about 148-167 s. CPU/GPU v1 agreement remains exact on every promoted candidate; v2 packs provide aggregate screens. Workspace FUSE output stalls were isolated by regenerating authoritative packs on local `/tmp`.
- Key discoveries: tick-56 kept carries give a small exact paired saving; active Leapfrog seams bypass legacy seam fusion; rail-window truncation is neutralized downstream; fold-window width is the dominant current score/error knob, with one-bit direction-isolated steps delivering about 145-155 paired T each.
- Rejected/closed: E021 seam flag no-op, E023 compare trim dominated, E024 tick57 carry inconclusive, E025 rail trim dominated, tick58 exact kept-carry attempt hits a planner/cap cliff.
- Next: finish E030 1M if 100K is acceptable, then continue one-bit direction-isolated window exploration only while the measured error frontier remains quantitatively useful; otherwise recensus exact HB opportunities under the narrower-width context.

## E030 validation completion

- Fixed 10K CPU/GPU audit agreed exactly: value/phase/ancilla/union `17/10/0/19`, mean T `726658.8862`. At 100K: `204/133/0/227`, mean T `723408.61687`.
- Fixed 1M GPU screen: `2137/1250/0/2320`, union `0.2320%`, mean T `723178.977556`; pack SHA-256 `d9686c7d...`. Versus E029, union +32 and mean T -155.6464. Decision: retain as a score/error Pareto point.

## Experiment E031 — reverse fold window 56 to 55

- Candidate `6b6b89e83e`; Q/T/score `1238 / 723022.330 / 895101236`; 12,386,449 ops; stream `a260...`.
- Independent 10K `18/12/0/19`, 100K `215/117/0/236`, and 1M `2275/1289/0/2460` (union `0.2460%`, mean T `723019.235766`). The +140 union failures versus E030 are statistically resolved while T falls 159.742/shot.
- Decision: `RESEARCH_PROMISING`, approximate score/error frontier; zero ancilla failures, not official-valid.

## Experiment E032 — both fold windows to 55

- Candidate `34dddf98`; Q/T/score `1238 / 722861.598 / 894903156`; 12,383,485 ops; stream `2c2df...`.
- Independent 10K CPU/GPU exact `23/9/0/24`; 100K `236/140/0/265`; 1M `2405/1409/0/2604`, union `0.2604%`, mean T `722848.418885`. Versus E031, union +144 and T -170.817.
- Decision: retain approximate frontier point.

## Experiment E033 — reverse fold window 55 to 54

- Candidate `ffa12dcf`; Q/T/score `1238 / 722704.338 / 894707552`; 12,378,924 ops; stream `78f053...`.
- Independent 10K CPU/GPU exact `28/16/0/29`; 100K `274/160/0/302`; 1M `2687/1537/0/2866`, union `0.2866%`, mean T `722700.394237`. Versus E032, union +262 and T -148.025.
- Decision: retain approximate frontier point.

## Experiment E034 — both fold windows to 54

- Candidate `8f71b787`; Q/T/score `1238 / 722541.615 / 894506996`; 12,373,876 ops; stream `6efc...`.
- Independent 10K CPU/GPU exact `30/18/0/32`; 100K `294/167/0/317`; 1M `2939/1620/0/3121`, union `0.3121%`, mean T `722550.673784`. Versus E033, union +255 and T -149.720.
- Decision: retain approximate frontier point.

## Experiment E035 — reverse fold window 54 to 53

- Candidate `2045139d`; Q/T/score `1238 / 722409.844 / 894343580`; 12,369,442 ops; stream `f2b413...`.
- Independent 10K CPU/GPU exact `33/16/0/33`; 100K `351/194/0/373`, union `0.373%`, mean T `722632.56656`. Versus E034 at 100K, union +56 with a 95% unpaired interval excluding zero and T -150.683. The statistically clear jump makes 1M unnecessary.
- Decision: retain approximate frontier point; no official-valid claim.

## Experiment E036 — both fold windows to 53

- Candidate `9cbd0babd929af6afed45aaaa52fa5283e2c29be`; Q/T/score `1238 / 722241.134 / 894134358`, improvement `2262550` (`0.25240%`) over original; 12,364,984 ops; 2,571 classical bits; stream SHA-256 `400a337bbaaaa9ece82abffc17ed9c6877b8dffa22ec0894b464a697a7976bdb`.
- Protected distinct corpus: 37 classical, 17 phase-batch, zero ancilla failures. Independent 10K CPU/GPU agreement exact: `40/25/0/42`, mean T `725719.5819`. Independent 100K: `423/232/0/447`, union `0.447%`, mean T `722478.57436`.
- Versus E035 at 100K: union +74 (+0.074 percentage points; 95% unpaired interval excludes zero), mean T -153.9922. This is about +6.678 expected union failures per 9,024 inputs; 1M expansion was skipped.
- Decision: `RESEARCH_PROMISING`, current raw-score/error Pareto point, not official-valid. Stream snapshot `/workspace/project/ecdsa-research/scratch/e036-ops-400a337bbaaaa9ec.bin`.

## Experiment E037 — changed-context all-HB census at 53-bit windows

- Hypothesis: the narrower reverse folds might make previously room-gated exact `A2R_HB` ticks active. Transient change set `A2R_HB=1-121` over E036.
- Result: downstream composition emitted the exact E036 stream byte for byte: Q/T/score, op count, SHA-256 and protected result all identical. Intermediate planner counts differed but were fully neutralized.
- Decision: `REJECTED` structural no-op; revert to the isolated active schedule `1-107,110-112,115-117`. No independent replay was required.

## Experiment E038 — changed-context exact forward kept-carry tick 57

- Hypothesis/change: at the narrower forward window, extend exact `LFG_KFWD=1-56` through tick 57; the MAJ carry identity is exact and the five freed fold bits might remove E024's planner pressure.
- Result: Q=1,238, protected mean T=722,246.276, rounded score `894,140,548`, 6,190 worse than E036. Emitted operations 12,364,770; stream SHA-256 `cf45d25ce50de0aa995ef11d2c8e0142c3d70c5673c1753640883e33a8143946`. Protected distinct corpus: 42 classical, 21 phase-batch, zero ancilla failures.
- Decision: `REJECTED`. Three native/pre-pass Toffolis were removed, but downstream HMR/hold composition increased executed T by 5.142/shot. Revert to `LFG_KFWD=1-56`; no independent replay needed for a product-negative exact schedule.

## Experiment E039 — reverse fold window 53 to 52

- Candidate `aa6995d7ea5d5ab0019f0da830bb2a7581deb67d`; Q/T/score `1238 / 722108.318 / 893969704`, improvement 2,427,204 over original; 12,360,467 ops; stream SHA-256 `0beddda218d459a952f288c0781bda4ca24c190975f9350e4c8f010a38ec0de5`.
- Protected distinct corpus: 49 classical, 25 phase-batch, zero ancilla failures. Independent 10K CPU/GPU exact: `55/30/0/57`, mean T `725562.9437`. Independent 100K: `524/302/0/552`, union `0.552%`, mean T `722332.30890`.
- Versus E036 at 100K: union +105 (+0.105 percentage points, 95% unpaired interval `[+0.04321,+0.16679]`), mean T -146.2655; about +9.475 expected union failures per 9,024 inputs. Skip 1M.
- Decision: retain as lower-score/higher-error Pareto point, not official-valid.

## Experiment E040 — both fold windows to 52

- Candidate `57a62d9e5a912bd69ddb1fa9c59deda6349a0807`; Q/T/score `1238 / 721943.541 / 893766672`, improvement 2,630,236 (`0.29342%`) over original; 12,355,497 ops; stream SHA-256 `158c1f111c0c9c82b581ef688f4d43433f5ca6d8ba2cdc401c9c276acaf5214c`.
- Protected distinct corpus: 45 classical, 24 phase-batch, zero ancilla failures. Independent 10K CPU/GPU exact: `63/26/0/63`, mean T `725414.5590`. Independent 100K: `618/326/0/641`, union `0.641%`, mean T `722178.30496`.
- Versus E039 at 100K: union +89 (+0.089 percentage points, 95% unpaired interval `[+0.02151,+0.15649]`), mean T -154.0039; about +8.031 expected failures per 9,024. Skip 1M. Decision: retain as raw-score/error Pareto point, not official-valid.

## Thirty-experiment checkpoint (additional sprint)

- Completed: 30 (E011-E040); elapsed about 5h under the six-hour cap. Official-valid-best remains preserved baseline `667ce8d4`, Q=1,243, T=721,156.073, score 896,396,908, 9,024/9,024.
- Current raw-score research-best is E040 `57a62d9e`, Q=1,238, T=721,943.541, score 893,766,672; current fully screened low-score point is E039 `aa6995d7`, union 0.552% at 100K. E034-E039 form a monotone score/error frontier with zero observed ancilla failures.
- Exact changed-context censuses E037 (all HB) and E038 (forward kept-carry tick57) were respectively a final-stream no-op and product-negative. The dominant score mechanism remains one-bit direction-isolated fold-window reduction, with about 145-170 paired T saved per step and a convexly increasing carry-escape error rate.
- GPU path remains validated: every v1 10K candidate has exact CPU/GPU value, phase, ancilla and per-batch Toffoli agreement; fixed 100K screens finish in roughly one minute end-to-end. Official CPU evaluation remains the scoring/validation authority.
- Next: exact changed-context `A2R_SERIAL` tick99 at 52-bit reverse folds, then continue one-sided window steps only as explicit score/error Pareto experiments.

## Experiment E041 — changed-context exact reverse serial tick 99

- Hypothesis/change: extend exact `A2R_SERIAL=1-98` to tick 99 at the narrower 52-bit folds; five to six freed bits might make holding the serial carry cheaper than rebuilding it.
- Result: Q=1,238, protected mean T=721,951.835, rounded score `893,776,576`, 9,904 worse than E040. Emitted operations fell by 45 to 12,355,452, but executed T rose by 8.294/shot; stream SHA-256 `3c87f48cc2acf5ba4932344d177dbb307f71b898645af24aff089dd7eca98c23`.
- Decision: `REJECTED`, product-negative exact schedule. Restore `A2R_SERIAL=1-98`; no independent replay needed.

## Experiment E042 — reverse fold window 52 to 51

- Candidate `22c5782887b7ea6cabb88f5f1659e1dec9c35a99`; Q/T/score `1238 / 721803.453 / 893592114`, improvement 2,804,794 over original; 12,351,170 ops; stream SHA-256 `a31708bb9190289e9616dc716d647fc2298f55b8470e233eef9f7fbc141d550e`.
- Independent 10K CPU/GPU exact: `79/43/0/81`, mean T `725268.1959`. Independent 100K: `809/436/0/836`, union `0.836%`, mean T `722036.36569`.
- Versus E040 at 100K: union +195 (+0.195 percentage points, 95% interval `[+0.11996,+0.27004]`), mean T -141.9393; about +17.597 expected failures per 9,024. Skip 1M.
- Decision: retain as raw-score frontier point only; the convex error knee makes E040 the more useful lower-error rollback.

## Experiment E043 — both fold windows to 51

- Candidate `991ec5e7ad4a8ed511fc50e5c2b401e1bd8d0feb`; Q/T/score `1238 / 721636.855 / 893386606`, improvement 3,010,302 (`0.33582%`) over original; 12,346,782 ops; stream SHA-256 `1e68825c3ab3b99113dc903768be252d1c6fd7770b1fcb573b5e3a94b850da67`.
- Protected distinct corpus: 93 classical, 45 phase-batch, zero ancilla failures. Independent 10K CPU/GPU exact: `101/55/0/103`, mean T `725121.9757`. Independent 100K: `1009/533/0/1033`, union `1.033%`, mean T `721880.38640`.
- Versus E042 at 100K: union +197 (+0.197 percentage points, 95% interval `[+0.11267,+0.28133]`), mean T -155.9793; about +17.777 expected failures per 9,024. Skip 1M. Decision: retain raw-score frontier only.

## Experiment E044 — reverse fold window 51 to 50

- Candidate `48e5a6a4966e41f1121387f13d088def6c2c84e1`; Q/T/score `1238 / 721498.014 / 893214524`, improvement 3,182,384 (`0.35503%`) over original; 12,342,433 ops; stream SHA-256 `8369d32a5a4a1e9e9376618ee6ae73b885a2c8230670f871ba893420fa5bfb7d`.
- Protected distinct corpus: 145 classical, 51 phase-batch, zero ancilla failures. Independent 10K CPU/GPU exact: `144/77/0/145`, mean T `724970.8689`. Independent 100K: `1380/706/0/1407`, union `1.407%`, mean T `721734.35618`.
- Versus E043 at 100K: union +374 (+0.374 percentage points, 95% interval `[+0.27779,+0.47021]`), mean T -146.0302; about +33.750 expected failures per 9,024. No 1M. Decision: raw-score frontier only; the error trade is sharply dominated in practical usefulness by E040.

## Experiment E045 — both fold windows to 50

- Candidate `1fe9bfbd07970a97afbdf6ef8ba87706fff5ff07`; Q/T/score `1238 / 721352.984 / 893035014`, improvement 3,361,894 (`0.37504%`) over original; 12,337,830 ops; stream SHA-256 `31b4e395ff38a8d67a5222371ee56a007b4df60915c7ae64019c32375d18f9aa`.
- Protected distinct corpus: 151 classical, 62 phase-batch, zero ancilla failures. Independent 10K CPU/GPU exact: `199/103/0/200`, mean T `724808.5248`. Independent 100K: `1836/958/0/1855`, union `1.855%`, mean T `721583.25613`.
- Versus E044 at 100K: union +448 (+0.448 percentage points, 95% interval `[+0.33699,+0.55901]`), mean T -151.1001; about +40.428 expected failures per 9,024. No 1M. Decision: raw-score frontier only.

## Experiment E046 — exact forward kept-carry tick 57 at 50-bit folds

- Candidate/current `49eb09099cc12988e6f772c2136e7c05cc7e97ed`; exact change `LFG_KFWD=1-56` to `1-57` over E045. At this narrower context the held carry finally becomes product-positive.
- Q/T/score `1238 / 721347.821 / 893028824`, improvement 3,368,084 (`0.375736%`) over original and 6,190 over E045; 12,337,798 ops; stream SHA-256 `6b752869c4a4972ba05a4a54809d0be1ed9f92ca3f498fe06fd42ab586672333`.
- Protected distinct corpus: 137 classical, 53 phase-batch, zero ancilla failures; hash-derived counts are not paired. Independent 10K v1 CPU/GPU exact: `199/106/0/201`, mean T `724808.3777`. Versus E045 on identical inputs, value masks are identical, ancilla masks are identical/all zero, T falls 0.1471/shot; phase moves 45 out/48 in and union adds one due to changed HMR draw alignment.
- Fixed 100K v2: `1836/914/0/1854`, union `1.854%`, mean T `721579.79545`; pack SHA-256 `57ea6af8968579b832adbcd3c06129576878fd940c0aec93430fbc9f5cf5f19d`. A final paired-mask dump confirms all 100K value masks exactly identical to E045 and all ancilla masks identical/zero; phase moves 498 E045-only/454 E046-only, union 12/11, and T falls 3.46068/shot. Mask SHA-256 values are E045 `ceee234a627e8ef1dc34ec64c40ea3db1b5f8f7a3f5b95993d90f92a386aff91`, E046 `62af56194e600fd807f29af4bdd2fd5042ba9922057e81b277f831dfba727727`.
- Decision: `RESEARCH_PROMISING`, current raw-score best and exact scheduling improvement over E045. The fixed 100K aggregate screen shows no error-rate regression; no 1M expansion was run.

## Six-hour sprint stop checkpoint

- Completed 36 additional experiments, E011-E046. Stop reason: six-hour wall-clock budget, not the 60-experiment ceiling. The last build finished at about 5h52m and the final paired 100K audit at about 5h59m; remaining time was used to persist this checkpoint.
- Official-valid-best remains preserved baseline `667ce8d4d8774be3edde6a8c9777bb04a1876bf8`: Q=1,243, T=721,156.073, score 896,396,908, 9,024/9,024, stream `1c542d...cebb`.
- Raw-score research-best is E046 `49eb0909`: Q=1,238, T=721,347.821, score 893,028,824, improvement 3,368,084 (`0.375736%`). It is not official-valid at its base hash and inherits E045's 50-bit approximation plus an exact carry schedule.
- Practical score/error knee is E040 `57a62d9e`: Q=1,238, T=721,943.541, score 893,766,672; fixed 100K union `0.641%` (`618/326/0/641`). Preserved baseline fixed 1M estimate is union `0.2118%` (`1948/1129/0/2118`); the baseline's public ~0.21% was therefore independently reproduced at this scale.
- Raw frontier after the knee: E042 `0.836%`/893,592,114; E043 `1.033%`/893,386,606; E044 `1.407%`/893,214,524; E045 `1.855%`/893,035,014. All rates are fixed 100K union estimates except the independently established larger samples noted earlier; all observed ancilla counts are zero.
- GPU integration remains operational and validated on RTX 4090 / CUDA toolkit 12.8.93 (driver advertises CUDA 13.2). Architecture-matched v1 audits achieved exact CPU/GPU value, phase, ancilla and per-batch T agreement for every promoted stream; fixed 100K v2 packs typically took about 30-32 s plus 18-26 s replay. The legacy Dialog predicate was never used as a Skywalk/A2 oracle.
- Major discoveries this sprint: peak width reduced and held at 1,238; exact HB tick112/tick56 carry scheduling provides small composable T savings; tick57 becomes exact-product-positive only at 50-bit folds; one-bit fold-window steps save roughly 145-170 T but their carry-escape error becomes sharply convex below 53 bits; tail-only and rail/seam knobs did not explain the dominant failures.
- Rejected/closed classes include seam fusion no-op, terminal compare trim dominated, hosted m0 +76 T, serial tick99 product-negative at both wider and 52-bit contexts, HB broad census final-stream no-op, tick57 kept carry product-negative at 53 bits, rail trim neutralized downstream, and cap reductions below 1,238 blocked by planner/peak constraints.
- Exact restart action: branch from practical-knee E040 and investigate a per-site/direction window policy driven by arithmetic reachability rather than continuing global width descent. Retain E046 as the raw-score endpoint; consider 1M only if that extreme-error point becomes decision-relevant. Do not submit or publish.
