# Index lifecycle contract

Contract version `1` treats canonical storage as the source of truth and the index as derived state.

A rebuild compares the canonical ID sets from storage and the rebuilt snapshot. The result reports:

- status: `consistent`, `inconsistent`, or `failed`
- storage and index record counts
- deterministic ID-set generations
- IDs missing from the index
- unexpected IDs present only in the index

Generation values use FNV-1a 64-bit over canonical IDs in ascending order. Machine codes, rather than human-readable messages, are the compatibility keys.
