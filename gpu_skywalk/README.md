# GPU Replay and Research Utilities

Tools for packing reversible secp256k1 circuits and input sets, replaying them on CUDA, and analyzing value, phase, and ancilla failures.

| File | Purpose |
| --- | --- |
| `src/main.rs` | Rust operation/input packer (`skywalk_gpu_pack`) |
| `skywalk_replay.cu` | CUDA circuit replay and screening |
| `compare_refs.py` | Reference-output comparison |
| `r125_filter_eval.py` | Partial/full screening comparison |
| `r125_select_pack.py` | Selects replay inputs using partial failure masks |
| `r125_nonce_reduce.py` | Reduces failure masks to nonce-level results |
| `r125_local_clean_search.py` | Bounded nonce search with exact replay of survivors |
| `Cargo.toml` | Depends on `../r125_opt`; also exposes the toolkit state dumper |

## Build

From this directory, with Rust 1.93.0 and the CUDA toolkit installed:

```bash
cargo +1.93.0 build --release --locked --bin skywalk_gpu_pack
nvcc -O3 -std=c++17 skywalk_replay.cu -o skywalk_replay
```

Operation streams, packs, inputs, masks, and executables are generated local data and are excluded from the source export. Inspect the Rust/CUDA entry points for the supported arguments and environment variables. The `r125_*.py` utilities expose argument help with `--help`.

The local clean-search script expects an E048 operation stream at `../e048_rebuild/ops.bin`, a compatible state dump at `/tmp/r125-e048-gcd402.gpu_state`, and the specialized executable `../gpu_toolkit/gpu_island2_r125_fold`. These must be prepared for the intended configuration before running the search. Approximate filters rank candidates; exact replay is a separate validation stage.

For the broader GPU toolkit workflow, see [the toolkit README](../gpu_toolkit/README.md). For recorded results and their limitations, see [the repository README](../README.md).
