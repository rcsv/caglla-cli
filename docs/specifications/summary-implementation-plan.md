# Summary Implementation Plan（実装計画）

Caglla.Travel CLI に **Trip / Day Summary** を実装する場合の計画メモです。

**v1.16.0 時点: 仕様整理のみ。** DB migration、CLI、export schema、Markdown export、テストの変更は行いません。

| ドキュメント | 役割 |
|---|---|
| [travel-ledger-responsibilities.md](travel-ledger-responsibilities.md) (v1.10.0) | Summary / Remark / Note の横断比較 |
| [summary-responsibilities-review.md](summary-responsibilities-review.md) (v1.14.0) | 責務・境界（What it is / is not） |
| [summary-entity-design.md](summary-entity-design.md) (v1.15.0) | フィールド・格納・表示方針（How we might model it） |
| **本書** (v1.16.0) | 実装計画（If we build it, how） |

関連: [Day モデル](day-model.md) / [Note モデル](note-model.md) / [Export Schema](export-schema.md) / [Itinerary モデル](itinerary-model.md)

---

## 1. 実装目的

Trip / Day に **summary** を追加し、以下に反映する。

| 領域 | 目的 |
|---|---|
| **DB** | `trips.summary` / Day 側 summary 格納（§2 — 既存列との整理含む） |
| **CLI** | 入力・表示・JSON 出力（`trip show` / `day show`、将来 `day update`） |
| **export / import** | JSON roundtrip で summary を保持 |
| **export-md** | しおり冒頭・Day 見出し直下に要約を表示 |
| **trip diff** | summary 変更の検出（任意だが推奨） |

実装の主目的は **旅行前の計画共有**（しおり・同行者向け概要）と、Note / Remark への要約の混在を減らすことです。canonical sample（旅行後台帳）への一括投入は **別フェーズ**（§10）。

前提は [summary-entity-design.md](summary-entity-design.md) — Trip/Day の短い plain text、Itinerary.summary なし、NULL 許可、export は v3 拡張または v4 再評価。

---

## 2. DB migration plan

### 追加・整理する列

| テーブル | 列 | 型 | 状態 |
|---|---|---|---|
| `trips` | `summary` | TEXT NULL | **新規追加** |
| `days` | `summary` | TEXT NULL | **新規追加または既存列の整理**（下記） |

### 既存 `days.description` との関係（実装時に決定）

現行 DB には **`days.description`** が既に存在する（[day-model.md](day-model.md) — 章立て・GUI 用として未活用）。export v3 には **含まれていない**。

| 方式 | 内容 | 長所 | 短所 |
|---|---|---|---|
| **A. 列リネーム** | migration で `description` → `summary` にリネーム（SQLite は RENAME COLUMN 可） | 用語が entity design と一致 | `day list --json` の `description` キーが breaking |
| **B. 意味の再割当** | DB 列名は `description` のまま、ドキュメント・export キーのみ `summary` | migration 軽い | DB と用語が不一致 |
| **C. 新列追加** | `summary` を追加し、`description` は非推奨のまま残す | 既存 API 無変更 | 二重フィールド |

**推奨（案）:** **A または B**。データ未使用が多いため **A（リネーム）** を第一候補。`day list --json` の `description` は `summary` へ変更（breaking — JSON 消費者が少ない想定）。

`days.title` は **変更しない**（Day 章タイトル用として [day-model.md](day-model.md) と共存）。

### Migration 方針

既存パターン: `src/db.rs` の `add_column_if_not_exists`（Expense 追加時と同型）。

```sql
-- Trip（新規）
ALTER TABLE trips ADD COLUMN summary TEXT;

-- Day（方式 A の場合）
ALTER TABLE days RENAME COLUMN description TO summary;

-- Day（方式 C の場合）
ALTER TABLE days ADD COLUMN summary TEXT;
```

| 考慮点 | 方針 |
|---|---|
| **既存データ** | すべて NULL（または description が空）— **backfill 不要** |
| **空文字** | アプリ層で **NULL 正規化**（INSERT/UPDATE 前に trim + 空なら NULL） |
| **Trip 作成時** | `trip add` — summary 未指定なら NULL |
| **Day 自動生成** | 新規 Day 行の summary は NULL |
| **end_date 短縮** | 現行どおり Day 削除判定 — summary のみ入っている Day は **削除ブロック**（description と同様の扱いを `summary` に移行） |
| **rollback** | 必須ではない。downgrade 時は列が余るだけ — **読み取りは NULL 扱いで互換** |
| **互換性注意** | 旧バイナリ + 新 DB → 未知列は SQLite が無視しない（SELECT * で落ちうる）— **前方互換は新 CLI 必須** と明記 |

### `db reset` / 新規インストール

