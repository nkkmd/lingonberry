# Secret Management

**Status: draft** | **Last updated: 2026-06-19**

## 目的

この文書は、Lingonberry における secret の置き場所と注入経路を整理する正本メモです。  
設定ファイル、protocol object、carrier metadata に secret を混ぜないための運用境界を固定します。

## 原則

- secret は protocol semantic ではない
- secret は `storage-config.json` のような一般設定ファイルに平文で置かない
- secret は repository に commit しない
- secret の保管は deployment 側の secret store に寄せる
- secret の注入は process start 時に行い、runtime 設定と分離する
- core protocol は secret を前提にしない

## 1. 保管先

secret の保管先は、deployment 環境に依存してよいものとします。

想定する保管先の例:

- OS の secret 機構
- container orchestrator の secret 機構
- systemd の drop-in や EnvironmentFile から参照する外部 secret ファイル

このリポジトリでは、単一の secret backend を強制しません。  
ただし、どの方式を使っても secret 本体は設定ファイルや公開メモに残さないことを前提にします。

## 2. 注入経路

secret は、起動時に次のいずれかで注入します。

- environment variable
- mount された secret file
- deployment tool が生成する一時的な runtime file

どの経路を使う場合でも、secret の値は config 解決の一部ではなく、実行時入力として扱います。

## 3. 現時点の扱い

現在の `relay` / `storage node` の基本運用は、secret を必須にしません。  
したがって、現時点での主な決定は次の 2 点です。

- 設定ファイルには secret を載せない
- 必要になったときだけ deployment 側で secret を注入する

authn/authz や外部サービス連携が必要になった場合は、ここで定めた注入経路を使い、protocol core には持ち込みません。

## 4. 関連

- [運用前提メモ](./OPERATIONAL_PREMISES_MEMO.md)
- [storage node runtime](./STORAGE_NODE_RUNTIME.md)
- [Access and Retention Policy](./ACCESS_RETENTION_POLICY.md)
- [Node Lifecycle Runbook](./NODE_LIFECYCLE_RUNBOOK.md)
