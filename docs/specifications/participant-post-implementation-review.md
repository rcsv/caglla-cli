# Participant Post-Implementation Review

Caglla CLI v2.0.0 **Participant Foundation** の実装（Issue #10 / PR #24）が、設計系列 #7 / #8 / #9 および補正 PR #21 / #22 の意図と整合しているかを検証する Post-Implementation Review です。

**v2.0.0 時点: 仕様整理・リリース判定が主目的。** 本書は大きな実装変更を伴わない。改善候補は §Known Gaps / §Non-blocking Follow-ups に記録する。

| ドキュメント | 役割 |
|---|---|
| [participant-model.md](participant-model.md) (#7) | Responsibilities Review — **上書きしない** |
| [participant-entity-design.md](participant-entity-design.md) (#8) | Entity Design — **上書きしない** |
| [participant-implementation-plan.md](participant-implementation-plan.md) (#9) | Implementation Plan — **上書きしない** |
| PR #21 | Person / Trip participation 境界の設計補正 |
| PR #22 | count semantics / `is_self` の設計補正 |
| PR #24 (`e638652`) | Implementation — CRUD + export v4 |
| **本書** (#11) | **実装後**の整合性レビュー・Release 判定 |

設計系列:

```text
#7  Responsibilities Review   → participant-model.md
#8  Entity Design             → participant-entity-design.md
#9  Implementation Plan        → participant-implementation-plan.md
#10 Implementation             → PR #24 (merge e638652)
#11 Post-Implementation Review → this document
#12 Release v2.0.0             → 次フェーズ
```

---

## Purpose

1. Issue #10 の実装が [participant-implementation-plan.md](participant-implementation-plan.md) の範囲内に収まっていることを確認する。
2. 設計前提（Trip participation record、`is_self`、count semantics）がコード・CLI・export・stats で一貫していることを確認する。
3. テスト・ドキュメント・canonical sample の不足を整理し、**Release blocker の有無**を判定する。
4. Issue #12 Release v2.0.0 に進める状態かを明記する。

---

## Source Documents

| 種別 | 参照 |
|---|---|
| ワークフロー | [github-workflow.md](../github-workflow.md) |
| 設計系列 | 上表の participant-model / entity-design / implementation-plan |
| データモデル | [data-model.md](../data-model.md) |
| Export / Import | [export-import.md](../export-import.md), [export-schema.md](export-schema.md) |
| CLI | [command-reference.md](../command-reference.md) |
| Markdown | [markdown-export.md](../markdown-export.md) |
| 開発 | [development.md](../development.md) |
| ロードマップ | [long-term-version-strategy.md](../long-term-version-strategy.md) |
| Release draft | [v2.0.0-notes.md](../releases/v2.0.0-notes.md) |
| 実装 | `src/participant.rs`, `src/trip.rs`, `src/diff.rs`, `src/markdown.rs`, `src/stats.rs`, `src/doctor.rs`, `src/advisor.rs` |

---

## Implementation Summary

PR #24（merge `e638652`）で以下を `master` に反映した。

| 領域 | 内容 |
|---|---|
| **DB** | `participants` テーブル、`is_self INTEGER NOT NULL DEFAULT 0`、partial unique index |
| **Domain** | `src/participant.rs` — CRUD、count 集計、export/import 検証 |
| **CLI** | `participant add/list/show/update/delete`（`--trip`, `--name`, `--self`, `--not-self`） |
| **Trip 連携** | delete cascade、`duplicate_trip` は export/import 経由で participants 複製 |
| **Export** | `schema_version: 4`、top-level `participants[]` |
| **Import** | v3/v4 ルーティング、v3 は `participants` 省略 = 空 |
| **diff** | added / removed / `is_self` changed（identity: `sort_order` + `name`） |
| **export-md** | Overview 直後に `## Participants` 表 |
| **stats / doctor / advisor** | count semantics、info 警告 2 種 |
| **Tests** | unit（`participant.rs`）、integration（`participant_cli.rs`）、golden 更新 |
| **Docs** | command-reference、export-schema、markdown-export、v2.0.0-notes draft |

**意図的に未実装:** Person / Traveler Profile、`person_id`、Expense FK、Settlement、Reservation guest linking、Cargo.toml version bump。

---

## Scope Review

### Issue #10 実装範囲（Implementation Plan 対照）

| 項目 | 計画 | 実装 | 判定 |
|---|---|---|---|
| `participants` table | §DB Plan | `migrate_participants` | ✅ |
| `is_self` | §DDL | 列 + export 出力 | ✅ |
| partial unique index | §DDL | `idx_participants_one_self_per_trip` | ✅ |
| app-side validation | §Validation | add / update / export validate | ✅ |
| CLI CRUD | §CLI Plan | `main.rs` Participant サブコマンド | ✅ |
| export schema v4 | §Export Plan | `TRIP_EXPORT_SCHEMA_VERSION = 4` | ✅ |
| import v4 | §Import Plan | `import_export_participants` | ✅ |
| v3 import 互換 | §Import Plan | v3 パス、participants 省略可 | ✅ |
| diff | §Diff Plan | `compute_participants_diff` | ✅ |
| export-md | §Markdown Plan | `append_participants_section` | ✅ |
| stats | §Stats Plan | `TripStats` 拡張 | ✅ |
| doctor / advisor | §Doctor Plan | 3 issue code（multiple self は warning） | ✅ |
| tests / golden | §Test Plan | 下記 §Test Coverage | ✅ 軽微ギャップあり |
| docs | §Docs Plan | 主要 doc 更新済み | ✅ draft 拡充は #12 |

### スコープ外への踏み込み（なし）

| 除外項目 | コード確認 |
|---|---|
| Person / Traveler Profile | `persons` テーブル・`person_id` 列なし |
| Expense FK / `paid_by_participant_id` | `expense.rs` 変更なし |
| Expense shares / Settlement | 未実装 |
| Reservation guest linking | `reservation.rs` に Participant FK なし |
| User / cloud sync | 未実装 |
| v1.23 stash | 未取り込み（`stash@{0}` 温存） |

**結論:** 実装範囲は Implementation Plan に収まっている。

---

## Design Consistency Review

### 設計前提の実装対照

| 前提 | 実装 | 判定 |
|---|---|---|
| `participants` = Trip-scoped participation record | Trip 配下 CRUD、export は Trip 直下 | ✅ |
| Participant = 自分を含む旅行者全員 | 文言・stats・docs 一致 | ✅ |
| Companion = 自分以外 | `companion_count` は self 1 件時のみ `N-1` | ✅ |
| `is_self` = Trip participation 側マーカー | DB 列 + CLI + export | ✅ |
| `companion_count` は `self_count == 1` のときのみ | `compute_participant_counts` | ✅ |
| self unknown 時は機械算出しない | `participant_count` / `companion_count` = `None` | ✅ |
| participants 未登録 ≠ 0 人旅行 | `participants_recorded: false` → "not recorded" | ✅ |

### count semantics（核心）

```rust
// src/participant.rs — compute_participant_counts
if self_count == 1 {
    companion_count: Some(registered_count.saturating_sub(1)),
    self_known: true,
} else {
    companion_count: None,
    self_known: false,
}
```

| 状態 | `participant_count` | `companion_count` | 表示例 |
|---|---|---|---|
| 0 件（未登録） | `None` | `None` | `Participants: not recorded` |
| N 件、self 1 件 | `Some(N)` | `Some(N-1)` | `Participants: 2 (companions: 1)` |
| N 件、self 0 件 | `None` | `None` | `… recorded (traveler count unknown)` |
| N 件、self 2 件以上 | `None` | `None` | doctor: MultipleSelfParticipants (warning) |

**結論:** #21 / #22 / participant-model の count 意味論と一致。無条件の `count(participants) - 1` は存在しない。

---

## CLI Review

### コマンド対照

| コマンド | 実装 | 判定 |
|---|---|---|
| `participant add --trip <id> --name <name> [--sort-order N] [--self]` | `ParticipantAction::Add` | ✅ |
| `participant list --trip <id> [--json]` | list + `ParticipantListJson` | ✅ |
| `participant show <id> [--json]` | show + JSON は `Participant` 直列化 | ✅ |
| `participant update <id> [--name] [--sort-order] [--self \| --not-self]` | transaction 内 self 付け替え | ✅ |
| `participant delete <id>` | 単純 DELETE | ✅ |

### 挙動確認

| 観点 | 期待 | 実装 | テスト |
|---|---|---|---|
| `--self` on add | 既存 self あり → エラー | `trip already has a self participant` | unit + CLI |
| `--self` on update | 他行 self を false にして付け替え | `UPDATE … SET is_self = 0 WHERE trip_id …` | unit + CLI |
| `--not-self` | self 0 件を許容 | `set_self = Some(false)` | CLI |
| self 0 件 | 削除・not-self 後も許容 | count unknown 扱い | unit + CLI |
| sort_order 未指定 | max + 1 | `next_sort_order_for_trip` | unit（暗黙） |
| human output | 表 + counts フッター | `print_participant_list_human` | CLI |
| JSON output | `counts` ネスト | `ParticipantListJson` | CLI |
| エラー | trip / participant not found | exit 1、内部エラー非露出 | `not_found_cli.rs` |

**結論:** Implementation Plan の CLI 方針と一致。

---

## Data Integrity Review

| 観点 | 実装 | 判定 |
|---|---|---|
| migration 冪等性 | `CREATE TABLE/INDEX IF NOT EXISTS` | ✅ `test_migrate_participants_idempotent` |
| partial unique index | `WHERE is_self = 1` on `trip_id` | ✅ SQLite で CI 通過 |
| app-side validation | add 前 count、update 付け替え | ✅ |
| Trip delete cascade | `delete_participants_for_trip` in `delete_trip` tx | ✅ unit |
| Trip duplicate | `import_trip_from_export_v3` が participants 復元 | ✅ unit + CLI roundtrip |
| `db reset` | `DELETE FROM participants` + sequence | ✅ `test_reset_db_clears_participants` |
| FK なし + manual cascade | Entity Design 方針どおり | ✅ |

### partial unique index のテスト状況

| ケース | テスト |
|---|---|
| 同一 Trip に self 2 件 → rejected | `test_self_max_one_on_add`（app） |
| 別 Trip に self 1 件ずつ → OK | `test_duplicate_participants_for_trip`（2 trips） |
| self 0 件 → OK | `test_participant_crud_and_counts`（削除後） |
| update で self 付け替え → OK | `test_update_self_transfer` |

DB 層の unique 違反を直接叩く integration テストはないが、index 定義と app 検証の二重化は計画どおり。**Release blocker ではない。**

**結論:** データ整合性方針は Entity Design / Implementation Plan と一致。

---

## Export / Import Review

| 観点 | 実装 | 判定 |
|---|---|---|
| `schema_version: 4` | `finalize_trip_export_v3` | ✅ |
| top-level `participants[]` | `TripExportV3.participants` | ✅ |
| `is_self` export | `ExportParticipantV4` | ✅ |
| internal id 非 export | `build_export_participants` は name/sort_order/is_self のみ | ✅ |
| v3 import 互換 | v3 パス、participants 省略 = `[]` | ✅ roundtrip 既存テスト継続 |
| v4 roundtrip | `cli_participant_export_v4_roundtrip` | ✅ |
| multiple self invalid | `collect_export_participant_validation_errors` | ✅ unit |
| participants 未登録 | export は `participants: []` | ✅ |
| canonical sample | `expected-export-v3.json` → schema 4 + `participants: []` | ✅ okinawa seed CLI |

### v3 互換の根拠

- `parse_trip_export_for_import`: `TRIP_EXPORT_SCHEMA_VERSION_V3 | TRIP_EXPORT_SCHEMA_VERSION` を同一 V3 構造で deserialize。
- v3 JSON に `participants` キーがなくても import 成功（空配列）。
- 既存 `export_roundtrip_cli` / okinawa seed は schema 4 期待値に更新済みだが、**v3 ファイルの import** は `trip_import_cli` legacy テスト等で継続確認。

**結論:** export v4 化は v3 import を壊していない。

---

## Diff Review

`src/diff.rs` — `compute_participants_diff`

| 差分種別 | 検出方法 | 判定 |
|---|---|---|
| added | 新 JSON のみのキー（またはペア余剰） | ✅ |
| removed | 旧 JSON のみ | ✅ |
| renamed | `(sort_order, name)` キー変化 → removed + added | ✅ 保守的パターン |
| reordered | 同上（キーに sort_order 含む） | ✅ |
| `is_self` changed | 同一キー・出現順ペアで field diff | ✅ |

### Identity 方針

- **複合キー:** `(sort_order, name)`（Implementation Plan どおり）。
- **同名許可:** 同一キーが複数ある場合は配列内出現順でペアリング（Note diff と同型の限界）。

### 限界・将来改善

| 限界 | 影響 | 分類 |
|---|---|---|
| rename + reorder 同時 | removed + added として検出（過剰検出の可能性） | 文書化済み・許容 |
| 同名 + 同一 sort_order 複数 | 出現順ペアリングに依存 | 許容（v2 は同名許可） |
| rename / reorder 専用テストなし | `is_self` changed の unit のみ | non-blocking follow-up |

**結論:** Diff Plan の最低要件を満たす。限界は計画どおり文書化可能。

---

## Markdown Export Review

| 観点 | 実装 | 判定 |
|---|---|---|
| Trip Overview 付近に Participants | Overview の直後に `## Participants` 表 | ✅ |
| Day / Itinerary に出さない | `append_participants_section` は Trip スコープのみ | ✅ |
| self marker | 表の `yes` / `no` | ✅ |
| 0 件時 | セクション省略（`participants.is_empty()` で return） | ✅ 計画「省略推奨」 |
| Overview 統計行 | self_known 時 Travelers、否则 unknown 文言 | ✅ |

**結論:** Markdown Export Plan と一致。専用 integration テストは未追加（§Known Gaps）。

---

## Stats / Doctor / Advisor Review

### stats (`src/stats.rs`)

| フィールド | 意味 | 判定 |
|---|---|---|
| `registered_participant_count` | DB 登録数 | ✅ |
| `participants_recorded` | count > 0 | ✅ |
| `self_known` | self ちょうど 1 件 | ✅ |
| `participant_count` / `traveler_count` | self_known 時のみ Some | ✅ |
| `companion_count` | self_known 時のみ Some | ✅ |

未登録一人旅: `Participants: not recorded`（0 人表示しない）。

### doctor / advisor

| Code | 重大度 | 条件 | 判定 |
|---|---|---|---|
| `PARTICIPANTS_NOT_RECORDED` | info | participants 0 件 | ✅ |
| `SELF_PARTICIPANT_UNKNOWN` | info | 登録あり・self 0 件 | ✅ |
| `MultipleSelfParticipants` | warning | self 2 件以上 | ✅ |

advisor は上記に対する advice / try ヒント（`participant add --self` 等）を提供。

**結論:** count semantics と doctor 方針は設計と一致。

---

## Test Coverage Review

### カバレッジ一覧

| 観点 | テスト所在 | 判定 |
|---|---|---|
| migration 冪等 | `participant.rs` `test_migrate_participants_idempotent` | ✅ |
| `init_db` creates table | **未追加** | ⚠️ 代替あり（下記） |
| CRUD | `test_participant_crud_and_counts` + CLI | ✅ |
| `--self` / `--not-self` | unit + `participant_cli.rs` | ✅ |
| self 最大 1 件 | `test_self_max_one_on_add` + CLI | ✅ |
| self 0 件 | CRUD test + CLI delete 後 | ✅ |
| trip delete cascade | `test_delete_participants_for_trip_cascade` | ✅ |
| trip duplicate | `test_duplicate_participants_for_trip` + export duplicate 経由 | ✅ |
| export v4 | CLI roundtrip + export_roundtrip schema 4 | ✅ |
| import v4 | CLI roundtrip | ✅ |
| v3 import 互換 | `trip_import_cli` legacy、`export_roundtrip` 構造比較 | ✅ 明示的 v3 participants 省略テストは薄い |
| invalid export multiple self | `test_validate_export_multiple_self`（unit） | ⚠️ integration なし |
| diff | `test_diff_participants_added_removed_and_is_self_changed` | ✅ rename/reorder なし |
| export-md Participants | **専用テストなし** | ⚠️ Overview 既存テストのみ |
| stats | `stats.rs` tests + CLI | ✅ |
| doctor / advisor | `doctor.rs` / `advisor.rs` tests 更新済み | ✅ |
| JSON output | CLI `--json` | ✅ |
| canonical / golden | okinawa `expected-export-v3.json` schema 4 | ✅ |

### PR #24 レビュー時の軽微所見（再掲・判定）

| 所見 | 代替・状況 | 判定 |
|---|---|---|
| `test_init_db_creates_participants_table` 未追加 | `test_migrate_participants_idempotent` + `open_db_at` 経由で migrate 実行 | **Non-blocking** |
| validate-export multiple self は unit のみ | `collect_export_participant_validation_errors` は import/validate 双方で呼ばれる | **Non-blocking**（CLI validate integration は follow-up 可） |
| export-md Participants 専用テストなし | 手動・`export_md_cli` 既存で regression リスク低 | **Non-blocking** |

---

## Known Gaps

実装バグではなく、テスト・ドキュメントの薄い箇所。

1. `db.rs` に `test_init_db_creates_participants_table` がない（計画記載あり）。
2. `validate-export` の multiple self を叩く **integration** テストがない。
3. `export-md` の `## Participants` セクションを assert するテストがない。
4. diff の rename / reorder 専用テストがない。
5. v3 export ファイル（`participants` キーなし）を明示 import する integration が薄い（回帰は legacy import で間接カバー）。
6. `duplicate_participants_for_trip` は production では export/import 経由が主経路（関数は unit test のみ・`#[allow(dead_code)]`）。

いずれも **Release blocker ではない**。

---

## Release Blockers

**なし。**

PR #24 時点・本レビュー時点で、v2.0.0 Participant Foundation のリリースを妨げる仕様ズレ・データ破壊・v3 互換破壊・count semantics 不備は確認されなかった。

---

## Non-blocking Follow-ups

| # | 内容 | 推奨タイミング |
|---|---|---|
| 1 | `test_init_db_creates_participants_table` を `db.rs` に追加 | v2.0.x パッチ or Maintenance Issue |
| 2 | `validate-export` multiple self の CLI integration テスト | 同上 |
| 3 | `export-md` Participants セクションの assertion | 同上 |
| 4 | diff rename / reorder ケースの unit テスト | Maintenance |
| 5 | v3 JSON（participants 省略）明示 import integration | Maintenance |
| 6 | [v2.0.0-notes.md](../releases/v2.0.0-notes.md) の Release #12 向け拡充（CLI 例・migration 注意） | **Issue #12** |
| 7 | [export-import.md](../export-import.md) の schema 説明を v4 中心に更新（現状やや旧記述） | **Issue #12** docs |

---

## Deferred Scope

### v3 Shared Expense

- `paid_by_participant_id`、beneficiary、Settlement は未実装（計画どおり）。
- v2 Participant の安定 `id` が v3 の参照先になる前提は維持されている。

### Future Person / Traveler Profile

- `persons` テーブル、`participants.person_id` は未実装。
- `is_self` は将来も Trip participation 側マーカーとして残る設計（#7 / #8）。

### v5 Travel Book

- Participant 一覧の外部出力は export-md / export v4 で足りる現段階。Travel Book 固有レイアウトは未着手。

### v6 Travel Journal

- Participant を Journal ナラティブに展開する機能は未着手。

---

## Release Readiness

### 判定

| 項目 | 判定 |
|---|---|
| v2.0.0 release blocker | **なし** |
| docs / release notes | [v2.0.0-notes.md](../releases/v2.0.0-notes.md) は **draft として十分**。#12 で正式化・拡充 |
| Cargo.toml / Cargo.lock version bump | **Issue #12 で実施**（実装 PR では意図的に未 bump — 妥当） |
| tag / GitHub Release | **Issue #12 で実施** |
| **Issue #12 に進めるか** | **はい — 進めてよい** |

### Release #12 で行うこと（参考）

1. `Cargo.toml` / `Cargo.lock` を `2.0.0` に bump。
2. `v2.0.0-notes.md` を正式 Release Notes に仕上げる。
3. Git tag + GitHub Release 作成。
4. （任意）Non-blocking follow-up を Maintenance Issue 化。

### レビュー結論（一文）

```text
Participant 実装は設計系列 #7–#10 および PR #21 / #22 / #24 と整合しており、
v2.0.0 Release（Issue #12）に進める状態である。
```

---

## Completion Criteria

| # | 条件 | 状態 |
|---|---|---|
| 1 | 本書 `participant-post-implementation-review.md` が存在 | ✅ |
| 2 | #7 / #8 / #9 / #10 / #21 / #22 / #24 との整合確認 | ✅ |
| 3 | release blocker の有無が明確 | ✅ なし |
| 4 | non-blocking follow-up 整理 | ✅ §Non-blocking Follow-ups |
| 5 | deferred scope 整理 | ✅ §Deferred Scope |
| 6 | Issue #12 進行可否の明記 | ✅ 進めてよい |
| 7 | 関連 doc からのリンク | ✅ README 等（本 PR） |
| 8 | 大きな実装変更なし | ✅ documentation-only |
| 9 | v1.23 stash 未混入 | ✅ |

---

## Next phase notes（Release #12）

Post-Implementation Review 完了後、Issue #12 `[Release] v2.0.0` でバージョン bump・正式 Release Notes・tag を行う。本書の §Non-blocking Follow-ups は Release をブロックしない。
