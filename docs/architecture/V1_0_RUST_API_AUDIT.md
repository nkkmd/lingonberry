# v1.0 Rust Public API Audit

**Status: complete** | **Target: v1.0.0** | **Parent issue: #112**

This audit is complete. Mechanical inventory, classification rules, consumer review, error compatibility, deprecation policy, evidence mapping, and final workflow validation are recorded in the previous revision of this document. The final validated commit is `feb501a2de0cfa172cfc1495f510d5af18ce4740`; standard CI run 1169 and Rust public API audit run 6 both succeeded.

For the full audit record, use this file's history and the retained GitHub Actions artifact `rust-public-api-ff3ca598c983276664ee34b871ac184eb6536e7e` (artifact ID `8532290377`, SHA-256 `baecd8243eecb55ed89d5f5b0c28561113eb721359ed454ddfe2eeb4b2956548`).

The v1 compatibility policy must preserve the following conclusions:

- `pub` does not automatically mean supported third-party API.
- Stable and behavior-stable contracts are the documented protocol, identity, validation, storage, index, HTTP, operator, recovery, and machine-readable diagnostic semantics.
- File-loading helpers, heuristic shape detection, helper-module paths, runtime wiring, intermediate implementation structures, blanket implementations, debug formatting, and free-form prose ordering are not stable contracts.
- Parser byte and nesting limits are behavior-stable security boundaries.
- No exported item is removed for v1.0.0 solely to reduce surface area.
- No current external dependency on free-form validation error ordering was identified.
- Stable contract families are mapped to protocol, identity, validation, lifecycle, quarantine, crash-matrix, index, relay, migration, backup, restore, and operator acceptance evidence.
- Any compatibility-relevant source change after the recorded inventory requires regeneration and review.
