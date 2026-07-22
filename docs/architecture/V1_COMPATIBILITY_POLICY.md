# Lingonberry v1 Compatibility Policy

**Status: approved normative policy** | **Effective from: v1.0.0** | **Applies through: v1.x** | **Tracking issue: #113**

## 1. Purpose

This document defines the compatibility commitments that begin with Lingonberry v1.0.0.

It is the normative policy for deciding whether a proposed v1.x change is compatible, requires deprecation, requires migration support, or must wait for v2.0. It covers:

- protocol and canonical object behavior
- Rust library APIs
- HTTP and operator-facing APIs
- command-line interfaces and exit behavior
- configuration and environment variables
- durable storage formats and evidence artifacts
- migration, backup, restore, index rebuild, and rollback behavior
- deprecation and security exceptions

The mechanical Rust export inventory and classification are recorded in `V1_0_RUST_API_AUDIT.md`. Protocol, API, CLI, and storage freeze inputs from v0.9 remain supporting evidence, but this document supersedes candidate language when v1.0.0 is published.

## 2. Compatibility period

The v1 compatibility period begins at the published `v1.0.0` tag and ends when v2.0.0 is published.

Within v1.x:

1. stable contracts are not removed or incompatibly changed
2. additive behavior is permitted when older valid inputs and supported workflows continue to operate
3. behavior tightening requires explicit security, correctness, and migration review
4. deprecation does not itself permit removal during v1.x
5. changes to durable state require supported migration and recovery paths

Pre-release builds such as `v1.0.0-rc.N` are qualification candidates and may change before v1.0.0. Once v1.0.0 is published, the commitments in this document apply.

## 3. Compatibility classes

### 3.1 Stable contract

A stable contract is intentionally supported for third-party or operator use throughout v1.x.

Examples include:

- versioned protocol and schema behavior
- documented HTTP endpoints and response semantics
- documented CLI commands, flags, and machine-readable output
- documented durable storage and evidence formats
- Rust entry points explicitly classified as stable

Stable contracts may receive additive extensions, but their existing valid use must remain supported.

### 3.2 Behavior-stable contract

A behavior-stable contract is protected even when its implementation type or module layout is not independently frozen.

Examples include:

- canonicalization and identity derivation
- acceptance decision categories
- duplicate and conflict outcomes
- migration rejection rules
- parser resource boundaries
- backup verification and restore safety rules

### 3.3 Operator contract

An operator contract is a documented operational interface, including:

- command names and required arguments
- exit status categories
- environment and configuration keys
- service health and readiness behavior
- backup, restore, migration, doctor, and rebuild workflows
- documented diagnostic severity and machine-readable codes

Human-readable prose may improve without being byte-for-byte stable unless explicitly documented otherwise.

### 3.4 Workspace-internal or unstable surface

A public Rust symbol or helper is not automatically a stable contract. Items classified as workspace-internal or unstable may evolve, but:

- they are not removed solely to reduce surface area in v1.0.0
- deprecation and replacement guidance are required before intentional removal
- removal normally waits until v2.0
- public documentation must not present them as recommended third-party entry points

### 3.5 Implementation detail

The following are not compatibility promises unless a specification explicitly says otherwise:

- private or helper-module layout
- transaction decomposition
- temporary workspace paths
- debug formatting
- compiler-generated trait and auto-trait implementations
- internal batching, cursor, staging, and lock representations
- unversioned error prose

## 4. Protocol and canonical object compatibility

The following are stable or behavior-stable during v1.x:

- supported protocol, schema, archive, capability, and identity-rule versions
- accepted canonical object structure and required fields
- canonical JSON normalization and serialization behavior
- canonical identity and identity-key derivation rules
- signature verification semantics
- supported knowledge object types
- capability manifest semantics
- documented validation and acceptance categories
- versioned machine-readable rejection, defer, and error codes

### 4.1 Additive protocol changes

An additive v1.x change may:

- add an optional field with a defined default or absence meaning
- add a new capability that older peers can ignore safely
- add a new object type only when negotiation and unknown-type behavior remain well-defined
- add a new machine-readable code without changing the meaning of existing codes

An additive change must not cause a previously valid supported object to be rejected by default without an explicit security or correctness exception.

### 4.2 Breaking protocol changes

The following require v2.0 unless covered by the security exception process:

- changing canonicalization so an existing object obtains a different canonical representation or identity
- changing required field meaning
- removing a supported object type or schema version
- changing signature verification semantics incompatibly
- reusing an existing machine-readable code for a different condition
- changing a documented accept/defer/reject category incompatibly

### 4.3 Parser and resource boundaries

`MAX_JSON_INPUT_BYTES` and `MAX_JSON_NESTING_DEPTH` are behavior-stable security boundaries.

