# 0.1.0 公開前チェックリスト

**Status: draft** | **Last updated: 2026-06-22**

## 目的

この文書は、Lingonberry を `0.1.0` として OSS 公開・配布する前に、最低限確認する項目をまとめるための実務用チェックリストです。

対象は source release を前提にした公開リポジトリです。  
README、仕様文書、schema、fixtures、実装のあいだで、公開してよい範囲が揃っているかを確認します。

このチェックリストは、「公開してよいか」を判断するための最終確認です。  
未完了項目がある場合は、その理由を release note または README に明示します。

## 公開範囲の確認

- [x] `0.1.0` で公開する範囲を 1 文で説明できる
- [x] `core protocol` と `application profile` の境界を説明できる
- [x] `relay` と `storage node` の責務分離を説明できる
- [x] `wire` と `canonical` が別プロトコルではなく別表現であると説明できる
- [x] `Toitoi` は application profile であり、core ではないと明示できる
- [x] Phase 1 の JavaScript 実装が検証用ブートストラップであることを説明できる
- [x] Phase 2 以降の Rust + SQLite 本命実装との関係を説明できる
- [x] 未完了の roadmap / backlog を draft / active として扱う方針がある

### 確認メモ

- `0.1.0` では、core protocol の概念・用語・wire 仕様・JSON Schema・fixture・運用メモ・ロードマップを source release として公開し、Toitoi を含む profile 差分は core から切り分けて扱う
- `core protocol` は分野非依存の共通層で、`application profile` はその上に載せる追加ルールとして定義している
- `relay` は ingress / validation / routing、`storage node` は persistence / replay / export を担う
- `wire` と `canonical` は同じ protocol object の別表現として説明している
- `Toitoi` は core ではなく application profile として明示している
- Phase 1 の JavaScript 実装は検証用ブートストラップで、Phase 2 以降の Rust + SQLite 本命実装へ移行する前段として位置づけている
- 未完了の roadmap / backlog は `draft` / `active` として扱う方針が、各文書の `Status` と本文で揃っている

## 法務とライセンス

- [x] `LICENSE` を追加している
- [x] README に license の種別を明記している
- [x] 主要な外部由来素材があれば、再配布条件を確認している
- [x] 追加した文書やコードに、公開できない第三者情報が混ざっていない
- [x] 依存物や引用物の出典を、必要に応じて明示している

### 確認メモ

- 現時点で公開対象の `docs/`、`schemas/`、`fixtures/`、`packages/` を確認した範囲では、第三者由来素材や秘密情報の同梱は見当たらない
- 外部由来素材や引用物を今後追加する場合は、その都度、再配布条件と出典表示を個別に確認する
- `Apache-2.0` を採用し、現時点では `NOTICE` を追加しない方針を維持する

### 現時点の判断

- `Apache-2.0` を採用する
- `NOTICE` は現時点では追加しない
- 今後、第三者由来の notice を同梱する場合だけ `NOTICE` を再検討する

## リポジトリ衛生

- [x] 秘密情報、API key、認証情報、個人情報を含めていない
- [x] ローカル実行時の生成物が `.gitignore` で適切に除外されている
- [x] 公開不要な一時ファイルやキャッシュが残っていない
- [x] `git status --short` が空、または差分が意図通りで説明できる
- [x] ファイル名、配置、リンク切れが公開前に整理されている

### 確認メモ

- 秘密情報の典型パターンや OS / editor / build 由来の一時ファイルは、公開対象ツリー内で見当たらなかった
- `.gitignore` には `.lingonberry/` と `target/` が含まれており、ローカル生成物を除外する意図がある
- 現在の `git status --short` の差分は、このチェックリストへの追記のみで、意図した変更である
- 主要な相対リンクは、チェックリスト・README・operations・fixtures をまたいで実在を確認した

## 仕様整合

- [x] [GLOSSARY](../concepts/GLOSSARY.md) と実装・README の用語が一致している
- [x] [CONCEPT_MODEL](../concepts/CONCEPT_MODEL.md) と README の説明が一致している
- [x] [CARRIER](../concepts/CARRIER.md) と carrier 実装の説明が一致している
- [x] [Toitoi Application Profile](../profiles/TOITOI_APPLICATION_PROFILE.md) と core の境界が一致している
- [x] [PROTOCOL_NATIVE_WIRE_FORMAT](../protocols/PROTOCOL_NATIVE_WIRE_FORMAT.md) と schema が矛盾していない
- [x] [schemas/README](../../schemas/README.md) と各 schema ファイルの説明が一致している
- [x] README、operations README、roadmap README の案内先が一致している
- [x] `Status` と `Last updated` が、更新した文書でそろっている
- [x] `（完了済み）` の表記が必要な完了済み見出しに付いている

### 確認メモ

- `GLOSSARY`、`CONCEPT_MODEL`、`CARRIER`、`Toitoi Application Profile`、`PROTOCOL_NATIVE_WIRE_FORMAT` は、README や関連文書の説明と整合している
- `README.md`、`docs/operations/README.md`、`docs/roadmap/README.md` の案内先は、互いに参照関係が揃っている
- `Status` と `Last updated` は、今回見た更新済み文書で揃っている
- `（完了済み）` の表記は、完了済みフェーズや issue の見出しに付いている
- `schemas/README` と各 schema ファイルの説明は、README の記述と schema の description が揃っている

