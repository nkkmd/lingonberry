# Phase 2 Duplicate / Conflict Plan

## 目的

canonical ID、carrier identity、canonical contentの3軸でduplicate／conflictを決定論的に分類し、すべてのstorage entry pathで同じ安全境界を適用する。

## 完了済み

- contract version `1`
- pure duplicate／conflict classifier
- live CLI／HTTP ingestionへのclassified append適用
- File／SQLite backend parity tests
- retry／archive import parity tests
- quarantine promotion classified API
- quarantine promotion File／SQLite parity tests
- active `quarantine-promote` CLI entrypointのclassified API接続
- duplicate／conflict時のraw log非増加
- conflict後のcanonical object／identity binding不変性

## 現在の作業

- batch quarantine promotionをclassified promotionへ統合
- replay-derived restoreをclassified appendへ接続
- archive importがclassified appendを通ることを明示

## 後続作業

- File backend内部の防御的な手書き判定を共通classifierへ置換
- SQLite backend内部の防御的な手書き判定を共通classifierへ置換
- release roadmapを同期

## 安全境界

- exact duplicateはidempotent success
- conflictではcanonical storageとraw wire logを変更しない
- canonical IDとcarrier identityを別の対応へ再束縛しない
- conflictしたquarantine recordをpromotedとして解決しない
- I/O errorやcorruptionをduplicate／conflictへ縮退させない
- File／SQLiteで同じ外部結果へ収束する
