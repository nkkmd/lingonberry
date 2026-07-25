# Orphan Transition Retention

**Status: normative for the v1.0.0 pre-release implementation**  
**Rule version: `lb.transition.orphan.v1`**

## 1. Purpose

This document defines the checked-in relay behavior when a valid signed Transition Object is ingested before its target Knowledge Object is locally readable.

The transition is retained as append-only evidence. Target absence alone does not reject the transition, authorize it, or apply it to an effective view.

## 2. Ingestion prerequisites

The orphan rule is reached only after the transition request has passed the implemented ingestion checks:

- valid JSON;
- an envelope containing only `transition` and `publisher`;
- publisher public-key and signature shape validation;
- transition field, identifier, schema-version, type, timestamp-shape, and supersession validation;
- Ed25519 verification of the canonical unsigned request using the local `openssl` command.

A request that fails one of these checks is rejected before orphan retention.

## 3. Implemented target classification

After the transition record is appended, the relay calls the configured storage backend for `targetId`.

The implemented response classification is:

- `available` when the backend returns the target object;
- `missing` when the backend returns no object;
- `missing` when the backend returns an error.

Consequently, `targetStatus: "missing"` does not distinguish confirmed absence from a target-backend read failure.

The transition publish response does not currently include an authority classification or effective-view decision.

## 4. Durable transition record

A newly accepted transition is appended to:

```text
<state-dir>/transitions/append-only.jsonl
```

Each canonical JSON line contains:

```json
{
  "storedAtUnixSeconds": 0,
  "transitionId": "lb:transition:<id>",
  "targetId": "lb:obj:<id>",
  "request": {
    "publisher": {},
    "transition": {}
  }
}
```

The stored `request` is the parsed request serialized as canonical JSON. The implementation does not preserve the exact original HTTP byte sequence, whitespace, or member ordering.

The append operation writes one newline-terminated record and calls `sync_data` on the file. The implementation does not currently provide an interprocess append lock, a signed storage receipt, or a separate record digest.

## 5. Reevaluation intent

After appending the transition record, the relay appends a pending intent to:

```text
<state-dir>/transitions/reevaluation-queue.jsonl
```

The intent has this implemented shape:

```json
{
  "ruleVersion": "lb.transition.reevaluation.queue.v1",
  "status": "pending",
  "targetId": "lb:obj:<id>",
  "triggerTransitionId": "lb:transition:<id>"
}
```

The transition-log append and queue append are separate operations. They are not atomic.

If the transition append succeeds and the queue append fails:

- the transition may already be durable;
- the HTTP request returns `500` with `LB_TRANSITION_STORAGE_ERROR`;
- a retry may be classified as a duplicate;
- the failed request does not roll back the transition record.

The checked-in repository does not currently provide a background consumer that guarantees processing of these queue intents. The queue record is durable reevaluation intent, not proof that reevaluation occurred.

## 6. HTTP results

A newly stored transition returns:

```text
HTTP 201 Created
code: LB_TRANSITION_STORED
status: stored
```

The implemented success body contains only:

```json
{
  "status": "stored",
  "code": "LB_TRANSITION_STORED",
  "transitionId": "lb:transition:<id>",
  "targetId": "lb:obj:<id>",
  "targetStatus": "missing"
}
```

It does not include:

- `authority`;
- `effectiveView`;
- `applyToEffectiveView`;
- a queue identifier;
- a storage digest.

Target absence is not returned as HTTP `404` after the transition has otherwise passed validation and storage.

## 7. Duplicate and conflict behavior

Before appending, the relay scans the transition log for the same transition identifier.

An existing identifier with the same canonical request returns:

```text
HTTP 200 OK
code: LB_TRANSITION_DUPLICATE
status: duplicate
```

The duplicate response recalculates `targetStatus` at request time.

An existing identifier with different canonical request content returns:

```text
HTTP 409 Conflict
code: LB_TRANSITION_CONFLICT
```

A duplicate or conflict response does not append another transition record or another queue intent.

## 8. Effective-view boundary

Transition publication does not perform inline effective-view projection.

The implemented effective-view read path later reloads the append-only transition log and compares each transition publisher key with the target publisher key available from the target's stored raw publish request.

Current limitations include:

- no delegation evaluation in this path;
- no revocation evaluation in this path;
- no independent persisted orphan-authority record;
- no use of `issuedAt` as an authorization-time decision input;
- no guarantee that target arrival triggers an automatic background reevaluation.

When the target later becomes readable, a subsequent effective-view read can observe the retained transition without rewriting it.

If the target publisher evidence cannot be read, the effective-view path produces unreadable diagnostics and excludes the transition from a complete semantic projection.

## 9. Supersession boundary

An orphan transition's `supersedesTransitionIds` field is retained unchanged.

Publication-time ingestion validates identifier shape, non-emptiness, uniqueness, and prohibition on self-supersession. It does not require the referenced parent transitions or target to be locally available.

The effective-view graph evaluator later requires each referenced parent to exist in the loaded authorized transition set for the same target. Missing or unavailable parent evidence produces an inventory-conflict diagnostic and prevents a complete result.

Target absence is not proof that a referenced parent is invalid or unauthorized.

## 10. Failure and recovery boundaries

The implementation can return `LB_TRANSITION_STORAGE_ERROR` when it cannot:

- create the transition state directory;
- read or parse the append-only log while checking for an existing identifier;
- append or sync the transition record;
- append or sync the reevaluation intent.

There is no transactional rollback across the two append-only files.

Operators MUST treat a `500` after signature verification as potentially partially durable and MUST retry idempotently with the identical canonical request rather than changing the transition under the same identifier.

## 11. Safety requirements

A conforming implementation MUST NOT:

- mutate the stored Transition Object because the target later appears;
- invent target publisher authority;
- treat `targetStatus: "missing"` as an authorization decision;
- apply replacement or withdrawal semantics during transition publication;
- delete or rewrite another canonical object as part of orphan retention;
- represent queue persistence as completed reevaluation.

## 12. Non-guarantees

Orphan retention is not:

- proof that the target does not exist elsewhere;
- proof that the target-backend lookup succeeded;
- proof that the transition is authorized by the target publisher;
- proof that delegation or revocation checks passed;
- proof that reevaluation has run;
- a multi-node replication guarantee;
- permission to delete transition evidence;
- release qualification evidence.

## 13. Release boundary

Documentation checks, walkthroughs, ordinary CI, and local transition tests do not constitute the formal 72-hour soak or privileged reference-host qualification.

The fixed v1.0.0 candidate remains:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

Later documentation and tooling commits do not redefine that candidate.