`CREATE TABLE` 定義に `trips.summary` / `days.summary` を含める。migration と初期スキーマを **同期**。

---

## 3. Model / Repository plan

### 更新対象（Rust）

| 領域 | ファイル | 変更内容 |
|---|---|---|
| **Trip model** | `src/models.rs` — `Trip` | `pub summary: Option<String>` 追加 |
| **Day model** | `src/models.rs` — `Day` | `description` → `summary`（方式 A）または両方整理 |
| **Trip CRUD** | `src/trip.rs` — `add_trip`, `get_trip`, `update_trip`, `list_trips` | SELECT/INSERT/UPDATE に summary |
| **Day 読取** | `src/day.rs` — `list_days`, `get_day`, row mapping | summary 列の読み書き |
| **Day 更新** | `src/day.rs` — **新規** `update_day` | summary 更新（§4） |
| **Export v3 DTO** | `src/models.rs` — `Trip`, `ExportDayV3` | `trip.summary`; `days[].summary` |
| **Export 構築** | `src/trip.rs` — `build_trip_export_v3` | DB から summary を埋める |
| **Import** | `src/trip.rs` — import パス | trip / day summary を復元 |
| **validate-export** | `src/trip.rs` | summary 長さ・空文字（任意） |
| **duplicate** | `src/trip.rs` | export/import 経由で summary も複製 |
| **Markdown** | `src/markdown.rs` | §6 |
| **diff** | `src/diff.rs` | §7 |
| **main.rs** | CLI フラグ配線 | §4 |

### Export / import 用 DTO

```rust
// Trip（export の trip オブジェクト）
pub struct Trip {
    // 既存フィールド
    pub summary: Option<String>,  // 追加
}

// ExportDayV3
pub struct ExportDayV3 {
    pub day_number: i64,
    pub summary: Option<String>,  // 追加 — skip_serializing_if = None
    pub itineraries: Vec<ExportItineraryV3>,
}
```

`Trip` struct は DB と export で **共用** されているため、export に `id` を出さない既存 serde 設定を維持する。

### duplicate / diff / stats への影響

| 機能 | 影響 |
|---|---|
| **`trip duplicate`** | **あり** — import 経由で summary 複製 |
| **`trip diff`** | **あり** — §7 |
| **`trip stats`** | **なし**（原則対象外） |
| **`trip doctor`** | **なし**（初手） |
| **`trip advisor`** | **なし** |

### 共通ヘルパ（案）

```rust
// src/summary.rs または trip.rs / day.rs 内
fn normalize_summary(input: Option<&str>) -> Result<Option<String>>
```

trim → 空なら `None` → 長さチェック。CLI / import / update で共用。

---

## 4. CLI plan

### Trip

| 操作 | コマンド案 |
|---|---|
| 作成 | `trip add --name "..." --start ... --end ... [--summary "..."]` |
| 更新 | `trip update <id> [--summary "..."]` |
| クリア | `trip update <id> --clear-summary` |
| 表示 | `trip show <id>` — 概要セクション（NULL なら省略） |
| JSON | `trip show <id> --json` — `"summary": null \| "..."` |

`trip list` — **初手は summary 非表示**（行が長くなるため）。将来 `--verbose` で先頭 1 行のみ検討。

### Day

