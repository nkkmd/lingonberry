# Index ambiguity guard

Index lifecycle contract version `1` compares both canonical ID sets and deterministic record-content fingerprints.

A record fingerprint covers:

- carrier identity
- stored timestamp
- canonical object JSON

When storage and index contain the same canonical IDs but one or more fingerprints differ, verification returns:

- status: `inconsistent`
- code: `LB_INDEX_AMBIGUOUS`
- `ambiguousIds`: the affected canonical IDs

This state is fail-closed. It must not be treated as an up-to-date checkpoint or silently repaired from the ambiguous snapshot. Canonical storage remains the source of truth.
