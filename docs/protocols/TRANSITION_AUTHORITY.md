# Transition Authority Classification

## Status

Normative for v0.6.0 transition authority classification.

Rule version: `lb.transition.authority.v1`

## Principle

A structurally valid, correctly signed Transition Object is retained append-only regardless of authority classification. Authority determines semantic effect, not evidence retention.

| Classification | Retain transition | Apply to effective view |
|---|---:|---:|
| `authorized` | yes | yes |
| `unauthorized` | yes | no |
| `unknown` | yes | no; pending re-evaluation |

A relay MUST NOT delete or overwrite a transition solely because it is unauthorized or its authority is unknown.

## Inputs

Authority classification evaluates:

- target availability;
- `transitionPublisherKey`: publisher key that signed the transition publish request;
- `targetPublisherKey`: publisher key associated with the original target record, when known;
- zero or more verified delegation records;
- transition `issuedAt`.

Signature validity and structural validity are prerequisites and are evaluated separately.

## Classification algorithm

1. If the target Knowledge Object is unavailable, return `unknown` with basis `target-unavailable`.
2. If the target exists but `targetPublisherKey` is unknown, return `unknown` with basis `target-publisher-unknown`.
3. If `transitionPublisherKey` equals `targetPublisherKey`, return `authorized` with basis `original-publisher`.
4. Evaluate verified delegations from `targetPublisherKey` to `transitionPublisherKey`.
5. A delegation is applicable only when:
   - its issuer key equals `targetPublisherKey`;
   - its delegate key equals `transitionPublisherKey`;
   - its scope includes `transition`;
   - it is valid at the transition `issuedAt` timestamp;
   - it has not been revoked before `issuedAt`.
6. If at least one applicable delegation exists, return `authorized` with basis `delegated-publisher`.
7. If authority data is complete and no applicable authority exists, return `unauthorized`.
8. If required delegation or revocation evidence is unavailable or unsupported, return `unknown` rather than guessing.

`target-unavailable` and `target-publisher-unknown` are distinct. The former means the referenced Knowledge Object is not locally resolvable; the latter means the object is available but sufficient publisher evidence is absent.

## Fail-closed effect

Only `authorized` transitions affect the effective view. `unauthorized` and `unknown` transitions remain queryable as evidence but MUST NOT replace or withdraw the target in derived projections.

## Separation of concerns

The following are independent results:

- transition schema classification;
- transition signature verification;
- target availability classification;
- transition authority classification;
- effective-view conflict resolution.

A successful signature does not imply authority. Authority does not resolve conflicts between multiple authorized transitions.

## Re-evaluation

Authority classification is derived state. A transition classified as `unknown` MAY be re-evaluated when the missing target, target publisher, delegation, or revocation evidence becomes available. The stored transition bytes, identity, publisher signature evidence, and original receipt metadata remain unchanged.

Orphan retention and target-arrival behavior are specified by `lb.transition.orphan.v1` in [ORPHAN_TRANSITIONS.md](./ORPHAN_TRANSITIONS.md).
