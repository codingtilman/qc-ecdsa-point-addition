# autoresearch

This is an experiment to have the LLM do its own research on reversible quantum circuits for secp256k1 point addition, minimizing the Toffoli count × qubit width score.

## Setup

To set up a new experiment, work with the user to:

1. **Agree on a run tag**: propose a tag based on today's date (e.g. `oct28`). The branch `autoresearch/<tag>` must not already exist — this is a fresh run.
2. **Create the branch**: `git checkout -b autoresearch/<tag>` from current main.
3. **Read the in-scope files**: Read these files for full context:
   - `README.md` — repository context, circuit contract, editable paths, and memory notes. Treat historical scores and research claims as leads to verify.
   - `benchmark.json`, `benchmark.sh`, `setup.sh`, `Cargo.toml`, `rust-toolchain`, `src/lib.rs`, `src/bin/build_circuit.rs`, `src/bin/eval_circuit.rs`, `src/circuit.rs`, `src/sim.rs`, and `src/weierstrass_elliptic_curve.rs` — fixed build, serialization, evaluation, and scoring contract. Do not modify.
   - `src/point_add/` — the code you modify. Start with `mod.rs` and follow the code from there.
4. **Verify build prerequisites exist**: Check that the pinned Rust toolchain, a C compiler/linker, and cached Cargo dependencies are available. If not, tell the human to run `./setup.sh`. The benchmark runs offline after setup; it needs no GPU, training data, or tokenizer.
5. **Initialize autoresearch.tsv**: Create `autoresearch.tsv` with just the five-column header row described below. The baseline will be recorded after the first run. Preserve the existing `results.tsv`; it is a separate, harness-owned log and must not be initialized or edited by hand.
6. **Confirm and go**: Confirm setup looks good.

Once you get confirmation, kick off the experimentation.

## Experimentation

Each experiment builds and validates a reversible circuit on the CPU. Launch it as: `./benchmark.sh --note "what I tried"`. The script builds both release binaries offline, runs `build_circuit` to emit `ops.bin`, then runs the separate trusted `eval_circuit` to validate 9024 Fiat-Shamir-derived test inputs and write `score.json`. There is no fixed five-minute execution budget; runtime depends on compilation, circuit construction, and simulation.

**What you CAN do:**
- Modify anything inside `src/point_add/` — this is the only source tree you edit. Everything is fair game: point-addition algorithms, modular arithmetic, division and multiplication walks, squaring, qubit allocation and reuse, uncomputation, and circuit simplification. Capture useful findings in Markdown notes under `src/point_add/memory/`.

**What you CANNOT do:**
- Modify benchmark or implementation files outside `src/point_add/` for an experiment. The harness, benchmark and setup scripts, configuration, CI workflow, and shared circuit/simulator/reference-curve code are read-only. Do not manually edit `results.tsv`, `ops.bin`, or `score.json`; the harness produces them.
- Install new packages or add dependencies for experiments. You can only use what's already in `Cargo.toml` and `Cargo.lock`; keep `rust-toolchain` unchanged.
- Modify or bypass the evaluation harness. The trusted `src/bin/eval_circuit.rs` and its generated `score.json` are the ground truth. Preserve the four 256-element registers (quantum target x/y and classical offset x/y), correct affine addition, clean ancillas, reversible semantics, and zero residual phase.

**The goal is simple: get the lowest score.** The canonical score is `round(average executed Toffoli gates per shot) × qubits`, as written by the trusted evaluator to `score.json`. Optimize the product: fewer gates can justify more qubits, and fewer qubits can justify more gates. Clifford counts and emitted op counts are useful diagnostics, but they are not the objective. A candidate only counts if the full benchmark succeeds with correct outputs, clean ancillas, and zero residual phase. Establish the baseline from this checkout rather than assuming the README's historical reference circuit or old `results.tsv` rows describe the current code.

**Host memory** is a soft constraint. Some increase is acceptable for meaningful score gains, but circuit construction and simulation must fit the machine. Quantum qubit width is part of the score, not just a memory limit; use the evaluator's reported qubits rather than an internal estimate.

**Simplicity criterion**: All else being equal, simpler is better. A small improvement that adds ugly complexity is not worth it. Conversely, removing something and getting equal or better results is a great outcome — that's a simplification win. When evaluating whether to keep a change, weigh the complexity cost against the improvement magnitude. A 0.01% score improvement that adds 20 lines of hacky code? Probably not worth it. A 0.01% score improvement from deleting code? Definitely keep. An improvement of ~0 but much simpler code? Keep.

**The first run**: Your very first run should always be to establish the baseline, so you will run `./benchmark.sh --note "baseline"` with the circuit code as is.

## Output format

Once the script succeeds it prints a summary like this (illustrative reference numbers, not a measured baseline for this checkout):

