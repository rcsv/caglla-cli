# Foundation Hardening Review

Caglla.Travel CLI **v2.0.0 Participant Foundation リリース後**、**v3 Shared Expense 着手前**に、v1 Planning Foundation + v2 Participant Foundation の土台を点検するレビューです。

**本書の位置づけ:**

```text
新機能追加ではない
v3 着手前の足場固め（点検・分類・提案）
Release blocker の有無を明確にする
```

| 項目 | 状態 |
|---|---|
| 最新リリース | **v2.0.0** Participant Foundation |
| `Cargo.toml` | `2.0.0`（本レビューでは bump しない） |
| v3 Shared Expense | **未着手** |
| 本書 | **documentation-only** |

---

## Purpose

v1 系 Planning Foundation（v1.0.0–v1.22.0 + 完了総括文書）と v2.0.0 Participant Foundation を、v3 Shared Expense（`paid_by_participant_id`、beneficiary、Settlement）に進める **安定した参照土台** にする。

点検対象:

- Post-Implementation Review の non-blocking follow-up
- Planning Foundation 総括後の軽微な穴
- export v4 / import / diff / export-md / stats / doctor / advisor の整合
- canonical sample / golden / ユーザー向け docs
- v3 前に片付けるべき小さな不安要素

**やらないこと（本フェーズ）:** Shared Expense FK、Settlement、Person / Traveler Profile、Cargo bump、tag / GitHub Release。

---

## Current Baseline

| レイヤ | 到達点 |
|---|---|
| **Planning Foundation** | Trip / Day / Itinerary / Checklist / Note / Expense / Summary / Reservation + export v3 + Hardening v1.19–v1.22 + [planning-foundation-completion-review.md](planning-foundation-completion-review.md) |
| **Participant Foundation** | `participants` CRUD、`is_self`、export **schema v4**、import v3 互換、diff / export-md / stats / doctor / advisor 対応 — **v2.0.0 リリース済み** |
| **Export 現行** | `schema_version: 4`（top-level `participants[]`） |
| **Canonical sample** | `okinawa_sesoko_2026` — golden `expected-export-v3.json` は **schema 4 + `participants: []`** |
| **CI** | `make check` PASS（262 unit + integration tests） |

---

## Source Documents

| 文書 | 役割 |
|---|---|
| [planning-foundation-completion-review.md](planning-foundation-completion-review.md) | v1 総括・既知ギャップ §9 |
| [participant-post-implementation-review.md](participant-post-implementation-review.md) | v2 実装後レビュー・§Non-blocking Follow-ups |
| [participant-implementation-plan.md](participant-implementation-plan.md) | v2 実装計画・テスト計画 |
| [v2.0.0-notes.md](../releases/v2.0.0-notes.md) | 正式 Release Notes |
| [export-schema.md](export-schema.md) | export JSON **一次仕様**（v4 追記あり、後述ギャップあり） |
| [export-import.md](../export-import.md) | ユーザー向け手順 |
| [github-workflow.md](../github-workflow.md) | Issue / PR / Milestone 運用 |

---

## Review Targets

| 領域 | 判定（2026-06 点検） | 不安度 |
|---|---|---|
| Participant CRUD + export v4 roundtrip | ✅ `participant_cli.rs` + unit | 低 |
| import v3（participants 省略） | ✅ legacy import 経由で間接カバー | 中（明示 integration は薄い） |
| validate-export v4 | ✅ 現行 export PASS；multiple self は **unit のみ** | 中 |
| trip diff Participant | ✅ add/remove/is_self；rename/reorder は **removed+added 扱い**（仕様どおり） | 低 |
| export-md Participants | ✅ 実装済み；**専用 assertion なし** | 中 |
| stats / doctor / advisor | ✅ unit + CLI JSON 更新済み | 低 |
| canonical / golden | ✅ okinawa seed + schema 4 golden | 低 |
| ユーザー向け docs | ⚠️ 冒頭は v4 対応、**中盤・仕様 doc に旧記述残存** | 中 |

**Release blocker: なし**（v2.0.0 時点と同様）。

---

## Participant Follow-ups

