# v0.9.0 Rust API Inventory

**Status: freeze candidate** | **Release target: v0.9.0** | **Last updated: 2026-07-22**

この文書は、workspace内Rust crateのexported surfaceをv1.0互換性監査のために分類します。Rustの`pub`指定だけで外部互換性を決定せず、documented consumer contract、wire／storage semantics、operator behaviorを優先します。

## Classification

- **FROZEN-CANDIDATE**: v1.0まで名称、型、意味、error semanticsを安定させる候補。
- **BEHAVIOR-FROZEN**: Rust item自体ではなく、その背後の外部挙動を安定させる。
- **WORKSPACE-INTERNAL**: crate間利用のため公開されているが、v1.x external APIとは扱わない。
- **IMPLEMENTATION-DETAIL**: refactor可能。ただし安全境界と永続データの意味を変えてはならない。

## `lingonberry-protocol`

### FROZEN-CANDIDATE

- protocol、schema、archive、capability version constants
- carrier kind constants
- `JsonValue`
- `JsonError`のbounded message／byte position contract
- `FinalizedKnowledgeObject`
- `parse_json`
- `normalize_json`
- `to_canonical_json`
- `validate_knowledge_object`
- `validate_publish_request`
- `finalize_knowledge_object`
- `derive_identity_key`
- `is_lb_object_id`
- `supported_knowledge_types`
- capability manifest construction behavior

### BEHAVIOR-FROZEN

- canonical object-key ordering
- canonical signature payload
- duplicate object-key handling
- identity-key basis
- malformed JSON rejection
- signature mismatch rejection

### IMPLEMENTATION-DETAIL

- recursive-descent parser implementation
- OpenSSL subprocess workspace layout
- helper function decomposition
- temporary filename generation

Resource bounds、exclusive temporary workspace creation、cleanupはhardeningとして変更可能ですが、valid inputのcanonical resultとsignature outcomeを変更してはなりません。

## `lingonberry-identity`

### FROZEN-CANDIDATE

- identity claim validation behavior
- canonical identity key derivationとの整合
- issuer／verification evidence field semantics

### WORKSPACE-INTERNAL

- fixture construction helper
- test-only signing helper
- OpenSSL command assembly helper

## `lingonberry-validation`

### FROZEN-CANDIDATE

- validation report semantics
- acceptance decision categories
- reject／defer／acceptの区別
- acceptance policyの明示設定
- full validation／finalization behavior

### BEHAVIOR-FROZEN

- validation未通過objectをcanonical storageへ進めない
- incomplete evidenceをacceptしない
- unknown fieldとschema mismatchを黙って正規化しない

### WORKSPACE-INTERNAL

- individual rule module layout
- report assembly order。ただしpublished error orderingに依存するCLI fixtureがある場合を除く

## `lingonberry-core`

### FROZEN-CANDIDATE

- `StorageBackend` behavior
- append outcomeのduplicate semantics
- store error code／message boundary
- canonical get／raw request get／list／subscribe／replay semantics
- quarantine outcome categories
- archive import／export report semantics

### BEHAVIOR-FROZEN

- duplicateとconflictを同一分類にしない
- conflictで既存objectを上書きしない
- canonical storageをsemantic sourceとする
- quarantine promotionをvalidation bypassに使用しない
- archive importでimmutable evidenceをrewriteしない

### WORKSPACE-INTERNAL

- concrete file／SQLite backend implementation details
- runtime path helper
- quarantine module decomposition
- transaction helper structsでoperatorまたはwire contractに露出しないもの

## `lingonberry-indexer`

### FROZEN-CANDIDATE

- checkpoint／catch-up result semantics
- deterministic verify／rebuild behavior
- canonical storageからのreconstruction contract

### BEHAVIOR-FROZEN

- indexをcanonical sourceとして扱わない
- incomplete catch-upでlast-known-good checkpointを上書きしない
- restart後のdeterministic recovery

### WORKSPACE-INTERNAL

- batch sizing
- internal cursor representation
- derived table implementation

## `lingonberry-storage`

### FROZEN-CANDIDATE

- storage format manifest semantics
- migration plan／apply／resume／rollback result
- backup binding and verification evidence
- unknown-newer format rejection

### BEHAVIOR-FROZEN

- normal startupでimplicit migrationしない
- verified backupなしにnon-empty migrationしない
- durable verification前にtarget formatをpublishしない
- active／non-empty／symlink restore targetを拒否する

### WORKSPACE-INTERNAL

- migration workspace layoutで文書化されていない中間file
- helper type decomposition
- checksum calculation implementation。ただしpublished digest algorithmは除く

## `lingonberry-relay`

### FROZEN-CANDIDATE

- publish／retrieve／query contract
- admin authentication decision semantics
- transition API behavior
- health／ready／status／doctor／verify／metrics output contract
- stable diagnostic codeとexit code

### BEHAVIOR-FROZEN

- authorizationをmutation後に評価しない
- read-only diagnosticが状態を修復しない
- untrusted request materialをunbounded errorへ展開しない
- effective viewがoriginal objectをrewriteしない

### WORKSPACE-INTERNAL

- binary main file decomposition
- route handler function names
- runtime wiring
- metrics collection internals。ただしmetric名／label cardinalityがpublished contractの場合を除く

## Freeze review rules

1. FROZEN-CANDIDATE itemの削除、rename、型変更、error category変更には明示的なcompatibility reviewを必要とする。
2. BEHAVIOR-FROZEN領域の変更にはfixture、regression test、release noteを必要とする。
3. WORKSPACE-INTERNAL itemはrefactor可能だが、public documentationで直接使用を推奨しない。
4. security hardeningによるrejection追加は、従来validだった入力を拒否する場合にcompatibility evidenceを必要とする。
5. storage、wire、signature、identity、diagnostic codeの意味変更を単なるinternal refactorとして扱わない。

## Remaining audit work

- [ ] rustdoc上のexported item一覧を機械生成する
- [ ] README／operator docsから直接参照されるRust APIを照合する
- [ ] external fixtureが依存するerror orderingを調査する
- [ ] deprecated candidateを識別する
- [ ] v1.0 freeze documentへ承認結果を反映する
