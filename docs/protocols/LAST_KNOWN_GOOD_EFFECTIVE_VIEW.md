# Last-Known-Good Effective View

**Status: normative for the v1.0.0 pre-release implementation**  
**Rule version: `lb.transition.effective-view.last-known-good.v1`**

## 1. Purpose

This document defines how the relay preserves the most recently persisted complete effective-view result when newly observed transition evidence cannot produce a complete result.

The rule applies only to derived effective-view state. It does not modify, delete, roll back, or authorize canonical Knowledge Objects or Transition Objects.

## 2. Implemented state model

The checked-in relay does not persist separate `semanticCheckpoint` and `observationCheckpoint` records.

For each target, it currently uses:

- one persisted effective-view snapshot containing the most recent successfully materialized complete response;
- a newly calculated evidence generation and diagnostics for each read request;
- an in-memory stale response created by combining the persisted snapshot with the newly calculated incomplete observation.

The persisted snapshot path is derived from the FNV-1a 64-bit hash of the target identifier:

```text
<state-dir>/transitions/effective/<16-lowercase-hex>.json
```

The hash in the filename is a local path-key mechanism. It is not an authenticity or collision-resistance guarantee.

## 3. Complete current result

A current calculation is complete only when:

- the target object can be read;
- transition storage can be read and parsed;
- the evidence generation can be calculated;
- no transition or graph diagnostic is produced;
- replacement projection, when applicable, can read the replacement object.

A complete result has:

```json
{
  "effectiveView": {
    "classification": "original | replaced | withdrawn | ambiguous",
    "generation": "evidence:sha256:<hex>",
    "freshness": "current"
  },
  "evidenceObservation": {
    "generation": "evidence:sha256:<hex>",
    "snapshotClassification": "complete",
    "diagnosticSummary": {
      "total": 0,
      "returned": 0,
      "truncated": false,
      "byClassification": {
        "unsupported": 0,
        "corrupt": 0,
        "unreadable": 0
      }
    },
    "diagnostics": []
  }
}
```

The relay writes the complete response to the target's snapshot file before returning HTTP `200`. If snapshot persistence fails, the read returns HTTP `500` with `LB_EFFECTIVE_VIEW_STORAGE_ERROR`; the newly calculated complete result is not returned as a successful current view.

## 4. Incomplete observation with a persisted snapshot

When current evidence is incomplete and a readable persisted snapshot exists, the relay:

1. loads the prior complete response;
2. leaves its `effectiveObject`, `originalObject`, effective-view classification, and effective-view generation unchanged;
3. changes `effectiveView.freshness` to `stale`;
4. replaces `evidenceObservation` with the newly calculated incomplete observation;
5. returns HTTP `200`.

Example:

```json
{
  "effectiveView": {
    "classification": "replaced",
    "generation": "evidence:sha256:<prior-complete-generation>",
    "freshness": "stale"
  },
  "evidenceObservation": {
    "generation": "evidence:sha256:<current-incomplete-generation>",
    "snapshotClassification": "incomplete"
  }
}
```

The semantic generation and observation generation are intentionally different in this response.

The stale response is assembled in memory. The relay does not overwrite the stored complete snapshot with the incomplete observation.

## 5. Incomplete observation without a readable snapshot

If current evidence is incomplete and no readable prior snapshot can be loaded, the relay returns HTTP `200` with:

- the original target as both `effectiveObject` and `originalObject`;
- `effectiveView.classification` equal to `unresolved`;
- `effectiveView.generation` equal to `null`;
- `effectiveView.freshness` equal to `unavailable`;
- the current incomplete evidence observation and diagnostics.

This response does not authorize a replacement, withdrawal, or other transition effect.

A missing snapshot, unreadable snapshot, and snapshot JSON parse failure are currently collapsed into the same “no readable snapshot” behavior. The implementation does not expose a distinct snapshot-corruption diagnostic for this case.

## 6. Recovery to a complete result

A later read that can calculate a complete result evaluates the currently readable target and transition evidence from the beginning. It does not incrementally repair the stale response.

The new complete result may classify the target as:

- `original`;
- `replaced`;
- `withdrawn`;
- `ambiguous`.

After successful projection and snapshot persistence, the returned result is `current` and replaces the previous persisted snapshot.

Recovery is not required to preserve the previous classification. A newly complete evidence generation may validly produce a different semantic result.

## 7. Diagnostics and incompleteness

Incomplete observations expose bounded diagnostics as defined by the effective-view diagnostics contract. The current implementation emits at most 20 inline diagnostics and reports totals in `diagnosticSummary`.

Observed diagnostic classifications include:

- `corrupt`;
- `unreadable`;
- the reserved `unsupported` classification, although the current transition path does not normally produce it.

Diagnostic evidence MUST NOT be interpreted as valid transition authority merely because it is visible in a response.

## 8. Persistence behavior

The complete snapshot writer currently:

1. creates the parent directory;
2. writes canonical JSON to a `.json.tmp` file;
3. calls `sync_all` on the temporary file;
4. renames the temporary file over the target snapshot.

The implementation does not currently document or enforce:

- parent-directory `fsync` after rename;
- a separate snapshot manifest or digest;
- multi-generation snapshot retention;
- a lock that serializes concurrent readers or writers for the same target;
- compare-and-swap against an observation generation;
- a second “still current” check immediately before snapshot persistence.

Consequently, this document does not claim stronger crash consistency or concurrent-writer ordering than the checked-in implementation provides.

## 9. Failure boundaries

The last-known-good fallback is used only after the target object, transition log, and evidence generation have been read sufficiently to construct an incomplete observation.

The relay returns an HTTP error instead of a stale view when, for example:

- the target identifier is invalid;
- the target does not exist;
- target storage returns an error;
- the transition log cannot be opened, read, or parsed;
- evidence-generation hashing fails;
- persistence of a newly complete snapshot fails.

Replacement-object read absence and replacement-object backend errors are currently both converted into an unreadable evidence diagnostic and may therefore trigger a stale or unavailable HTTP `200` response.

## 10. Safety requirements

An implementation conforming to this checked-in rule MUST:

- never create a new semantic transition effect from an incomplete observation;
- preserve the prior complete effective object, classification, and generation when serving a stale response;
- clearly label the preserved result as `stale`;
- expose the newer incomplete observation separately;
- return `unresolved` and `unavailable` when no readable complete snapshot exists;
- avoid persisting the incomplete observation over the last complete snapshot;
- avoid altering canonical Knowledge Objects or Transition Objects as part of derived-state recovery.

## 11. Non-guarantees

A last-known-good effective view is not:

- proof that the underlying canonical evidence is still available;
- proof that the prior transition authority remains valid under an external revocation system;
- a signed or cryptographically authenticated checkpoint;
- a multi-node consensus result;
- a retention lease;
- permission to delete transition evidence;
- release qualification evidence.

## 12. Release boundary

Documentation checks, walkthroughs, ordinary CI, and local effective-view tests do not constitute the formal 72-hour soak or privileged reference-host qualification.

The fixed v1.0.0 candidate remains:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

Later documentation and tooling commits do not redefine that candidate.