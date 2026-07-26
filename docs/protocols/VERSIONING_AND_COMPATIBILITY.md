# Versioning and Compatibility

**Status: v1.0.0 pre-release normative contract** | **English is normative** | **Last reviewed: 2026-07-26**

## English

### 1. Scope

This document defines how Lingonberry interprets version identifiers and compatibility decisions. Version axes are independent. Support for one axis does not imply support for another, and an implementation MUST NOT infer a missing version from an unrelated version field.

This contract distinguishes:

- protocol and schema identifiers carried by objects or requests;
- rule-version identifiers used by canonicalization, identity, signatures, evidence generation, and derived-state processing;
- API contract versions;
- node-internal storage, journal, and proof formats;
- the repository release version.

The repository release version is packaging metadata. It does not replace any protocol, schema, rule, API, storage, journal, or proof identifier.

### 2. Independent version axes

| Axis | Identifies | Unknown or unsupported behavior |
|---|---|---|
| Protocol version | Semantic protocol contract | Reject or classify as unsupported; do not guess semantics |
| Schema version | Structural object or request profile | Reject unless explicit compatibility is declared and tested |
| Canonicalization rule | Deterministic byte serialization | Refuse operations requiring canonical bytes |
| Identity rule | Identity basis and identifier derivation | Reject or retain as unsupported evidence; do not derive an identifier |
| Signature rule | Signature target, algorithm, and encoding | Reject or classify the signature as unsupported |
| Derived-state rule | Authority, generation, supersession, or projection semantics | Preserve canonical evidence and fail closed for derived application |
| API version | HTTP or CLI request and response contract | Return an explicit versioned error |
| Storage format | Node-internal durable layout | Refuse unsafe write/open until a supported migration is available |
| Journal format | Transaction and recovery record layout | Refuse unsafe resume or mutation |
| Proof format | Backup, cleanup, replacement, or completion evidence | Reject the proof as unsupported |
| Repository release | Distribution and release packaging | No direct compatibility implication for the axes above |

### 3. Compatibility classifications

A producer/consumer pair is classified as exactly one of:

- `compatible`: the consumer can process the artifact without semantic loss under the referenced contract;
- `compatible-with-restrictions`: processing is allowed only under explicit, testable restrictions;
- `unsupported`: the identifier is known but the required behavior is not implemented;
- `unknown`: the identifier is not recognized;
- `invalid`: the identifier value, field combination, or artifact violates the selected contract.

`unknown` and `unsupported` MUST NOT be converted to `compatible` through fallback, prefix matching, nearest-version selection, or silent field removal.

### 4. Implemented v1 pre-release baseline

The current repository implements a narrow baseline rather than a general compatibility registry.

| Producer artifact or claim | Current consumer capability | Classification |
|---|---|---|
| Knowledge Object `schemaVersion: "0.1.0"` | Rust protocol validator and JSON Schema for `0.1.0` | `compatible` when all other validation succeeds |
| Any other Knowledge Object schema version | Current validator | `unsupported` or `invalid`; the validator emits a schema-version error |
| Transition Object `schemaVersion: "0.1.0"` | Current transition schema/validator | `compatible` when all other validation succeeds |
| HTTP publish request `schemaVersion: "0.1.0"` | Current publish request validator | `compatible` when all other validation succeeds |
| `lb.canonical.json.v1` behavior | Current canonical JSON implementation | `compatible` for the covered JSON domain |
| `lb.identity.key.v1` | Current Rust identity implementation | `compatible` for the implemented identity basis |
| `lb.identity.key.v2` | Conformance model and fixtures | `compatible` only where the v2 conformance contract is explicitly exercised; not a claim of complete production support |
| Unknown identity rule | Current validators/conformance model | `unsupported`; no identity fallback |
| Known rule-version fixtures in the conformance manifest | Conformance runner | `compatible` only for the behavior fixed by those fixtures |
| Unknown derived-state rule | Current runtime | No general negotiation mechanism; derived application MUST fail closed |

The constants currently exposed by `packages/protocol` include protocol, Knowledge Object schema, and HTTP publish request versions of `0.1.0`, plus separate archive and capability version identifiers. Their coexistence does not make them interchangeable.

### 5. Compatibility rules

A change is backward compatible only when an older conforming consumer can process the new producer output without changing any meaning required by the older contract.

A change is forward compatible only when a newer consumer explicitly declares and tests support for the older artifact. Merely ignoring unknown fields is not a forward-compatibility guarantee.

The following changes are breaking unless the governing contract explicitly says otherwise:

- changing canonical bytes or normalization rules;
- changing identity or digest input;
- changing signature targets, algorithms, or encodings;
- adding, removing, or changing required fields;
- changing identifier semantics;
- changing acceptance, authority, quarantine, or projection classifications;
- changing unknown-field behavior;
- changing durable recovery semantics.

An optional field is not automatically compatible. Compatibility depends on the selected schema, identity basis, signature target, and unknown-field policy.

### 6. Unknown and legacy input

A consumer receiving an unknown or unsupported version MUST:

1. preserve the original evidence when the surrounding storage contract permits preservation;
2. avoid canonical re-emission under a different rule;
3. avoid asserting a current identifier or valid signature for unprocessed semantics;
4. return or record a machine-readable unsupported/unknown classification;
5. fail closed for authority, supersession, effective-view, or other semantic application.

