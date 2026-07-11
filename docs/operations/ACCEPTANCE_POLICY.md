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

`defer` is intentionally non-persistent in this version. A durable quarantine store can be added separately without weakening the guarantee that unverified objects never enter the canonical catalog.

## Defaults

The default policy preserves existing behavior:

```text
identity claim required: false
unsupported identity rule: reject
```
