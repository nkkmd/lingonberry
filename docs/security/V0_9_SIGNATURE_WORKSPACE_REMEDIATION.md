# v0.9.0 Signature Verification Workspace Remediation

**Status: implementation-ready** | **Finding: LB-SEC-009-001** | **Last updated: 2026-07-22**

この文書は、`packages/protocol/src/lib.rs` の `verify_publish_request_signature_with_openssl` が使用するtemporary workspaceを安全に管理するための実装契約です。

## Goal

署名検証の外部挙動とcanonical payloadを変更せず、temporary artifactの残留、path collision、symlink追従、過剰permissionを除去します。

## Non-goals

- signature algorithmの変更
- canonical publish request payloadの変更
- OpenSSL command contractの変更
- public error messageへのpayload、public key、signatureの追加
- protocol schemaまたはwire formatの変更

## Required implementation shape

### Workspace creation

workspace rootは引き続きOS temporary directoryを使用します。ただし、単一のtimestamp値をそのままdirectory名として信用しません。

生成名には少なくとも次を含めます。

- process id
- monotonic process-local counter
- current time由来の補助値

候補pathは`create_dir`でexclusiveに作成します。`create_dir_all`は使用しません。既存path、symlink、file、directory collisionはいずれも再利用せず、有限回の再生成後にfail closedとします。

### Permissions

Unix reference platformでは、workspace作成直後にpermissionを`0o700`へ設定します。permission設定に失敗したworkspaceは使用せず、best-effort cleanup後に検証失敗として返します。

workspace内のartifactは新規作成のみを許可します。

- `public-key.der`
- `signature.bin`
- `message.bin`

既存artifactをtruncateまたはoverwriteしません。`OpenOptions::create_new(true)`を使用します。

### Cleanup ownership

workspace作成成功後は、単一のscope guardがworkspace ownershipを持ちます。関数のreturn pathに依存した手書きcleanupを複数箇所へ分散させません。

scope guardの`Drop`で`remove_dir_all`をbest-effort実行します。これにより次の経路を同一契約で扱います。

- signature success
- signature mismatch
- public-key write failure
- signature write failure
- message write failure
- OpenSSL spawn failure
- OpenSSL non-zero exit
- UTF-8 path conversion failure

process abort、SIGKILL、host crashはscope cleanupの保証対象外です。残留workspaceはoperator diagnosticsとperiodic cleanup policyで扱います。

### Error precedence

署名検証のprimary resultをcleanup failureで置き換えません。

- verification failureはverification failureとして返す
- setup failureはsetup failureとして返す
- cleanup failureはpayloadやsignatureを含まないbounded diagnosticとして記録する

現行crateに安全なdiagnostic sinkが存在しない場合、v0.9.0ではcleanupをbest-effortとし、error surfaceを拡張せず、残留workspace検出をsecurity regression testとoperator diagnostic backlogへ接続します。

## Concurrency contract

同一processおよび複数processの並行検証がworkspaceを共有してはなりません。exclusive directory creationと`create_new` artifact creationの両方で保証します。

workspace identifierはsecurity identityではありません。予測困難性だけに依存せず、filesystemのexclusive creationを正本とします。

## Regression tests

最低限、次を自動検証します。

1. 有効な署名の検証後にworkspaceが残らない。
2. 無効な署名の検証後にworkspaceが残らない。
3. OpenSSL executableが利用できない経路でもworkspaceが残らない。
4. pre-existing candidate pathを再利用しない。
5. candidate pathがsymlinkの場合に追従しない。
6. artifact collision時に既存fileを変更しない。
7. 並行検証が異なるworkspaceを使用する。
8. error messageへcanonical payload、signature、private materialを含めない。

## Review checklist

- [ ] `create_dir_all`をtemporary verification workspace生成に使用していない
- [ ] workspace permissionがUnixで`0o700`
- [ ] artifactが`create_new(true)`
- [ ] cleanup ownershipが単一scope guardに集約されている
- [ ] success／failure／intermediate errorの全経路をtestしている
- [ ] canonical payloadとsignature verification resultが従来と一致する
- [ ] standard CIが成功する
- [ ] LB-SEC-009-001のfinding stateとevidenceを更新する
