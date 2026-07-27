# Lingonberry v1.0.0 Pre-Version Candidate

**Status: designated; main-push qualification evidence pending inspection** | **Target release: v1.0.0** | **Parent issue: #109** | **Tracking issue: #332** | **Redesignation date: 2026-07-27**

## 1. Designated candidate

The redesigned Lingonberry v1.0.0 pre-version qualification candidate is the `main` squash-merge commit produced by PR #333:

```text
8c6b48082205a3af555130eec1f3e7d2ac8811fe
```

This exact commit includes the runtime-affecting Ed25519 publisher-signature enforcement merged through PR #331 and the candidate-redesignation record merged through PR #333. Later evidence-only or documentation-only commits do not silently move or redefine the candidate.

The candidate must be qualified by the `v1 candidate qualification` workflow through its `main` push trigger. The exact workflow run, artifact ID, artifact digest, binary SHA-256 values, security disposition, and documentation-walkthrough evidence must be recorded in the release evidence documents and issue #332 before the candidate is treated as fully qualified.

## 2. Superseded candidate

The previous candidate was:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

That candidate and its executable evidence are historical. PR #331 introduced runtime-affecting Ed25519 publisher-signature enforcement, so the previous candidate must not be represented as enforcing authenticated publishing.

The following evidence bound to the previous candidate is invalid for current release authorization and must not be silently reused:

- candidate binaries and binary digests;
- candidate qualification artifact;
- security and compatibility disposition;
- documentation walkthrough artifact;
- reference-host qualification;
- formal soak evidence.

## 3. New candidate scope

The redesigned candidate includes:

- the v0.9.0 production implementation and single-node operator baseline;
- the approved v1 compatibility policy and Rust public API audit;
- Ed25519 publisher-signature verification before acceptance, quarantine, duplicate/conflict classification, raw-request append, or canonical storage;
- stable malformed, invalid-signature, and verifier-error result codes;
- Rust ingestion coverage for valid and tampered signature vectors;
- updated signed fixtures preserving validation and conflict terminal-state coverage;
- qualification, documentation-integrity, security-review, walkthrough, and soak contracts.

## 4. Candidate qualification requirements

The designated candidate must produce a checksummed qualification artifact containing:

- the exact candidate commit;
- repository and workflow provenance;
- Ubuntu 24.04 x86_64 platform record;
- Rust, Cargo, and Node versions;
- release-built `lingonberry-storage` and `lingonberry-relay` binaries;
- binary SHA-256 values;
- per-gate JSON results and complete logs;
- aggregate summary;
- bundle `SHA256SUMS`.

All recorded gates must pass. The artifact candidate commit must equal `8c6b48082205a3af555130eec1f3e7d2ac8811fe`.

## 5. Evidence boundary

The successful PR #333 qualification run was a dry run and does not replace the required `main` push evidence. Main-push candidate qualification does not by itself authorize version preparation or publication.

After redesignation, the following remain mandatory:

1. independently inspect and record the main-push qualification artifact;
2. rerun the candidate security and compatibility review for the signature-enforcement delta;
3. rerun the reference-platform documentation walkthrough using candidate-built binaries;
4. complete privileged reference-host qualification;
5. execute the formal 72-hour soak and workload floors from the new candidate;
6. record residual-risk and release disposition;
7. prepare version `1.0.0` and release-specific documents;
8. review and merge the release PR, validate the merged commit, tag, publish, and record final evidence.

## 6. Change control

After redesignation:

- a runtime-affecting protocol, durable-format, CLI/HTTP contract, default, migration, recovery, or security-boundary change invalidates candidate-bound executable evidence and requires another explicit redesignation;
- an evidence-only or documentation-only correction must be reviewed for effect on operator acceptance and documentation freeze;
- a command, path, required setting, diagnostic code, or recovery-instruction change invalidates affected walkthrough evidence;
- no change may reuse old binary, walkthrough, reference-host, or soak evidence while silently moving the qualified SHA.

## 7. Post-merge qualification record

```text
candidate merge SHA: 8c6b48082205a3af555130eec1f3e7d2ac8811fe
qualification workflow run: Pending connector-visible inspection
qualification artifact ID: Pending
qualification artifact digest: Pending
lingonberry-storage SHA-256: Pending
lingonberry-relay SHA-256: Pending
standard CI result: PR checks passed; main-push result pending inspection
qualification disposition: Candidate designated; executable evidence not yet recorded
```