Increasing or decreasing them requires:

- parser boundary tests
- compatibility impact review
- denial-of-service and resource-use review
- release notes

A security-driven reduction may be shipped in v1.x only through the security exception process.

### 4.4 Error ordering and prose

Stable error compatibility is based on documented categories, codes, status, and severity—not free-form text ordering.

The following are not stable unless explicitly specified:

- exact prose wording
- punctuation
- debug output
- order of free-form validation messages

Deterministic order may still be tested as an implementation regression without becoming a public contract.

## 5. Rust API compatibility

The definitive v1 Rust surface classification is `V1_0_RUST_API_AUDIT.md` plus the successful mechanical inventory generated by `scripts/generate-rust-public-api.sh`.

### 5.1 Stable Rust entry points

Stable Rust entry points and behavior-stable categories include the documented protocol, identity, validation, storage abstraction, lifecycle, index verification, relay contract, migration, and recovery interfaces identified by the audit.

Within v1.x, a stable Rust item must not undergo:

- removal
- rename without a compatibility-preserving alias
- incompatible signature change
- narrowing of accepted valid inputs
- incompatible enum or result meaning change
- incompatible public field removal where direct construction is supported

### 5.2 Additive Rust changes

Additive items may be introduced in v1.x. Care is required for:

- adding enum variants used in exhaustive downstream matches
- adding required trait methods without defaults
- adding public struct fields to types commonly constructed with literals
- changing generic bounds or auto-trait properties

Such changes require downstream compatibility review even when source additions appear additive.

### 5.3 Workspace-internal exports

Exports classified as workspace-internal or unstable are outside the stable third-party Rust commitment. Current examples include file-loading convenience helpers, heuristic shape detection, runtime wiring, handler decomposition, and helper-module paths.

They may be deprecated in v1.x but are not removed before v2.0 except under an approved security emergency.

### 5.4 Mechanical inventory gate

Any compatibility-relevant Rust source change after the recorded audit requires:

1. regeneration of the public API inventory
2. comparison against the previous inventory
3. classification of additions and removals
4. successful standard CI and public API audit workflow
5. compatibility notes in the pull request

An unexplained public API removal or signature change is a release blocker.

## 6. HTTP and relay compatibility

Documented HTTP behavior is an operator and integration contract.

Protected behavior includes:

- endpoint method and path
- authentication requirement
- documented request schema
- documented response schema
- HTTP status semantics
- machine-readable error codes
- publish, retrieve, query, transition, and effective-view outcomes
- health and readiness semantics
- documented metrics and diagnostics

### 6.1 Compatible HTTP changes

Compatible changes may include:

- new optional response fields
- new endpoints
- new optional request fields
- additional diagnostic detail
- performance improvements that preserve semantics

Clients must be expected to ignore unknown optional response fields.

### 6.2 Incompatible HTTP changes

The following normally require v2.0:

- removing or renaming an endpoint
- changing a method or authentication requirement incompatibly
- removing a documented response field
- changing status or code meaning
- making an optional request field required
- changing pagination, ordering, or filtering semantics when documented as stable

## 7. CLI compatibility

Documented CLI commands are operator contracts.

Protected elements include:

- command and subcommand names
- required positional arguments
- documented flags and environment variables
- exit status categories
- machine-readable output schemas
- destructive-operation safeguards
- resume, verify, rollback, and dry-run behavior

### 7.1 Human-readable output

Human-readable output wording and layout may improve unless documentation marks it as parseable. Automation must use documented machine-readable modes and codes.

### 7.2 Exit status

Existing documented success and failure categories must retain their meaning. New distinct nonzero codes may be added when they do not reinterpret existing codes and are documented.

### 7.3 Removal and rename

A CLI command, flag, or configuration key may not be removed during v1.x. A rename requires:

- continued support for the old form
- a deprecation notice
- replacement guidance
- release notes

## 8. Configuration compatibility

Documented configuration files and environment variables are operator contracts.

Within v1.x:

- existing valid values remain accepted unless unsafe
- defaults do not change in a way that silently weakens safety or changes durable behavior without release notes
- new required configuration is avoided; new keys should have safe defaults
- unknown-key behavior remains documented and deterministic
- secret handling and file-permission requirements may be tightened for security with explicit upgrade guidance

## 9. Durable storage compatibility

Durable storage compatibility applies to supported single-node state, archives, journals, manifests, proofs, inventories, backups, and migration evidence.

Protected behavior includes:

- recognition of supported legacy formats
- rejection of unknown newer formats
- detection of corrupt, contradictory, partial, symlinked, active-target, or unverified state
- migration planning and evidence
- resumable and idempotent migration behavior where documented
- backup verification before restore
- isolated restore semantics
- index rebuild from canonical durable state
- rollback boundaries and refusal conditions

