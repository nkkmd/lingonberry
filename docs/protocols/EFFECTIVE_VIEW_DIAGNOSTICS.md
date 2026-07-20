# Effective View Diagnostics

## Status

Normative foundation for v0.6.0 public effective-view diagnostics.

Rule version: `lb.http.effective-view.diagnostics.v1`

## Purpose

The public read API exposes stable protocol-level diagnostics for incomplete evidence observations without exposing relay implementation details.

A diagnostic entry is machine-readable and implementation-independent:

```json
{
  "kind": "transition",
  "evidenceId": "lb:transition:t2",
  "classification": "corrupt",
  "reasonCode": "LB_EVIDENCE_PARSE_FAILED"
}
```

## Required fields

Every diagnostic entry MUST include:

- `kind`: `target`, `transition`, `delegation`, or `revocation`
- `evidenceId`: the stable protocol evidence identifier
- `classification`: `unsupported`, `corrupt`, or `unreadable`
- `reasonCode`: a stable code from this contract

A `supported` evidence entry is not a diagnostic.

## Optional fields

A diagnostic MAY include fields whose semantics are stable across conforming implementations:

- `ruleVersion`: the declared unsupported rule version, when known
- `digest`: the immutable `sha256:<hex>` carrier digest already used in the evidence-generation marker

These fields MUST NOT contain relay-local identifiers or mutable receipt metadata.

## Reason codes

Initial v0.6 reason codes:

| Code | Classification | Meaning |
|---|---|---|
| `LB_EVIDENCE_RULE_UNSUPPORTED` | `unsupported` | The declared rule or schema version is not supported. |
| `LB_EVIDENCE_PARSE_FAILED` | `corrupt` | Stored bytes are readable but do not parse under the declared representation. |
| `LB_EVIDENCE_VALIDATION_FAILED` | `corrupt` | Parsed evidence violates structural requirements. |
| `LB_EVIDENCE_DIGEST_MISMATCH` | `corrupt` | Stored bytes do not match the trusted immutable carrier digest. |
| `LB_EVIDENCE_SIGNATURE_INVALID` | `corrupt` | Required signature evidence is present but invalid. |
| `LB_EVIDENCE_BYTES_UNREADABLE` | `unreadable` | Payload bytes cannot currently be read, but a trusted marker digest exists. |
| `LB_EVIDENCE_INVENTORY_CONFLICT` | `corrupt` | The same evidence kind and identifier resolve to conflicting classifications or digests. |

Unknown internal errors MUST NOT be copied into `reasonCode`. A new externally observable condition requires a new versioned stable code.

## Forbidden public fields

The public response MUST NOT expose:

- filesystem or object-storage paths
- database row IDs or table names
- stack traces
- parser or library exception text
- worker, lease, process, or host identifiers
- credentials, tokens, request headers, or environment values
- relay-local ingestion sequence numbers
- mutable receipt timestamps unless defined by another protocol contract

Such details belong in operator logs and authenticated operational tooling, not the public protocol response.

## Body authority

Diagnostics are part of the authoritative response body under `evidenceObservation.diagnostics`. Optional response headers MUST NOT replace or alter their meaning.

## Ordering

Diagnostic entries are ordered deterministically by:

1. evidence kind order: `target`, `transition`, `delegation`, `revocation`
2. ASCII byte ascending `evidenceId`
3. ASCII byte ascending `classification`
4. ASCII byte ascending `reasonCode`

Exact duplicate diagnostic entries are collapsed. Conflicting diagnostics for the same `kind` and `evidenceId` are not silently selected.

## Safety

- Do not expose implementation exception strings as protocol fields.
- Do not classify unreadable evidence as not found.
- Do not omit a known unusable evidence marker from an incomplete observation.
- Do not use a diagnostic entry to authorize or apply a semantic transition effect.
- Do not silently map an unknown condition to an unrelated stable reason code.