**`day update` は現行未実装**（[day-model.md § Day コマンド](day-model.md#day-コマンドv120)）— Summary 実装と同フェーズで **新設**。

| 操作 | コマンド案 |
|---|---|
| 更新 | `day update <trip_id> <day_number> --summary "..."` |
| クリア | `day update <trip_id> <day_number> --clear-summary` |
| 表示 | `day show <trip_id> <day_number>` — 概要セクション |
| JSON | `day show <trip_id> <day_number> --json` — summary フィールド追加 |

`day list` — **初手は summary 非表示**。`day list --json` の Day エントリに `summary` を含める（`description` からキー変更する場合は breaking）。

### 表示 UX（テキスト）— [entity design §5](summary-entity-design.md#5-cli-display-policy) と同型

```text
trip show 1

Trip #1  沖縄 瀬底 4日間
  期間: 2026-04-26 〜 2026-04-29

  概要:
    GWちょっと手前で行くことで、飛行機の料金を格安に抑える。

day show 1 2

Day 2  (2026-04-27)

  概要:
    海洋博公園、古宇利島、ハナサキマルシェ

  Itineraries:
    ...
```

### Note との並列

Summary 節は **Notes 節の前**（要約 → 詳細）。

### リリース分割（案）

| フェーズ | 内容 |
|---|---|
| **Phase 1** | migration + Model + `trip add/update/show` + summary 正規化 |
| **Phase 2** | `day update` + `day show` JSON 拡張 |
| **Phase 3** | export/import + validate-export + duplicate roundtrip |
| **Phase 4** | export-md + trip diff |

---

## 5. Export / Import plan

### JSON 形状（v3 拡張候補）

```json
{
  "schema_version": 3,
  "trip": {
    "name": "沖縄 瀬底 4日間",
    "start_date": "2026-04-26",
    "end_date": "2026-04-29",
    "summary": "GWちょっと手前で行くことで、飛行機の料金を格安に抑える。"
  },
  "days": [
    {
      "day_number": 1,
      "summary": "首里城、瀬底島（瀬底大橋）",
      "itineraries": []
    }
  ]
}
```

> **未確定:** Participant / Photo / Reservation 等と合わせ **schema v4** にする選択肢は残す（[summary-entity-design.md §6](summary-entity-design.md#6-export--import-policy)）。実装着手時に再評価。

### v3 拡張 vs v4

| 観点 | v3 拡張 | v4 新設 |
|---|---|---|
| 変更量 | 小 — optional キー追加 | 大 — `schema_version: 4` |
| 互換 | 旧 export import 継続 | v3 import 継続が別途必要 |
| 適合 | Summary のみなら十分 | 複数モデル同時追加時 |

Summary **単独実装**なら **v3 拡張を第一候補**。

### Import 考慮点

| 論点 | 方針 |
|---|---|
| **キー省略** | `summary` なし → NULL |
| **`null`** | NULL |
| **空文字 `""`** | NULL 正規化 |
| **未知キー** | 旧 CLI は無視 — 新フィールドは optional |
| **import 順序** | 既存どおり Trip（summary）→ Day 行作成/更新（summary）→ Itinerary → … |
| **validate-export** | 任意文字列。空文字は warning または error。長さ上限は実装時 |
| **roundtrip test** | `tests/export_roundtrip_cli.rs` 等に summary ケース追加 |
| **duplicate** | export → import で trip/day summary 一致 |
| **v1 / v2 import** | `days[]` なし or summary なし — 問題なし |
| **golden file** | okinawa `expected-export-v3.json` は **初手変更しない**（§10） |

### `trip` オブジェクトと DB `id`

現行どおり export の `trip` に **DB `id` は含めない**。summary のみ追加。

---

## 6. Markdown export plan

対象: **`trip export-md`**（`src/markdown.rs`）

| レベル | 配置 | NULL 時 |
|---|---|---|
| **Trip** | `# 旅行名` 直下、期間の前または直後 | **何も出さない** |
| **Day** | `## Day N — 日付` 直下、Itinerary 列の前 | **何も出さない** |

### 例

```markdown
# 沖縄 瀬底 4日間

家族5人で瀬底島を訪れる4日間の旅行。

**期間:** 2026-04-26 〜 2026-04-29

## Day 1 — 2026-04-26

到着・買い出し・チェックイン。

### 10:30 出発
```

### 方針

| 論点 | 方針 |
|---|---|
| **Markdown 解釈** | **しない** — plain text として出力 |
| **改行** | summary 内 `\n` は **そのまま改行**（段落分割は実装時に 1 方針に統一） |
| **エスケープ** | `#` や `*` が含まれてもユーザー入力をそのまま出す（意図的な plain） |
| **Notes** | Summary の **後**、Itinerary **前** に既存 Note 節 |
| **Remark** | Itinerary 行内 — 変更なし |

---

## 7. Diff plan

`trip diff` で summary を **比較対象に含める**（推奨）。

### 比較対象

| 対象 | キー |
|---|---|
| Trip summary | `trip.summary`（単一フィールド） |
| Day summary | `day_number` ごとの `days[].summary` |

### 差分種別

| 操作 | 表示例 |
|---|---|
| Trip summary 追加 | `+ Trip summary set` |
| Trip summary 削除 | `- Trip summary cleared` |
| Trip summary 変更 | `~ Trip summary changed` |
| Day summary 追加 | `+ Day 2 summary set` |
| Day summary 削除 | `- Day 2 summary cleared` |
| Day summary 変更 | `~ Day 2 summary changed: "旧" → "新"`（長文は truncate 可） |

### 実装メモ

- `diff.rs` の `trip_changes` に Trip フィールド差分を追加（既存 `name` / date 変更と同列）
- Day summary は `HashMap<day_number, Option<String>>` で old/new を比較
- v2 export 側に `days[].summary` なし → 空扱い、**panic しない**

### 優先度

Phase 4（export 安定後）。diff なしでも Summary CRUD は成立するが、**計画共有の変更追跡** のため含める。

---

## 8. Validation plan

[summary-entity-design.md §4](summary-entity-design.md#4-storage-policy) に基づく。

| ルール | 方針 |
|---|---|
| **trim** | 前後空白を除去してから判定 |
| **空文字** | `""` および空白のみ → **NULL** |
| **長さ制限** | Trip: 例 **2000 文字**、Day: 例 **1000 文字**（実装時に定数化） |
| **Markdown** | 解釈・検証 **なし** |
| **改行** | 許可 |

### エラーメッセージ案

```text
summary exceeds maximum length (2000 characters). Use a Note for longer text.
summary is empty after trimming; omit --summary or use --clear-summary
```

### Note への誘導

長さ超過時:

```text
Tip: For detailed travel notes, use `note add --trip <id>` or `note add --trip <id> --day <n>`.
```

---

## 9. Test plan

### Migration

| テスト | 内容 |
|---|---|
| 既存 DB 読取 | migration 前の DB を開き、summary NULL で既存 Trip/Day/Itinerary が読める |
| 新規 DB | `CREATE TABLE` に summary 列あり |
| 列追加 idempotent | migration 二重実行でエラーにならない |

### CLI

| テスト | 内容 |
|---|---|
| `trip add --summary` | 作成直後 `trip show` / `--json` に反映 |
| `trip update --summary` | 更新・上書き |
| `trip update --clear-summary` | NULL に戻る |
| `day update --summary` | 指定 day_number のみ更新 |
| `day update --clear-summary` | Day summary NULL |
| `day show --json` | summary フィールド存在 |
| 空文字 | `--summary ""` → NULL 正規化 |

### Export / Import

| テスト | 内容 |
|---|---|
| export v3 | `trip.summary` / `days[].summary` が出力される |
| summary 省略 | キーなしで serialize（`skip_serializing_if`） |
| import roundtrip | export → import → show で一致 |
| 旧 schema | summary なし JSON が import できる |
| duplicate | summary 含め複製 |
| validate-export | 長さ超過で error/warning |

### Markdown

| テスト | 内容 |
|---|---|
| export-md | Trip/Day summary 節の有無 |
| NULL | summary なしで余計な空行を増やさない |

### Diff

| テスト | 内容 |
|---|---|
| trip summary change | diff 出力に `~ Trip summary changed` |
| day summary add/remove | Day N の追加・削除メッセージ |

### 統合

| テスト | 内容 |
|---|---|
| `make check` | fmt / clippy / 全テスト |
| okinawa golden | **変更なし**（summary 未投入のまま pass） |

---

## 10. Non-goals

本実装計画のスコープ外。

```text
Itinerary.summary
Summary 用独立テーブル
Markdown editor / rich text 入力
Reservation 連携
Participant 連携
Photo / Attachment 連携
trip stats への summary 集計
trip doctor への summary 必須チェック
canonical sample（okinawa_sesoko_2026）の更新実施
Expense.sort_order / Checklist.sort_order の sample 形状見直し
```

canonical sample への Summary 投入は **Entity Design + 本計画確定後** に再検討（[summary-responsibilities-review.md §10](summary-responsibilities-review.md#10-canonical-sample)）。

---

## 11. 実装フェーズ（参考ロードマップ）

| Phase | 内容 |
|---|---|
| **0** | 責務整理 + entity design（v1.14–v1.15 完了） |
| **1** | 本実装計画（v1.16.0） |
| **2** | migration + Trip summary CRUD |
| **3** | `day update` + Day summary |
| **4** | export/import v3 拡張 + tests |
| **5** | export-md + trip diff |
| **6** | 小規模 sample / しおり検証（任意） |

---

## 12. v1.16.0 スコープ（本書）

### 実施する

| 項目 | 内容 |
|---|---|
| 仕様書 | 本ドキュメント |
| 索引 | [specifications/README.md](README.md) |
| 参照 | summary-entity-design、travel-ledger-responsibilities 等 |

### 実施しない

```text
DB migration
summary カラム追加
CLI コマンド追加
Repository 実装
export / import schema 変更
Markdown export 変更
テスト実装
canonical sample 更新
```

---

## 13. 用語

| 用語 | 意味 |
|---|---|
| **Summary** | Trip / Day の短い共有向け要約 |
| **Remark** | `itinerary_items.note` |
| **Note** | Long-form entity |
| **normalize** | trim + 空 → NULL |

---

## 14. 実装参照（現行）

| 概念 | パス / 状態 |
|---|---|
| Trip | `src/models.rs` — summary **なし** |
| Day | `src/models.rs` — `description` あり、export 未使用 |
| Trip CRUD | `src/trip.rs` |
| Day 読取 | `src/day.rs` — `day update` **なし** |
| Export v3 | `src/trip.rs` — `build_trip_export_v3` |
| Markdown | `src/markdown.rs` |
| Diff | `src/diff.rs` — summary **未対応** |
| Migration helper | `src/db.rs` — `add_column_if_not_exists` |
| 責務 / 設計 | [summary-responsibilities-review.md](summary-responsibilities-review.md)、[summary-entity-design.md](summary-entity-design.md) |
