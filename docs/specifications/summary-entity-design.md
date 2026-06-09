# Summary Entity Design（設計草案）

Caglla.Travel における **Summary エンティティ** の将来表現 — フィールド、格納方針、表示・export の設計メモです。

**v1.15.0 時点: 仕様整理のみ。** DB migration、CLI、export schema、Markdown export の変更は行いません。

| ドキュメント | 役割 |
|---|---|
| [travel-ledger-responsibilities.md](travel-ledger-responsibilities.md) (v1.10.0) | Summary / Remark / Note / Reservation の横断比較 |
| [summary-responsibilities-review.md](summary-responsibilities-review.md) (v1.14.0) | 責務・境界（What it is / is not） |
| **本書** (v1.15.0) | フィールド・格納・表示・export 方針（How we might model it） |
| [summary-implementation-plan.md](summary-implementation-plan.md) (v1.16.0) | 実装計画（If we build it, how） |

関連: [Note モデル](note-model.md) / [Day モデル](day-model.md) / [Itinerary モデル](itinerary-model.md) / [Export Schema](export-schema.md) / [Travel Ledger Responsibilities](travel-ledger-responsibilities.md) / [Summary Implementation Plan](summary-implementation-plan.md)

---

## 1. Purpose

Summary の目的 — **読者向けの短い要約** を Trip / Day に付与する。

| 目的 | 説明 |
|---|---|
| **旅行全体の概要説明** | Trip レベルで「この旅行は何か」「誰と・どんな狙いで」を同行者に伝える |
| **その日のテーマや流れの説明** | Day レベルで詳細旅程を読む前に「今日はどんな日か」を把握 |
| **一覧表示や共有時の短い説明** | Trip 一覧・export・しおりで名前以外の文脈を付与 |
| **しおりのスキャン入口** | 印刷・Markdown で冒頭・Day 見出し直下に載せ、全体を読まなくても orientation できる |

### Summary は Note の代替ではない

| | **Summary** | **Note entity** |
|---|---|---|
| 性質 | 短い **要約** | 長文 **詳細記録** |
| 件数 | Trip/Day 各 **0..1** | 各 owner **0..N** |
| 読者 | 共有・しおり・スキャン | 本人・同行者の詳細閲覧 |
| 典型用途 | 「沖縄4日間、家族5人で瀬底島」 | 航空券比較の経緯、一日の振り返り長文 |

