# Lingonberry Conformance Suite

This directory contains implementation-independent fixtures for the Lingonberry external protocol contract.

## Run

Requires Node.js 20 or later and no third-party dependencies.

```bash
node conformance/run.mjs
```

The runner prints a machine-readable JSON result and exits non-zero when any case fails.

## Structure

- `manifest.v1.json`: versioned registry of conformance cases
- `run.mjs`: standalone JavaScript reference runner
- `canonicalization/`: canonical JSON input and expected byte fixtures
- `identity-key-v2/`: semantic identity key fixtures
- `identity-claims/`: identity claim validation fixtures

## Fixture rules

Each fixture case must:

1. have a stable case ID;
2. identify every rule version needed to interpret it;
3. keep input and expected output separate;
4. define exact expected bytes or exact machine-readable classification;
5. be shared by Rust and at least one non-Rust implementation;
6. never be regenerated automatically merely to make an implementation pass.

New fixture categories should cover valid, invalid, boundary, digest, signature, conflict, replacement, withdrawal, timestamp, and legacy behavior.

The specification is normative. `run.mjs` is a reference implementation used to detect contract drift.