[v2.0.0-notes.md](../releases/v2.0.0-notes.md) / [participant-post-implementation-review.md](participant-post-implementation-review.md) で non-blocking とされた 4 項目の分類:

| # | 項目 | 分類 | 理由 |
|---|---|---|---|
| 1 | `test_init_db_creates_participants_table` 未追加 | **D**（必要なら **A**） | `test_migrate_participants_idempotent` + `open_db_at` / CRUD integration で **機能はカバー**。他テーブルと同型の `init_db` テストは **一貫性・可読性** のため v2.0.x hardening で追加してよい |
| 2 | validate-export multiple self — unit のみ | **A** | `collect_export_participant_validation_errors` は import/validate 双方から呼ばれるが、**CLI 経路の regression 用 integration** が無い。v3 前の小さな hardening PR に適する |
| 3 | export-md `## Participants` — 専用テストなし | **A** | 実装は `markdown.rs` に存在。`export_md_cli.rs` は Overview のみ assert。**v2 表面の明示テスト**として追加コストが低い |
| 4 | diff rename / reorder — 専用テストなし | **C** | キーは `sort_order` + `name`。rename/reorder は **removed + added** として保守的に検出（[v2.0.0-notes.md](../releases/v2.0.0-notes.md) 記載）。add/remove/is_self の unit あり。**v3 blocker ではない** |

### participant-post-implementation-review の追加 follow-up（#5–#7）

| # | 内容 | 分類 |
|---|---|---|
| 5 | v3 JSON（participants 省略）明示 import integration | **B** — v3 Shared Expense 前に v3 互換の明示テストがあると安心 |
| 6 | v2.0.0-notes Release #12 向け拡充 | **— 完了**（v2.0.0 リリース済み）。**残**: stash 行など **陳腐化記述の修正** → docs hardening |
| 7 | export-import.md v4 中心更新 | **B** — 冒頭は v4、本文・例・diff 節が旧い（下記 §Documentation） |

---

## Export / Import Review

| 観点 | 状態 | メモ |
|---|---|---|
| 現行 export | `schema_version: 4` | `trip.rs` `test_export_schema_version_is_four` ✅ |
| v4 → import roundtrip | ✅ | `cli_participant_export_v4_roundtrip` |
| v3 import（participants 省略） | ✅ 実装 | `trip_import_cli` legacy、`export_roundtrip` 構造比較 — **participants 省略の明示 fixture は薄い** |
| validate-export v4 rules | ✅ 実装 | multiple self invalid — unit `test_validate_export_multiple_self` |
| duplicate | ✅ | participant duplicate 経由 |
| downgrade 注意 | ✅ docs | v1.22 CLI は DB 内 participants 非対応 — [v2.0.0-notes.md](../releases/v2.0.0-notes.md) |

**不安点（non-blocking）:**

- v3 fixture の明示 integration（分類 **B**）
- validate-export multiple self の CLI integration（分類 **A**）

---

## Diff / Markdown Review

### trip diff

| 対象 | 状態 |
|---|---|
| Note / Summary / Reservation | ✅ integration / unit |
| Participant add / remove / `is_self` | ✅ `test_diff_participants_added_removed_and_is_self_changed` |
| Participant rename / reorder | 保守的 removed+added（専用テストなし）— **C** |
| Expense | ❌ **v1 既知ギャップ**（[planning-foundation-completion-review.md](planning-foundation-completion-review.md) §9）— v3 前 Maintenance 候補 |

### export-md

| セクション | テスト |
|---|---|
| Overview / Expense / Checklist / Reservation / Summary | ✅ 各所に coverage |
| `## Participants` | ⚠️ **専用 assertion なし** — **A** |
| Long-form Note entity | ❌ v1 既知ギャップ — Maintenance |

---

## Stats / Doctor / Advisor Review

| 機能 | Participant 対応 | テスト |
|---|---|---|
| `trip stats` | count semantics（self 1 件時のみ companion） | ✅ `stats.rs` + CLI |
| `trip doctor` | PARTICIPANTS_NOT_RECORDED / SELF_UNKNOWN / MULTIPLE_SELF | ✅ `doctor.rs` |
| `trip advisor` | 上記 issue code 向け hints | ✅ `advisor.rs` + `doctor_advisor_json_cli.rs` |

