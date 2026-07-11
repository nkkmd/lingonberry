# Acceptance Policy

Lingonberry ingress can be configured with environment variables.

## Identity Claim requirement

```bash
export LINGONBERRY_REQUIRE_IDENTITY_CLAIM=true
```

Default: `false`.

When enabled, publish and archive import reject knowledge objects that do not contain at least one Identity Claim with `LB_IDENTITY_CLAIM_REQUIRED`.

## Unsupported Identity Key rules

```bash
export LINGONBERRY_UNSUPPORTED_IDENTITY_POLICY=reject
```

Allowed values:

- `reject` (default): reject with `LB_UNSUPPORTED_IDENTITY_RULE`; HTTP returns 422.
- `defer`: do not store the object; HTTP returns 202 with `status: deferred`, while CLI and archive import return `LB_IDENTITY_DEFERRED`.

`defer` persists the original publish request and validation reasons to `<state-dir>/quarantine.jsonl`. Quarantined objects remain outside the canonical catalog. Use `quarantine-list`, `quarantine-get <id>`, `GET /v1/quarantine`, or `GET /v1/quarantine/<id>` to inspect them.

## Defaults

The default policy preserves existing behavior:

```text
identity claim required: false
unsupported identity rule: reject
```


## Revalidation and promotion

Quarantined records can be revalidated against the current implementation and acceptance policy.

```bash
lingonberry quarantine-promote <quarantine-id>
lingonberry quarantine-resolutions
```

HTTP equivalents:

```text
POST /v1/quarantine/<quarantine-id>/promote
GET /v1/quarantine-resolutions
```

Promotion is allowed only when the record now evaluates to `Accept`. Successful and duplicate promotions are recorded in the append-only `quarantine-resolutions.jsonl` ledger. The original quarantine record is retained for auditability, and repeated promotion requests return the existing resolution instead of writing again.


## Batch revalidation

Unresolved quarantine records can be processed in bounded batches.

```bash
lingonberry quarantine-promote-batch [limit] [--dry-run]
```

The default limit is 100 and the maximum is 1000. `--dry-run` evaluates records without writing to canonical storage or the resolution ledger.

HTTP equivalent:

```text
POST /v1/quarantine/promote-batch
```

Example body:

```json
{
  "limit": 100,
  "dryRun": true
}
```

Only records without an existing resolution are selected. The response includes aggregate counts and the outcome for each scanned record, making the command suitable for a scheduler or periodic maintenance job.
