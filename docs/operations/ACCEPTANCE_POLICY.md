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
