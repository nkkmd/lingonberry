# 0.1.0 Release Note Draft

**Status: draft** | **Last updated: 2026-06-23**

## 概要

Lingonberry `0.1.0` は、core protocol、application profile の境界、HTTP carrier、file / archive carrier、schemas、fixtures、最小 runtime 実装、運用メモを source release として公開する最初の版です。

この版では、知識オブジェクトの wire / canonical 表現、relay / storage node の役割分離、validate / normalize / finalize の境界を固定します。

## 0.1.0 の範囲

- core protocol の概念と用語
- `knowledge-object.schema.json` と `http-publish-request.schema.json`
- fixtures による validate / publish の最小確認
- `cargo run -p lingonberry-relay -- ...` と `cargo run -p lingonberry-storage -- ...` による最小 runtime 入口
- `packages/protocol`、`packages/core`、`packages/indexer`、`packages/relay`、`packages/storage` の Rust 実装
- HTTP carrier と file / archive carrier の正本文書
- relay / storage node の運用メモと runbook

## 公開直前の確認項目

公開直前に確認する項目は、次の 2 点です。

- `0.1.0` tag と release title が一致していること
- GitHub Release または `git archive` で source archive を取得できること

## 今後のロードマップ

- `private / encrypted object` は core の初期版に含めず、必要なら application profile 側で扱う
- federated carrier は将来の拡張として扱い、0.1.0 では正規 carrier の基礎に集中する
- relay / storage / archive の運用確認を引き続き固める
- carrier 拡張時の fail closed と failure routing を runbook に寄せる

## post-release follow-up

0.1.0 の公開後に直したい事項は、次の backlog に分けて追跡します。

- [実装バックログ](./IMPLEMENTATION_BACKLOG.md)
- [運用準備バックログ](./OPERATIONAL_READINESS_BACKLOG.md)

## source archive

source archive を作る場合は、Git tag を起点にして repository snapshot を書き出します。
タグ名を `0.1.0` にする場合の例は次のとおりです。

```bash
git archive --format=tar.gz --prefix=lingonberry-0.1.0/ 0.1.0 > lingonberry-0.1.0.tar.gz
```

必要に応じて、README と release note の内容が同じ tag を指しているかを確認します。

## 参照

- [0.1.0 公開前チェックリスト](./RELEASE_0_1_0_CHECKLIST.md)
- [ロードマップ](./README.md)
- [Node Lifecycle Runbook](../operations/NODE_LIFECYCLE_RUNBOOK.md)
