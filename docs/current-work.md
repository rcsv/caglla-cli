# Current Work — Caglla CLI / travel-ledger-cli

> **注意:** このファイルは正式仕様ではありません。開発中の **現在地メモ** です。設計・契約の正本は `docs/specifications/` および `docs/releases/` を参照してください。

最終更新: 2026-06-25

---

## 現在フェーズ

**v3.7.0 release preparation**（実装完了済み）

Receipt assignment and trash workflow の **実装は完了**。リリースノート・バージョン bump・索引更新を進めている段階です。

---

## 最新完了

| 項目 | 内容 |
|---|---|
| Commit | `776bab6` — **Implement Receipt assignment and trash workflow** |
| 実装概要 | `receipt assign` / `trash` / `restore`、`pending sum`、`ignored → trashed` migration、export schema **v8**、v6/v7 import 互換、`validate-export` / `diff` 更新、tests / docs |
| 現在地メモ | `1fd1e85` — Add current work tracker |

設計系列:

```text
Workflow Design        → docs/specifications/v3.7.0-receipt-assignment-and-trash-workflow-design.md
Implementation Plan    → docs/specifications/v3.7.0-receipt-assignment-and-trash-implementation-plan.md
Implementation         → 776bab6
Release preparation    → 進行中（この作業）
```

---

## 次アクション

**v3.7.0 release 確定**（結果確認後に判断）

- release preparation の変更を確認
- 問題なければ **commit** → **tag `v3.7.0`** → **GitHub Release**

---

## Release preparation checklist

- [ ] **Post-Implementation Review** を作成  
  `docs/specifications/v3.7.0-receipt-assignment-and-trash-post-implementation-review.md`
- [x] **Release notes** を作成  
  `docs/releases/v3.7.0-notes.md`
- [x] **`Cargo.toml` / `Cargo.lock`** を `3.7.0` に bump
- [x] **`README.md`** の最新リリース参照を v3.7.0 に更新
- [x] **`docs/releases/README.md`** を更新
- [x] **`docs/long-term-version-strategy.md`** を更新（v3.7 実装完了・release preparation 中）
- [ ] **`docs/specifications/README.md`** の v3.7.0 ステータスを「リリース済み」へ更新（tag 後）
- [ ] **`make check`** を再実行して通過を確認 — **通過済み**（release preparation 作業後）
- [ ] **Git commit**（release 作業用）— 結果確認後
- [ ] **Git tag `v3.7.0`** + **GitHub Release** — 結果確認後

---

## まだ始めないもの（defer / out of scope）

以下は **v3.7.0 release 後** または別テーマとして扱う。今は着手しない。

| テーマ | 理由 |
|---|---|
| **Evidence / Attachment** | 共通証憑レイヤーは別設計・別リリース |
| **`image_path`** | Receipt 専用画像パスは採用しない方針 |
| **OCR** | 自動解析は scope 外 |
| **Balance / Settlement** | 精算・分担は long-term defer |
| **Expense reassign / unassign / trash** | Receipt assign 後の Expense 補正ルートは defer |
| **`receipt purge`** | Trash からの物理削除は defer |
| **standalone `receipt summary`** | pending sum は `receipt list` に統合済み |
| **Potential Actual display** | 旅行中の補助表示は defer |
| **Settlement warning** | Balance / Settlement 系とセットで将来検討 |
| **Day-level Planned vs Actual** | 別バージョン候補 |
| **Participant sharing** | Shared expense 以外の拡張は defer |

---

## 重要方針（実装済み・維持）

- Receipt は **Actual ではない**
- Pending Receipt sum は **Actual ではない**
- `receipt assign` 後に作成された **Expense だけ** が Actual に入る
- `trip export-md` / `trip stats` / `trip stats --json` に pending Receipt を **混ぜない**
- `receipt assign` は **transaction 必須**、完了後 **Receipt を削除**
- `linked` / `converted` / `receipt link` / `linked_expense_id` は **復活させない**

---

## クイック参照

| 用途 | パス |
|---|---|
| リリースノート（v3.7.0） | [releases/v3.7.0-notes.md](releases/v3.7.0-notes.md) |
| CLI コマンド一覧 | [command-reference.md](command-reference.md) |
| Export / import | [export-import.md](export-import.md) |
| 長期バージョン戦略 | [long-term-version-strategy.md](long-term-version-strategy.md) |
| 仕様索引 | [specifications/README.md](specifications/README.md) |
| リリースノート索引 | [releases/README.md](releases/README.md) |