Legacy fixtures MUST state:

- the represented version or historical behavior;
- the expected acceptance, rejection, quarantine, or retention result;
- whether normalization is permitted;
- whether a current producer may re-emit the artifact;
- whether identity or signature validity changes during migration.

Legacy input MUST NOT be silently rewritten when rewriting could change identity, signature validity, provenance, authority, or effective-view meaning.

### 7. Migration boundary

Protocol/schema migration and node-internal storage migration are separate operations.

A storage migration MUST NOT silently upgrade protocol objects. A protocol migration MUST produce explicit new evidence or an explicitly specified transformed artifact; it MUST NOT mutate signed or identity-bearing historical bytes in place.

No document in this repository currently defines a universal automatic migration path between arbitrary protocol or schema versions. Absence of a migration specification means migration is unsupported, not implementation-defined.

### 8. Change procedure

A compatibility-affecting change requires all applicable items below:

1. specification update;
2. explicit decision identifying the affected version axis;
3. new identifier when the old contract cannot represent the change compatibly;
4. fixtures for accepted, rejected, unknown, unsupported, and legacy cases;
5. compatibility-matrix update;
6. production and conformance tests for the claimed support boundary;
7. migration and rollback instructions when durable state changes;
8. release notes describing producer and consumer impact.

Fixtures MUST NOT be regenerated merely to make an implementation pass. A changed expected result is a contract change and requires review.

### 9. Current non-claims

The v1.0.0 pre-release repository does not claim:

- automatic protocol or schema version negotiation;
- semantic-version range matching for protocol artifacts;
- fallback from unknown rules to the latest known rule;
- complete production support for every rule represented in conformance fixtures;
- automatic in-place migration of signed or identity-bearing objects;
- compatibility between repository release numbers and protocol/schema versions;
- that documentation-only commits redefine the fixed release candidate.

### 10. Release boundary

The fixed v1.0.0 candidate remains `f9543019f2c219aea3b085ff90f2da201b268a48`.

This documentation update does not redefine that candidate. Formal 72-hour soak, privileged reference-host qualification, version update, release pull request, tag, and GitHub Release remain separate incomplete release gates.

---

## 日本語

### 1. 適用範囲

Lingonberryでは、protocol、schema、canonicalization、identity、signature、derived-state、API、storage、journal、proof、repository releaseをそれぞれ独立したversion軸として扱います。あるversionへの対応から、別のversionへの対応を推定してはなりません。

repository release versionは配布物のversionであり、protocol objectやschema、rule versionを置き換えるものではありません。

### 2. 判定区分

互換性は次のいずれかで明示します。

- `compatible`: 意味を失わず処理できる
- `compatible-with-restrictions`: 明記された制約下でのみ処理できる
- `unsupported`: versionは既知だが実装が対応していない
- `unknown`: version識別子を認識できない
- `invalid`: version値、組み合わせ、artifactが契約違反

`unknown`や`unsupported`をfallback、前方一致、近いversionの選択、field削除によって`compatible`へ変換してはなりません。

### 3. 現在の実装範囲

現在のproduction実装は、汎用version negotiation機構ではなく限定された固定versionを実装しています。

- Knowledge Object、Transition Object、HTTP publish requestの主要schema versionは`0.1.0`
- canonical JSONは`lb.canonical.json.v1`の対象範囲を実装
- Rust production identityは主に`lb.identity.key.v1`を実装
- `lb.identity.key.v2`はconformance契約とfixtureで固定されている範囲があるが、production全体への完全実装を意味しない
- 未知のidentity ruleやderived-state ruleに対する一般的なnegotiationは存在しない

未知または未対応のversionは、現在versionへ読み替えず、unsupported／unknownとしてfail closedに扱います。

### 4. 互換性を壊す変更

canonical bytes、identity／digest入力、signature target、必須field、identifier semantics、acceptance／authority／projection分類、unknown-field policy、durable recovery semanticsを変える変更は、原則としてbreaking changeです。

optional fieldの追加も自動的に互換とは限りません。schema、identity basis、signature target、未知fieldの扱いを含めて判断します。

### 5. legacyとmigration

legacy inputは、対象version、期待されるaccept／reject／quarantine／retain、normalization可否、再出力可否、identity・signatureへの影響をfixtureで明示します。

storage migrationとprotocol/schema migrationは別の操作です。storage migrationがprotocol objectを暗黙に書き換えてはなりません。署名済みまたはidentityを持つ履歴bytesをin-placeで変更する自動migrationは、現在の契約では保証されません。

migration仕様が存在しないversion間の変換は、実装依存ではなくunsupportedです。

### 6. 変更手順

互換性に影響する変更では、影響するversion軸の決定、必要な新version識別子、fixture、compatibility matrix、production／conformance test、durable state変更時のmigration・rollback手順、release noteを更新します。

fixtureの期待値を実装に合わせて自動再生成してはなりません。期待値の変更は契約変更としてreview対象になります。

### 7. release境界

固定candidateは`f9543019f2c219aea3b085ff90f2da201b268a48`のままです。この文書変更はcandidateを再定義しません。formal 72-hour soak、privileged reference-host qualification、version update、release PR、tag、GitHub Releaseは未完了の独立gateです。