```
  all 9024 shots OK

=== circuit metrics (secp256k1, n=256) ===
  avg executed Toffoli  : 3942753.000
  avg executed Clifford : 22627118.169
  total Toffoli (sum)   : 35579403072 over 9024 shots
  total Clifford (sum)  : 204187114357
  emitted ops           : 28677221
  qubits                : 2715

=== experiment OK ===
```

The canonical score is in `score.json`, with a top-level `score` and `metrics.toffoli` and `metrics.qubits` fields. The evaluator rounds the average Toffoli count before multiplying, so compare that score directly. `benchmark.sh` removes stale `ops.bin` and `score.json` before each run. Require a successful exit, the success marker, and a fresh score file; partial build diagnostics are not a valid result. You can extract the key metrics from the log file and read the score:

```
rg 'avg executed Toffoli|avg executed Clifford|emitted ops|qubits|experiment OK' run.log
cat score.json
```

## Logging results

When an experiment is done, log it to `autoresearch.tsv` (tab-separated, NOT comma-separated — commas break in descriptions). This is the untracked research ledger for keep/discard/crash decisions. The evaluator separately appends to the repository's existing `results.tsv`, whose eight columns are `timestamp`, `commit`, `toffoli`, `clifford`, `qubits`, `ops`, `correct`, and `notes`. Its `OK`/`FAIL` verdict records correctness, not whether you kept the change. Never append to, truncate, or rewrite that harness-owned file yourself. Build failures before evaluation may have no harness row, but still belong in `autoresearch.tsv`.

The TSV has a header row and 5 columns:

```
commit	score	qubits	status	description
```

1. git commit hash (short, 7 chars)
2. canonical integer score from `score.json` (e.g. 10704574395) — use 0 for crashes or failed validation; zero is a failure sentinel, not an improvement
3. qubits from `score.json` (e.g. 2715) — use 0 for crashes or failed validation
4. status: `keep`, `discard`, or `crash`
5. short text description of what this experiment tried

Example:

```
commit	score	qubits	status	description
a1b2c3d	10704574395	2715	keep	baseline
b2c3d4e	10677424395	2715	keep	reduce modular multiplication Toffoli count
c3d4e5f	10842570750	2750	discard	widen arithmetic scratch registers
d4e5f6a	0	0	crash	aggressive ancilla reuse leaves phase garbage
```

## The experiment loop

The experiment runs on a dedicated branch (e.g. `autoresearch/oct28` or `autoresearch/oct28-circuit0`).

LOOP FOREVER:

1. Look at the git state: the current branch/commit we're on
2. Tune `src/point_add/` with an experimental idea by directly hacking the code.
3. git commit the experimental changes under `src/point_add/`
4. Run the experiment: `./benchmark.sh --note "what I tried" > run.log 2>&1` (redirect everything — do NOT use tee or let output flood your context)
5. Read out the results: `rg 'avg executed Toffoli|qubits|experiment OK' run.log` and `cat score.json`; check the command's exit status and require the full benchmark to succeed.
6. If the command failed, the success marker is missing, or no fresh score file exists, the experiment failed. Run `tail -n 50 run.log` to read the compiler, builder, or evaluator diagnostics and attempt a fix. If you can't get things to work after more than a few attempts, give up.
7. Record the results in the tsv (NOTE: do not commit `autoresearch.tsv`, leave it untracked by git; preserve all harness-generated `results.tsv` rows and leave them out of experimental commits)
8. If score improved (lower) and validation passed, you "advance" the branch, keeping the git commit
9. If score is equal or worse, you git reset back to where you started, reverting the experimental `src/point_add/` changes while preserving both logs

The idea is that you are a completely autonomous researcher trying things out. If they work, keep. If they don't, discard. And you're advancing the branch so that you can iterate. If you feel like you're getting stuck in some way, you can rewind but you should probably do this very very sparingly (if ever).

**Timeout**: Use the first full baseline run to establish the wall-clock budget, including compilation, circuit emission, compression, and all 9024 validation shots. If a later run exceeds twice that budget, kill it and treat it as a failure (discard and revert).

**Crashes**: If a run crashes (OOM, or a bug, or etc.), use your judgment: If it's something dumb and easy to fix (e.g. a typo, a missing import), fix it and re-run. If the idea itself is fundamentally broken, just skip it, log "crash" as the status in the tsv, and move on.

**NEVER STOP**: Once the experiment loop has begun (after the initial setup), do NOT pause to ask the human if you should continue. Do NOT ask "should I keep going?" or "is this a good stopping point?". The human might be asleep, or gone from a computer and expects you to continue working *indefinitely* until you are manually stopped. You are autonomous. If you run out of ideas, think harder — read papers referenced in the code, re-read the in-scope files for new angles, try combining previous near-misses, try more radical architectural changes. The loop runs until the human interrupts you, period.

As an example use case, a user might leave you running while they sleep. If each experiment takes you ~5 minutes then you can run approx 12/hour, for a total of about 100 over the duration of the average human sleep. The user then wakes up to experimental results, all completed by you while they slept!