**判定:** v2.0.0 設計（[participant-model.md](participant-model.md) count 意味論）と整合。**追加 blocker なし**。

---

## Test Coverage Review

### Integration tests（`tests/`）

| ファイル | 主 coverage |
|---|---|
| `participant_cli.rs` | CRUD、self 制約、v4 export roundtrip |
| `export_roundtrip_cli.rs` | checklist / notes / expenses（schema 4 暗黙） |
| `export_md_cli.rs` | stdout/file/expense — **Participants なし** |
| `validate_export_cli.rs` | 現行 export、v3 expense エラー — **multiple self なし** |
| `okinawa_sesoko_seed_cli.rs` | canonical golden + validate-export |
| `doctor_advisor_json_cli.rs` | JSON envelope |

### Golden

| ファイル | 内容 |
|---|---|
| `samples/okinawa_sesoko_2026/expected-export-v3.json` | 名前は v3 だが **schema 4** — seed 後 export の正規化比較 |

**判定:** コア path は green。**Participant 表面の regression 用に 2–3 本の小テスト追加**を v3 前 hardening で推奨（§Recommended Follow-up Issues）。

---

## Documentation Consistency Review

### 確認結果サマリー

| 文書 | latest v2.0.0 | schema v4 | Participant / is_self | Person 非対象 | v3 未着手 | v1.23 release 前提 |
|---|---|---|---|---|---|---|
| [README.md](../../README.md) | ✅ | 間接 | ✅ | — | — | ✅ なし |
| [data-model.md](../data-model.md) | — | — | ✅ v2.0.0 実装済み | ✅ | ✅ | ✅ |
| [export-import.md](../export-import.md) | — | 冒頭 ✅ / 例・diff 節 ⚠️ | 冒頭 ✅ / import 表 ⚠️ | — | — | ✅ |
| [markdown-export.md](../markdown-export.md) | — | — | ✅ Participants 節 | — | — | ✅ |
| [command-reference.md](../command-reference.md) | — | — | ✅ participant 節 | — | — | ✅ |
| [development.md](../development.md) | — | — | 軽微 | — | — | ✅ |
| [long-term-version-strategy.md](../long-term-version-strategy.md) | ✅ | — | ✅ | ✅ | ✅ v3 Shared Expense | ✅ tag なし明記 |
| [specifications/README.md](README.md) | — | ⚠️ 索引が v3 まで | ✅ | ✅ | — | ✅ |
| [releases/README.md](../releases/README.md) | ✅ v2.0.0 先頭 | — | — | — | — | ✅ |
| [v2.0.0-notes.md](../releases/v2.0.0-notes.md) | — | ✅ | ✅ | ✅ | ✅ | ⚠️ stash 行が陳腐化 |

### 主要な不整合（優先度順）

1. **[export-schema.md](export-schema.md)** — `validate-export` 節が v3 まで、`非対象` に「Participant / Settlement」と記載。**v4 実装と矛盾**（一次仕様 doc）。**v3 前に docs PR 必須級**。
2. **[export-import.md](../export-import.md)** — 冒頭は v4 対応だが、JSON 例は schema 2、import 表に Expense/Reservation/Participant 不足、`trip diff` 節が Note 中心の旧記述。
3. **[specifications/README.md](README.md)** — `export-schema.md` 説明が「v1/v2/v3」のみ（v4 未記載）。
4. **陳腐化参照** — [v2.0.0-notes.md](../releases/v2.0.0-notes.md) `v1.23 stash | Not merged`、[participant-post-implementation-review.md](participant-post-implementation-review.md) stash / Issue #12 進行中表現 — **PR #26 / Release 完了後未更新**。

### v1 Planning Foundation 既知ギャップ（引き続き valid）

[planning-foundation-completion-review.md](planning-foundation-completion-review.md) §9:

| # | 項目 | v3 前の扱い |
|---|---|---|
| 1 | `trip diff` — Expense | Maintenance（**B**）— Shared Expense 設計前に diff 方針を決めるとよい |
| 2 | `export-md` — Long-form Note | Maintenance（**C**） |
| 3 | canonical Reservation/Summary 共存 | **C** |
| 4 | Checklist Hardening | **C** |
| 5 | export-import.md v4 追従 | **B** — 本レビュー §Documentation #2 と同一 |