Note に要約を書く運用は移行期として許容できるが、**共有向け一行説明の正本** は将来 Summary フィールドとする（[summary-responsibilities-review.md §5](summary-responsibilities-review.md#5-summary-と-note)）。

---

## 2. Responsibility

Travel Ledger における説明系概念の責務境界。

```text
Summary  = 要約      （Trip / Day — 読者向け、短い）
Remark   = 行内補足  （Itinerary — 旅程表の備考欄）
Note     = 詳細メモ  （Trip / Day / Itinerary — 自由記述、長文可、複数件）
```

### 比較表

| 概念 | 日本語 | スコープ | 長さ | 件数 | 実装 |
|---|---|---|---|---|---|
| **Summary** | 概要 | Trip / Day | 短い（1〜5 文程度） | 0..1 | **未実装** |
| **Remark** | 備考 | Itinerary | より短い（一行多い） | 0..1 | **既存** `itinerary_items.note` |
| **Note** | メモ | Trip / Day / Itinerary | 長文可 | 0..N | **CRUD 済み** |

### 責務の違い（一文）

| 概念 | 役割 |
|---|---|
| **Summary** | 「この旅行／この日は **どんな日か**」を **先に** 伝える |
| **Remark** | 「この行動について **一言補足**」— 予約番号・ETC・集合場所など実務向き |
| **Note** | 「**詳しく書き残す**」— 検討・経緯・振り返り・背景 |

### 重複を避ける原則

```text
title（Itinerary）  → 行動の一行ラベル（旅程表の行見出し）
summary（Trip/Day） → 共有向け要約（しおり・一覧）
note（Remark）      → 行内短文補足
Note entity         → 長文詳細
```

Itinerary レベルに `summary` を追加すると `title` / Remark / Note と **四重化** しやすい。初手では Trip / Day のみ（§3）。

---

## 3. Entity Scope

### 検討対象

```text
Trip.summary    — 旅行全体の概要（trips テーブル列）
Day.summary     — その日のテーマ・主な行先（days テーブル列）
```

各 **0 or 1** の単一テキスト。別テーブル・別 entity 行は不要 — 親 Trip / Day に **埋め込み列** とする。

### 検討対象外

```text
Itinerary.summary   — 初手では採用しない
```

### Itinerary.summary が不要な理由

| 理由 | 説明 |
|---|---|
| **`title` が既に行動要約** | `チェックイン` / `NU045 NGO ⇒ OKA` / `美ら海水族館` は旅程表上の要約として機能している |
| **Remark で足りる** | 行内の実務補足（ETC、チェックアウト時刻、予約番号の暫定置き場）は `itinerary_items.note` |
| **Note で足りる** | 行動単位の説明文・詳細は Itinerary Note 0..N 件で表現可能 |
| **しおり構成** | 旅程表は `start_time` + `title` + Remark で成立。別 Summary 列は冗長 |
| **責務の明確化** | v1.14.0 で確定 — [summary-responsibilities-review.md §8](summary-responsibilities-review.md#8-itinerary-summary-は必要か) |

将来 GUI で「タイトルと説明を分けたい」需要が強い場合は、専用列より **Itinerary Note（短文 body）** を先に検討する。

### エンティティ構造（案）

```text
Trip
 ├─ name          （既存 — 旅行名）
 ├─ summary       （将来 — 概要）
 └─ Day
      ├─ day_number （既存）
      ├─ summary    （将来 — 日概要）
      └─ Itinerary
           ├─ title   （既存 — 行動ラベル）
           └─ note    （既存 Remark 列）
```

Summary は **独立テーブルを作らない**。Trip / Day の属性として保持する。

---

## 4. Storage Policy

将来実装時の DB 格納方針案。**本段階では確定しない。**

### フィールド定義（案）

| フィールド | 親 | 型（案） | 説明 |
|---|---|---|---|
| `summary` | `trips` | TEXT | 旅行全体の概要。NULL 可 |
| `summary` | `days` | TEXT | その日の概要。NULL 可 |

`description` という列名は **採用しない**（`summary` に用語統一 — [summary-responsibilities-review.md §6](summary-responsibilities-review.md#6-trip-summary)）。GUI ラベルは「旅行の説明」「この日の概要」等でよい。

### NULL 許可

| 方針 | 説明 |
|---|---|
| **NULL 許可** | Summary 未入力は **NULL**（0 件と同義） |
| 既存 Trip / Day | migration 後も NULL のまま — backfill 不要 |

### 空文字扱い

| 入力 | 格納（案） |
|---|---|
| CLI で未指定 | NULL のまま |
| 空文字 `""` | **NULL に正規化**（Remark / Note と同型の扱いを推奨） |
| 空白のみ | 実装時に trim して空なら NULL — または拒否 |

### 長さ制限

| レベル | 案 |
|---|---|
| **Trip.summary** | おおむね **500〜2000 文字** 上限（実装時に確定）。要約なので短い運用を想定 |
| **Day.summary** | おおむね **200〜1000 文字** 上限。1〜3 文・一行リストが典型 |

DB 型は SQLite `TEXT`（上限はアプリ層バリデーション）。過度に長い入力は Note へ誘導するエラーメッセージを検討。

### Markdown 可否

| 方針 | 案 |
|---|---|
| **初手** | **プレーンテキスト** — Markdown 記法は解釈しない |
| **改行** | 許可（複数行・箇条書き風の手入力は可） |
| **将来** | `export-md` 出力時にそのまま埋め込むか、軽い整形のみ — **入力時点では Markdown モードなし** |

理由: Summary はしおり・一覧向け。**装飾より可読性と単純さ** を優先。リッチ表現は Note の `body`（将来 Markdown 対応）に任せる。

### タイムスタンプ

Summary 専用の `summary_updated_at` は **初手不要**。親 `trips.updated_at` / `days` に相当する更新時刻があれば十分（実装時に Trip / Day の既存列を確認）。

---

## 5. CLI Display Policy

将来 CLI で Summary を表示・編集する場合の方針。**コマンド名・フラグは実装計画フェーズで確定。**

### 表示箇所

| 入口 | Summary の扱い |
|---|---|
| **`trip show <trip_id>`** | Trip 名・期間の直下に **概要** セクション。NULL なら省略または「（概要なし）」 |
| **`day show <trip_id> <day_number>`** | Day 見出し直下に **この日の概要**。NULL なら省略 |
| **`trip list`** | 初手は **表示しない**（行が長くなる）。将来 `--verbose` で 1 行目のみ |
| **`day list`** | 同上 — 初手は省略可 |
| **`trip export-md`** | §7 — Markdown しおりに出力 |
| **`trip export` / import** | §6 — JSON に含める |

### 編集箇所（将来案）

| 操作 | 方針案 |
|---|---|
| Trip Summary 更新 | `trip update <id> --summary "..."` または `trip update <id> --clear-summary` |
| Day Summary 更新 | `day update <trip_id> <day_number> --summary "..."`（`day update` は現行未実装 — 実装計画で追加） |
| Itinerary | Summary フィールド **なし** — `itinerary update --note` は Remark のまま |

### 表示 UX 例（テキスト）

**`trip show 1`**

```text
Trip #1  沖縄 瀬底 4日間
  期間: 2026-04-26 〜 2026-04-29

  概要:
    GWちょっと手前で行くことで、飛行機の料金を格安に抑える。
    夏前の過ごしやすい沖縄４日間。

  Days: 4
  Itineraries: 58
  ...
```

**`day show 1 2`**

```text
Day 2  (2026-04-27)

  概要:
    海洋博公園（美ら海水族館、ドリームセンター）、古宇利島、沖縄ハナサキマルシェ

  Itineraries:
    09:00  美ら海水族館
    ...
```

### `--json`

`trip show` / `day show` の JSON に `summary: null | string` を追加（実装時）。Note / Expense と同型のオプション出力。

### Note との並列表示

```text
Trip show:
  概要  ← Summary
  Notes ← Note entity 一覧（既存）

Day show:
  概要  ← Summary
  Notes ← Day Notes
```

Summary 節は **Notes 節の前** に置く（要約 → 詳細の読み順）。

---

## 6. Export / Import Policy

Export Schema **v3 延長** を想定した場合の考慮点。**schema v4 化を前提としない** が、v4 へ移す場合も同フィールドを `trip` / `days[]` に載せやすい形状とする。

> **v1.15.0 時点:** export 方針は **候補** であり確定事項ではない。Participant / Photo 等と合わせて実装フェーズで v3 拡張か v4 かを再評価する。

### JSON 形状（v3 拡張候補）

```json
{
  "schema_version": 3,
  "trip": {
    "name": "沖縄 瀬底 4日間",
    "start_date": "2026-04-26",
    "end_date": "2026-04-29",
    "summary": "GWちょっと手前で行くことで、飛行機の料金を格安に抑える。夏前の過ごしやすい沖縄４日間。"
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

### 考慮点

| 論点 | 方針案 |
|---|---|
| **キー名** | `summary`（`description` は使わない） |
| **省略時** | キーなしまたは `null` — import 時は DB NULL |
| **後方互換** | 既存 v3 export に `summary` なし → import 成功、NULL 扱い |
| **前方互換** | 旧 CLI が未知キーを無視できるか — import 側は未知フィールドを無視する現行方針を維持 |
| **内部 ID** | summary は trip / day に埋め込み — 別 ID・別配列不要 |
| **import 順序** | Trip（summary 含む）→ Day（summary 含む）→ 既存フロー |
| **validate-export** | 任意文字列。長さ上限は実装時。空文字は error または warning |
| **roundtrip** | `trip duplicate` 経由で summary も複製 |
| **trip diff** | `trip.summary` / per-day `summary` の追加・変更・削除を比較（将来） |

### v1 / v2 import

`trip` / `days` に `summary` キーなし — 問題なし。

### top-level `summaries[]` は採用しない

Note の `notes[]` とは異なり、Summary は **親オブジェクトの属性** のみ。フラット配列にしない。

---

## 7. Markdown Export Policy

対象: **`trip export-md`**

### 配置

| レベル | 位置 |
|---|---|
| **Trip** | `# 旅行名` 直下、期間メタデータの前または直後 |
| **Day** | `## Day N — 日付` 直下、Itinerary 列の前 |
| **Itinerary** | Summary 節なし — `title` + Remark のまま |

### 利用イメージ

```markdown
# 沖縄 瀬底 4日間

家族5人で瀬底島を訪れる4日間の旅行。GW手前の日程で航空券を抑えた沖縄滞在。

**期間:** 2026-04-26 〜 2026-04-29

## Day 1 — 2026-04-26

到着・買い出し・チェックイン。首里城と瀬底大橋経由で瀬底島へ。

### 10:30 出発
...

## Day 2 — 2026-04-27

海洋博公園（美ら海水族館）、古宇利島、ハナサキマルシェ。

### 09:00 美ら海水族館
- **Location:** 沖縄県国頭郡本部町
...
```

### 方針

| 論点 | 方針 |
|---|---|
| Summary なし | 現行どおり — Trip はタイトルのみ、Day は見出しのみ |
| 改行 | summary 内の改行は Markdown で **段落またはそのまま改行**（実装時に統一） |
| Markdown 入力 | ユーザーが `**太字**` を書いても初手は **エスケープまたはそのまま出力** — 解釈はしない |
| Trip / Day Notes | Summary 節の **後**、Itinerary 列の **前** に既存 Note 節 |
| Remark | Itinerary 行内 — 変更なし |
| Reservation | 将来別節 — Summary に予約番号を書かない |

### 旅行前しおりとの整合

計画共有 PDF にあった「旅行の狙い」「主な行先」は、Trip Summary / Day Summary の **実例テキスト** として参照できる（[summary-responsibilities-review.md §10](summary-responsibilities-review.md#10-canonical-sample)）。

---

## 8. 将来フック（実装フェーズ — 本書では未実装）

| 領域 | 方針案 |
|---|---|
| **DB** | `trips.summary` / `days.summary` 列追加 |
| **CLI** | `trip update --summary` / `day update --summary`（day update は新規） |
| **Export** | v3 拡張または v4 |
| **export-md** | §7 の配置 |
| **doctor** | 原則対象外。任意: Trip 名のみで summary も空 → Info |
| **canonical sample** | Entity Design + Implementation Plan 確定後に再検討 |

詳細: **[Summary Implementation Plan](summary-implementation-plan.md)**（v1.16.0）。

---

## 9. v1.15.0 スコープ（本書）

### 実施する

| 項目 | 内容 |
|---|---|
| 仕様書 | 本ドキュメント |
| 索引 | [specifications/README.md](README.md) |
| 参照 | summary-responsibilities-review、travel-ledger-responsibilities 等 |

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

## 10. 用語

| 用語 | 意味 |
|---|---|
| **Summary** | Trip / Day の短い共有向け要約（`summary` 列） |
| **Remark** | Itinerary 行の備考（`itinerary_items.note`） |
| **Note** | Long-form entity（`title` + `body`） |
| **Travel Ledger** | 行動台帳 — Sequence-first Itinerary + 紐づく補助情報 |

---

## 11. 実装参照（現行）

| 概念 | 状態 |
|---|---|
| Trip.summary | **未実装** |
| Day.summary | **未実装** |
| 責務整理 | [summary-responsibilities-review.md](summary-responsibilities-review.md) |
| フィールド設計 | **本書** |
| Remark | `itinerary_items.note` — `src/itinerary.rs` |
| Note entity | `src/note.rs` |
| Markdown | `src/markdown.rs` — Summary 節なし |
| Export | [export-schema.md](export-schema.md) — `summary` なし |
| 実装計画 | [summary-implementation-plan.md](summary-implementation-plan.md) |
