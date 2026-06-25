# Current Work — Caglla CLI / travel-ledger-cli

> **注意:** このファイルは正式仕様ではありません。開発中の **現在地メモ** です。設計・契約の正本は `docs/specifications/` および `docs/releases/` を参照してください。

最終更新: 2026-06-25

---

## 現在フェーズ

**v3.7.1 release preparation**（patch release）

v3.7.0 は **リリース済み**（tag `v3.7.0` @ `90e902a`、GitHub Release・assets 生成済み）。tag 後に入った Okinawa Receipt Inbox sample story と `src/receipt.rs` の trashed Receipt export 修正を、**v3.7.1 patch** として切り出す準備中。

---

## 背景

```text
v3.7.0 tag (90e902a)  →  v3.7.0 GitHub Release assets（receipt.rs 修正なし）
master (feb4043)      →  Okinawa sample + trashed_at RFC3339 export fix
```

公開済み `v3.7.0` tag は付け替えない。履歴を保つため patch release で配布する。

---

## 最新完了

| 項目 | 内容 |
|---|---|
| v3.7.0 release | tag `v3.7.0`、GitHub Release、workflow 成功、assets 3 件 |
| Commit `feb4043` | **Add Okinawa Receipt Inbox sample story** — sample / tests / `src/receipt.rs` export fix |
| v3.7.1 prep（進行中） | version bump `3.7.1`、release notes、README / index / current-work 更新 |

設計系列:

```text
v3.7.0 Workflow Design     → docs/specifications/v3.7.0-receipt-assignment-and-trash-workflow-design.md
v3.7.0 Implementation      → 776bab6
v3.7.0 Release               → 90e902a (tag v3.7.0)
v3.7.1 patch                 → feb4043 + release preparation（この作業）
```

---

## 次アクション

**v3.7.1 release 確定**（差分確認後に判断）

- release preparation の変更を確認
- 問題なければ **commit** `Prepare release v3.7.1` → **tag `v3.7.1`** → **GitHub Release**

**v3.7.1 完了後**（今は着手しない）:

- v3.7.1 release verification
- v3.7.1 post-implementation review if needed
- 次期設計はまだ開始しない

---

## Release preparation checklist

- [x] **Release notes** を作成  
  `docs/releases/v3.7.1-notes.md`
- [x] **`Cargo.toml` / `Cargo.lock`** を `3.7.1` に bump
- [x] **`README.md`** の最新リリース参照を v3.7.1 に更新（patch 説明・履歴）
- [x] **`docs/releases/README.md`** に v3.7.1 行を追加
- [x] **`docs/long-term-version-strategy.md`** — v3.7.0 リリース済み・v3.7.1 patch を反映
- [x] **`make check`** を再実行して通過を確認
- [ ] **Git commit**（`Prepare release v3.7.1`）— 差分確認後
- [ ] **Git tag `v3.7.1`** + **GitHub Release** — 差分確認後

---

## まだ始めないもの（defer / out of scope）

以下は **v3.7.1 release 後** または別テーマとして扱う。今は着手しない。

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
| **次期設計（v3.8 等）** | v3.7.1 完了・検証後に判断 |

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
| リリースノート（v3.7.1） | [releases/v3.7.1-notes.md](releases/v3.7.1-notes.md) |
| リリースノート（v3.7.0） | [releases/v3.7.0-notes.md](releases/v3.7.0-notes.md) |
| Okinawa canonical sample | [../samples/okinawa_sesoko_2026/README.md](../samples/okinawa_sesoko_2026/README.md) |
| CLI コマンド一覧 | [command-reference.md](command-reference.md) |
| Export / import | [export-import.md](export-import.md) |
| 長期バージョン戦略 | [long-term-version-strategy.md](long-term-version-strategy.md) |
| 仕様索引 | [specifications/README.md](specifications/README.md) |
| リリースノート索引 | [releases/README.md](releases/README.md) |