---

## Recommended Follow-up Issues

v3 Epic とは **別 Milestone / Maintenance label** で小 PR に分割することを推奨。

### テスト hardening（Rust、小）

| 提案 Issue / PR | 内容 | 分類 | 優先 |
|---|---|---|---|
| `test: add participant init_db table coverage` | `test_init_db_creates_participants_table` | A / D | 低 |
| `test: validate-export rejects multiple self (CLI)` | invalid v4 JSON fixture | A | 中 |
| `test: export-md includes Participants section` | `participant add` → assert `## Participants` | A | 中 |
| `test: import v3 export without participants key` | 明示 v3 fixture | B | 中 |
| `test: participant diff rename as remove+add` | 仕様ドキュメント化を兼ねる | C | 低 |

### ドキュメント hardening（docs-only）

| 提案 Issue / PR | 内容 | 優先 |
|---|---|---|
| `docs: align export-schema.md with v4` | validate-export v4、Participant 節、非対象リスト修正 | **高** |
| `docs: align export-import.md with v2.0.0` | 例・import 表・trip diff 節更新 | **高** |
| `docs: refresh v2.0.0-era stale references` | stash / Issue #12 完了後の文言 | 中 |
| `docs: update specifications README export-schema blurb` | v4 明記 | 低（本 PR で一部対応可） |

### v1 系 Maintenance（v3 と並行可）

| 提案 | 内容 |
|---|---|
| `trip diff` Expense 方針 | v3 Shared Expense 設計 Issue と連動 |
| `export-md` Long-form Note | Travel Book 前の改善 |

---

## Release Recommendation

### v2.0.1 を今すぐ出すべきか

**いいえ — 必須ではない。**

| 観点 | 判定 |
|---|---|
| Release blocker | **なし** |
| ユーザー影響 bug | 確認されず |
| テスト / doc follow-up | Maintenance として **v3 前に land 可能** |
| Cargo bump の要否 | follow-up が **docs-only のみ** なら bump 不要。**test-only hardening をまとめて patch リリースする**選択肢はあるが v3 着手の blocker ではない |

### 推奨シーケンス

```text
1. 本レビュー merge（Foundation Hardening Review 文書化）
2. docs PR: export-schema.md + export-import.md v4 整合（blocker 級 doc）
3. test PR: validate-export multiple self + export-md Participants（小）
4. （任意）v2.0.1 tag — 2+3 をまとめた場合のみ検討
5. v3 Shared Expense Epic 起票
```

---

## Completion Criteria

**Foundation Hardening フェーズ**（v3 着手前）の完了条件:

| # | 条件 | 本 PR 時点 |
|---|---|---|
| 1 | 本レビュー文書が存在 | ✅ |
| 2 | Participant follow-up が A/B/C/D 分類済み | ✅ §Participant Follow-ups |
| 3 | export/import/diff/markdown/stats/doctor の不安点整理 | ✅ |
| 4 | documentation consistency 点検 | ✅ §Documentation |
| 5 | v3 前 follow-up Issue 提案 | ✅ §Recommended Follow-up Issues |
| 6 | v3 Shared Expense 未着手 | ✅ |
| 7 | Cargo bump / tag / Release なし | ✅ |
| 8 | `make check` PASS | ✅（PR CI 確認） |

**v3 Shared Expense 着手の gate（推奨）:**

- [ ] `export-schema.md` v4 整合 PR merge
- [ ] Participant 関連 test hardening（少なくとも **A** 分類 2 件）merge または Issue 化
- [ ] v3 Epic / Responsibilities Review 起票

---

## 参照

| 領域 | ドキュメント |
|---|---|
| v1 総括 | [planning-foundation-completion-review.md](planning-foundation-completion-review.md) |
| v2 実装後 | [participant-post-implementation-review.md](participant-post-implementation-review.md) |
| ロードマップ | [long-term-version-strategy.md](../long-term-version-strategy.md) |
| GitHub 運用 | [github-workflow.md](../github-workflow.md) |
| v2.0.0 Release | [v2.0.0-notes.md](../releases/v2.0.0-notes.md) |
