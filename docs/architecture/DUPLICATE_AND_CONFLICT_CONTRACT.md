# Duplicate and Conflict Contract

**Status:** normative for the checked-in duplicate/conflict classifier  
**Contract version:** `1`

## 1. Scope

This document defines how Lingonberry classifies a finalized publish attempt when canonical storage may already contain a record with the same canonical ID or carrier identity.

The contract applies to the shared classified-append path used by supported storage backends. It defines only storage-ingest classification. It does not define semantic truth, transition supersession, effective-view selection, quarantine policy, replication conflict resolution, or distributed consensus.

## 2. Authoritative implementation

The checked-in contract is implemented by:

- `packages/core/src/duplicate_conflict.rs`;
- `packages/core/src/classified_append.rs`;
- the `StorageBackend` lookup and append behavior used by the file and SQLite implementations.

When this document conflicts with executable behavior, the implementation and its tests are authoritative for contract version `1`.

## 3. Inputs

Classification uses three incoming values:

1. `canonicalId`: the finalized canonical object identifier;
2. `carrierIdentity`: the stable identity derived from the complete publish request;
3. `canonicalJson`: the canonical JSON of the finalized knowledge object.

The classifier also receives up to two existing records:

- the record found by incoming canonical ID;
- the record found by incoming carrier identity.

Each existing record contributes its canonical ID, carrier identity, and stored canonical object.

Raw JSON formatting, object-key order, insignificant whitespace, and publisher-signature serialization are not compared directly. Existing objects are serialized with the canonical JSON routine before content comparison.

## 4. Classification table

| Classification | Condition | Public/internal code | Storage mutation |
|---|---|---|---|
| `New` | No record exists for either incoming identity | internal `LB_OBJECT_NEW`; successful publish normally reports `LB_OBJECT_STORED` | append through the backend |
| `ExactDuplicate` | The located record has the same canonical ID, carrier identity, and canonical JSON | `LB_OBJECT_DUPLICATE` | none |
| `CanonicalIdConflict` | Canonical ID matches but canonical JSON differs | `LB_OBJECT_CONFLICT` | none |
| `CarrierIdentityConflict` | Carrier identity matches but canonical JSON differs | `LB_OBJECT_CONFLICT` | none |
| `CrossIdentityConflict` | A located canonical ID or carrier identity is bound to a different counterpart | `LB_OBJECT_CONFLICT` | none |

Equal content does not make cross-identity aliasing valid. Both identity dimensions must preserve their existing one-to-one association.

## 5. Deterministic decision order

The classified append path performs the following sequence:

1. derive the incoming carrier identity from the publish request;
2. read the record indexed by incoming canonical ID;
3. scan the backend subscription result for incoming carrier identity;
4. classify the two optional records and the incoming values;
5. append only when the result is `New`;
6. return the existing record when the result is `ExactDuplicate`;
7. return an error for every conflict classification.

Within the classifier, carrier-identity evidence is evaluated first:

1. if a carrier-identity record exists, its canonical ID must match;
2. its canonical JSON must then match;
3. otherwise, if a canonical-ID record exists, its carrier identity must match;
4. its canonical JSON must then match;
5. only when neither record exists is the attempt `New`.

The lookup order is deterministic, but neither identity dimension is weaker. Any contradictory binding is a conflict.

## 6. Exact-duplicate behavior

An exact duplicate is idempotent success.

The classified append path:

- does not call the backend append operation;
- does not append another raw request record;
- does not create another canonical record;
- returns the existing record's canonical ID, carrier identity, object, and `storedAt` value;
- sets the append outcome's duplicate indicator to `true`.

A retry therefore preserves the first successful storage result instead of manufacturing a later storage timestamp.

## 7. Conflict behavior

Every conflict is fail-closed before append.

The path returns `LB_OBJECT_CONFLICT` and includes the contract version and concrete internal classification in the error message. It does not overwrite, merge, relabel, rebind, or append the incoming object.

A conflict is not a duplicate merely because:

- canonical content happens to match;
- one identity dimension matches;
- the attempt comes from a retry, import, restore, or administrative path;
- the caller prefers the newer timestamp or signature.

No timestamp, arrival-order, signature, actor, or lexicographic winner rule exists in this contract.

## 8. Storage-error boundary

Errors while deriving carrier identity, reading storage, scanning existing records, canonicalizing stored content, or appending a new record are storage/validation errors. They must not be reclassified as duplicate or conflict unless the classifier actually returns that classification.

The classifier assumes the optional existing-record inputs accurately represent the backend state observed by the caller. Contract version `1` does not provide transaction isolation across multiple processes or distributed nodes.

## 9. Backend and workflow boundary

Supported callers should route retries and import-like writes through the same classified append function when they require identical duplicate/conflict behavior.

This contract does not itself guarantee that every maintenance, quarantine, migration, recovery, or future replication path uses the classified append function. Each such workflow must document and test its own call path.

The file and SQLite backends are expected to preserve equivalent externally observable classification behavior, but their lookup cost, locking, transaction, and durability mechanisms may differ.

## 10. Relationship to other contracts

Duplicate/conflict classification occurs at canonical storage ingestion. It is separate from:

- schema validation and canonicalization;
- signature and authority evaluation;
- transition authorization;
- transition graph validation and supersession;
- effective-view derivation and last-known-good behavior;
- quarantine disposition and replacement workflows;
- replication or federation reconciliation.

A stored exact duplicate does not create a new transition or effective-view event. A rejected conflict does not become canonical evidence through this contract.

## 11. Security properties and limits

The contract prevents silent rebinding of canonical IDs and carrier identities in the observed backend state. It also prevents content replacement under an existing identity pair.

It does not prove that:

- the canonical object is true or trustworthy;
- the carrier identity belongs to an authorized actor;
- two isolated nodes observed the same prior state;
- concurrent writers cannot race without backend-specific serialization;
- a malicious or corrupted backend returned complete lookup results.

Those guarantees belong to validation, authorization, storage, operational, and future replication contracts.

## 12. Required tests

Contract version `1` requires coverage for at least:

- new object;
- exact duplicate;
- same canonical ID with different content;
- same carrier identity with different content;
- cross-identity aliasing even when content matches;
- exact-duplicate preservation of the existing record;
- no append on duplicate or conflict;
- file and SQLite externally observable parity;
- callers that claim retry, restore, promotion, or import parity.

## 13. Versioning and release boundary

Changing identity inputs, decision order, classification meanings, result codes, or mutation behavior requires an explicit contract-version review and corresponding tests.

The fixed pre-version release candidate remains `f9543019f2c219aea3b085ff90f2da201b268a48`. This documentation change does not redefine that candidate. Formal soak, privileged reference-host qualification, version preparation, release PR, tag, GitHub Release, and publication evidence remain separate release gates.
