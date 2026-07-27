# Lingonberry v1.0.0 Pre-Version Candidate

**Status: redesignation pending merge and main-push qualification** | **Target release: v1.0.0** | **Parent issue: #109** | **Tracking issue: #332** | **Redesignation date: 2026-07-27**

## 1. Redesignation rule

The `main` merge commit that introduces this redesignation record is designated as the new Lingonberry v1.0.0 pre-version qualification candidate.

The exact candidate SHA is not this pull-request head SHA and is not the earlier candidate. It is the resulting `main` merge commit, which must be qualified by the `v1 candidate qualification` workflow through its `push` trigger.

After merge, the exact SHA, workflow run, artifact ID, artifact digest, binary SHA-256 values, security disposition, and documentation-walkthrough evidence must be recorded in the release evidence documents and issue #332.

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

The designated merge commit must produce a checksummed qualification artifact containing:

- the exact candidate commit;
- repository and workflow provenance;
- Ubuntu 24.04 x86_64 platform record;
- Rust, Cargo, and Node versions;
- release-built `lingonberry-storage` and `lingonberry-relay` binaries;
- binary SHA-256 values;
- per-gate JSON results and complete logs;
- aggregate summary;
- bundle `SHA256SUMS`.

All recorded gates must pass. The artifact candidate commit must equal the pushed `main` merge commit.

## 5. Evidence boundary

Passing PR dry-run qualification does not designate the PR head as the candidate. Passing main-push candidate qualification does not by itself authorize version preparation or publication.

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

## 7. Post-merge record

To be completed from the `main` push evidence:

```text
candidate merge SHA: Pending
qualification workflow run: Pending
qualification artifact ID: Pending
qualification artifact digest: Pending
lingonberry-storage SHA-256: Pending
lingonberry-relay SHA-256: Pending
standard CI result: Pending
qualification disposition: Pending
```
