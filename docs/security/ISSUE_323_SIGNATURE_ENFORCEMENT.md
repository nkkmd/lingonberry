# Issue #323: HTTP Publish Signature Enforcement

[English](#english) | [日本語](#japanese)

> English is the normative version of this document. The Japanese section is a synchronized translation.

<a id="english"></a>

## English

**Status: implementation under review**  
**Rule:** `lb.http.publish.signature.v1`  
**Tracking:** Issue #323 / PR #331

The active Rust publish-ingestion path now verifies the Ed25519 publisher signature immediately after JSON parsing and before acceptance-policy evaluation, quarantine, duplicate or conflict classification, raw-request append, or canonical storage.

### Stable result codes

- `LB_PUBLISH_SIGNATURE_MALFORMED`: missing, structurally invalid, wrong-length, uppercase, or otherwise non-lowercase-hex key/signature encoding.
- `LB_PUBLISH_SIGNATURE_INVALID`: the correctly encoded signature does not verify the canonical request bytes.
- `LB_PUBLISH_SIGNATURE_VERIFIER_ERROR`: the verifier workspace, artifact creation, or OpenSSL execution fails. This is a fail-closed operational failure.

### Enforced order

```text
parse JSON
→ validate signature envelope and encoding
→ reconstruct canonical signature target
→ verify Ed25519 signature
→ validate object and acceptance policy
→ finalize
→ duplicate/conflict classification
→ append raw request and canonical object
```

Duplicate requests do not bypass signature verification. Invalid signatures cannot be deferred to quarantine as otherwise acceptable objects.

### Test coverage

The Rust ingestion module executes the checked-in valid HTTP publish signature vector and a tampered-signature negative case. The JavaScript external conformance runner already executes the registered valid, invalid, and malformed signature vectors.

### Release consequence

This is a runtime-affecting security correction. The previous fixed v1.0.0 qualification candidate must not be reused. After this change is reviewed and merged, Lingonberry requires a new candidate designation and rerun of all affected candidate-bound qualification, security, documentation, and release evidence.

---

<a id="japanese"></a>

## 日本語

**状態: 実装レビュー中**  
**ルール:** `lb.http.publish.signature.v1`  
**追跡:** Issue #323 / PR #331

現在のRust publish取り込み経路は、JSON解析直後にEd25519 publisher署名を検証します。この検証は、acceptance policy判定、quarantine、duplicate／conflict分類、raw request追記、canonical storage保存より前に実施されます。

### 安定した結果コード

- `LB_PUBLISH_SIGNATURE_MALFORMED`: 公開鍵または署名の欠落、構造不正、長さ不正、大文字16進数、その他の小文字16進数形式違反。
- `LB_PUBLISH_SIGNATURE_INVALID`: 形式が正しい署名がcanonical request bytesに対して検証に失敗した場合。
- `LB_PUBLISH_SIGNATURE_VERIFIER_ERROR`: verifier workspace、artifact作成、OpenSSL実行に失敗した場合。処理はfail closedになります。

### 強制される処理順序

```text
JSON解析
→ 署名envelopeとencodingの検証
→ canonical署名対象の再構築
→ Ed25519署名検証
→ objectとacceptance policyの検証
→ finalize
→ duplicate／conflict分類
→ raw requestとcanonical objectの追記
```

duplicate requestでも署名検証は省略されません。不正署名を、その他の点では受理可能なobjectとしてquarantineへ送ることもありません。

### テスト範囲

Rust ingestion moduleは、リポジトリに登録された正常なHTTP publish署名vectorと、署名を改変したnegative caseを実行します。JavaScript external conformance runnerは、登録済みのvalid、invalid、malformed署名vectorを実行します。

### リリースへの影響

これはruntime動作に影響するsecurity修正です。以前の固定v1.0.0資格候補は再利用できません。この変更をレビュー・マージした後、新しいcandidateを指定し、影響を受けるcandidate-bound qualification、security review、documentation、release evidenceを再実行する必要があります。
