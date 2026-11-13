# E048 local clean-nonce search — stopped checkpoint

- Stopped by user request on 2026-11-06 UTC.
- Candidate: E048 `bdeeee93`.
- Approximate range completed: nonces `10008192..=10012287` (4,096).
- R125 W56 fold passes: 1,337.
- GCD-261 passes: 1,938.
- Intersection packed for exact replay: 648 nonces / 331,776 shots.
- Pack completed in 43.950 seconds.
- Exact 512-shot replay was interrupted; no exact verdict was recorded and no
  nonce from this range may be called clean or dirty from this incomplete run.
- Resumable pack: `n10008192-s512.pack`.
- Pack SHA-256: `b5fe139f65af0eaafb27885e91b11203dfefe0c3fde962c066c11bef42f72404`.
- Search runner SHA-256: `1b62a391d4624349de413ba6f6f83463af1a0ac2108382d9bd72192913614728`.

Exact next action: run `skywalk_replay` on the preserved pack with
`SKYWALK_SCREEN_ONLY=1`, `SKYWALK_EARLY_BATCHES_PER_NONCE=1`, and a fresh
`SKYWALK_MASK_OUT`, reduce the 648 masks, then continue stages 1,024, 2,048,
and 9,024 only for survivors. Do not redo the approximate filters or packing.

No leaderboard submission, publication, upload, or submission API was used.
