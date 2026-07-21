# Supported Platforms

**Status: active** | **Reference platform since: v0.8.0** | **Last updated: 2026-07-22**

## Reference platform

Lingonberryの正式なLinux基準環境は次のとおりです。

- Ubuntu Server 24.04 LTS
- x86_64（amd64）
- systemd
- ext4または同等の通常のローカルLinuxファイルシステム
- Rust stable toolchainによるsource build

v0.8.0以降のリリースでは、この環境を基準として以下を検証します。

- workspace build、format、Clippy、test
- storage configuration、doctor、health、readiness、metrics
- systemd unitの構文と起動契約
- publish、retrieve、replay
- backup create／verify
- isolated restore plan／apply
- index verify／rebuild
- isolated disaster-recovery drill
- migrationおよびupgrade runbook

## Support policy

Ubuntu Server 24.04 LTSは、手順と受け入れ試験を正式に維持するreference platformです。LingonberryのRust実装やデータ契約をUbuntu固有にはせず、他のsystemdベースLinuxでも動作できる構造を維持します。

次の環境はbest-effort supportです。

- Debian 12以降
- Ubuntuの新しいLTSリリース
- Fedora、Rocky Linux、AlmaLinuxなどのsystemdベースLinux
- arm64 Linux

best-effort環境では、ビルドや基本動作が可能でも、各リリースでsystemd、権限、バックアップ、復元、DR drillまでを正式検証するとは限りません。

## Out of scope for the reference contract

以下はv0.8.0の正式な基準環境には含めません。

- systemdを使用しないLinux
- WindowsまたはmacOSでのproduction node運用
- network filesystemをactive canonical storageとして使用する構成
- container-only deploymentを唯一の正式運用方式とする構成
- 32-bit architecture

これらを禁止するものではありませんが、reference platformのリリース判定からは分離します。

## Platform changes

reference platformの変更は通常の依存関係更新として暗黙に行いません。Ubuntu LTS、CPU architecture、init systemの変更は、roadmap、runbook、CI、release checklistを同時に更新する明示的な運用判断として扱います。
