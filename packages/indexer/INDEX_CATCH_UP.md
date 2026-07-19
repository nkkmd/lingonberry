# Index catch-up contract

Contract version `1` recalculates the canonical-storage generation on every run.

- `up-to-date`: the persisted checkpoint matches storage and is not rewritten.
- `rebuilt`: the checkpoint is missing or stale and is atomically replaced after a successful rebuild.
- `failed`: storage, checkpoint parsing, or checkpoint persistence failed.

Corrupt and unsupported checkpoints fail closed. Catch-up does not overwrite them automatically.