## 実装確認

- [x] `cargo run -p lingonberry-relay -- capabilities` を実行できる
- [x] `cargo run -p lingonberry-relay -- serve-http 127.0.0.1:8787` を実行できる
- [x] `cargo run -p lingonberry-relay -- publish fixtures/http-publish-request/minimal-request.json` を実行できる
- [x] `cargo run -p lingonberry-relay -- export-archive /tmp/lingonberry-archive` を実行できる
- [x] `cargo run -p lingonberry-relay -- import-archive /tmp/lingonberry-archive` を実行できる
- [x] `cargo run -p lingonberry-storage -- capabilities` を実行できる
- [x] `cargo run -p lingonberry-storage -- config` を実行できる
- [x] `cargo run -p lingonberry-storage -- ready` を実行できる
- [x] `cargo run -p lingonberry-storage -- run` を実行できる
- [x] `fixtures/` の最小入力が validate できる
- [x] 不正な fixture が reject される
- [x] relay と storage node の責務が README の記述どおりに分かれている
- [x] 公開向けの README 例が実際の CLI と一致している

### 確認メモ

- `lingonberry-relay capabilities` は実行できた
- `lingonberry-relay serve-http`、`publish`、`export-archive`、`import-archive` は実行できた
- `lingonberry-storage capabilities`、`config`、`ready`、`run` は実行できた
- `fixtures/` の最小入力 validate と不正 fixture の reject も確認できた
- relay / storage の責務分離は README、operations 文書、runtime 入口の見え方が一致している
- README の CLI 例は、`cargo run -p lingonberry-relay -- ...` と `cargo run -p lingonberry-storage -- ...` に揃えた

## 配布物

- [x] 公開対象のディレクトリ構成を 1 つの README から辿れる
- [x] `README.md` から最初の確認先が分かる
- [x] `docs/architecture/`、`docs/concepts/`、`docs/protocols/`、`docs/operations/`、`docs/roadmap/`、`docs/profiles/`、`schemas/`、`fixtures/` の役割が説明できる
- [x] release note で `0.1.0` の範囲を説明できる
- [x] 必要な場合は source archive の作成手順を説明できる
- [x] `0.1.0` のタグ名と公開名が一致している

### 確認メモ

- `README.md` の「まず読む場所」から、`architecture`、`roadmap`、`operations`、`concepts`、`protocols`、`schemas` へ辿れる
- `docs/architecture/README.md`、`docs/concepts/README.md`、`docs/protocols/README.md`、`docs/operations/README.md`、`docs/roadmap/README.md`、`docs/profiles/README.md`、`schemas/README.md`、`fixtures/README.md` が、それぞれの役割を説明している
- release note は [0.1.0 Release Note Draft](./RELEASE_0_1_0_RELEASE_NOTE.md) としてまとめた
- source archive の作成手順は release note に明記した
- タグ名と公開名の一致は、まだ実物のタグ作成前なので公開直前に確認する

## 運用・安全

- [x] public relay の trust model が文書化されている
- [x] public / private の扱いが core から外れている
- [x] access / retention の扱いが文書化されている
- [x] storage path、state dir、backup の考え方が文書化されている
- [x] readiness の確認方法が `ready` コマンドまたは endpoint で説明できる
- [x] 0.1.0 でサポートしないものを明示できる
- [x] 失敗時に参照すべき文書が 1 本化されている

### 確認メモ

- public relay の trust model は [Access and Retention Policy](../operations/ACCESS_RETENTION_POLICY.md) と [HTTP Carrier Contract](../operations/HTTP_CARRIER_CONTRACT.md) に明記されている
- public / private の扱いは core の初期版に private / encrypted object を含めない方針として整理されている
- access / retention は policy と carrier capability の責務として整理されている
- storage path、state dir、backup は [storage node runtime](../operations/STORAGE_NODE_RUNTIME.md) と [Node Lifecycle Runbook](../operations/NODE_LIFECYCLE_RUNBOOK.md) にまとまっている
- readiness の確認は `ready` コマンドと HTTP の readiness endpoint の両方で説明されている
- `0.1.0` のサポート外項目は release note にも明示した
- 失敗時の一次参照先は [Node Lifecycle Runbook](../operations/NODE_LIFECYCLE_RUNBOOK.md) に一本化した

## 公開直前の最終確認

- [x] `git status --short` が空、または公開してよい差分だけになっている
- [x] ルート README が初見の読者にとって入口として機能する
- [x] 0.1.0 の release note に、公開直前の確認項目と今後のロードマップを分けて書いている
- [x] 公開後に直したい事項が backlog に落ちている

## 参照

- [README](../../README.md)
- [概念](../concepts/README.md)
- [Protocols](../protocols/README.md)
- [Schemas](../../schemas/README.md)
- [Operations](../operations/README.md)
- [ロードマップ](./README.md)
- [Toitoi Application Profile](../profiles/TOITOI_APPLICATION_PROFILE.md)