### 9.1 Forward compatibility

A v1 binary is not required to open storage written by a future unsupported major version. Unknown newer format versions must fail safely rather than being interpreted as older state.

### 9.2 Backward compatibility

A v1.x binary must support the documented migration path from every legacy state explicitly supported by v1.0.0, unless that support is deprecated with a separately published end-of-support policy that does not take effect before v2.0.

### 9.3 Additive durable fields

Additive fields are compatible only when older readers can safely ignore them or when the format version prevents unsafe older interpretation.

### 9.4 Storage format changes

Any durable format change requires:

- explicit format version disposition
- migration plan
- backup and isolated restore validation
- interrupted-operation recovery testing
- index rebuild and consistency verification
- rollback or documented point-of-no-return behavior
- release notes

Silent in-place reinterpretation of existing durable bytes is prohibited.

## 10. Migration compatibility

A supported migration must be:

- planned before mutation
- guarded by preflight validation
- idempotent or safely resumable where documented
- explicit about point-of-no-return boundaries
- verifiable after application
- accompanied by durable evidence
- able to reject unsafe source and target conditions

Migration success is not established solely by process exit. Post-migration verification and supported object lifecycle checks are required.

## 11. Backup, restore, rebuild, and rollback compatibility

### 11.1 Backup

A supported backup workflow must create verifiable evidence sufficient to reject incomplete or tampered backups.

### 11.2 Restore

Restore must support isolation from the active target and must reject dangerous target conditions. A restored state is not accepted until verification and object lifecycle checks succeed.

### 11.3 Index rebuild

Derived indexes may change internally, but the supported rebuild operation must reconstruct equivalent observable state from canonical durable records.

### 11.4 Rollback

Rollback support is limited to documented boundaries. When rollback is unsafe or impossible after a point of no return, the tooling must state this before mutation and preserve recovery evidence.

## 12. Deprecation policy

A v1.x deprecation requires:

- the deprecated contract or item
- replacement guidance
- first deprecated release
- earliest permitted removal release
- migration instructions when applicable

Stable v1 contracts are not removed before v2.0.

Workspace-internal exports may also use deprecation annotations to guide workspace and third-party users, but their removal still normally waits until v2.0.

Deprecation warnings must not expose secrets or destabilize machine-readable output.

## 13. Security and correctness exceptions

Security or data-integrity fixes may require behavior tightening during v1.x.

An exception requires:

1. documented vulnerability or integrity risk
2. assessment of affected stable contracts
3. evidence that a compatible fix is insufficient
4. migration or mitigation guidance where possible
5. regression and adversarial tests
6. explicit release notes
7. release-blocker review

A security exception may reject previously accepted unsafe input, reduce resource bounds, strengthen authentication, or refuse unsafe storage state. It must not be used as a general mechanism for convenience-driven breaking changes.

## 14. Versioning and release notes

Lingonberry follows semantic versioning for published releases:

- patch: compatible fixes and security corrections
- minor: compatible features and additive contracts
- major: incompatible contract changes

Every release that changes a protected contract must identify:

- affected compatibility area
- whether the change is additive, corrective, deprecated, or exceptional
- required operator action
- migration or rollback implications
- evidence used to qualify the change

## 15. Release qualification gate

A v1 release candidate is blocked if:

- a public Rust change lacks regenerated inventory and review
- a documented protocol/API/CLI contract changes incompatibly
- a durable format change lacks migration and recovery evidence
- a supported legacy state cannot be migrated and verified
- backup and isolated restore qualification fails
- index rebuild produces inconsistent observable state
- a machine-readable code is reused incompatibly
- a deprecated contract lacks replacement guidance
- public documentation contradicts this policy
- a security exception lacks explicit disposition

Historical v0.9 evidence establishes precedent but does not replace rerunning executable gates against the final v1 candidate commit.

## 16. Normative precedence

For v1.x compatibility questions, apply the following order:

1. this compatibility policy
2. versioned protocol and storage specifications
3. documented public API and operator contracts
4. `V1_0_RUST_API_AUDIT.md`
5. conformance fixtures and acceptance evidence
6. current implementation behavior

When implementation behavior conflicts with a higher-precedence contract, the higher-precedence contract governs unless a security exception is approved and documented.

## 17. Change-control checklist

Every pull request that may affect compatibility must answer:

- Which compatibility class is affected?
- Is the change additive, corrective, deprecating, or breaking?
- Does it change accepted input, observable output, codes, status, or ordering?
- Does it change durable state or recovery behavior?
- Is migration required?
- Are public API inventories and conformance fixtures updated?
- Are operator documentation and release notes updated?
- Does the change require a security exception?

Unanswered material compatibility questions are release blockers.
