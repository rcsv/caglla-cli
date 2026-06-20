# Estimate Entity Design

Caglla.Travel CLI / 将来 Web 版に向けた **Estimate（事前見積 / Planned Budget）** の具体設計です。

**設計フェーズ 2/N: Entity Design のみ。** 本書は DB migration・CLI・export schema の **実装を伴わない**。実装手順は `estimate-implementation-plan.md`（未着手）で確定する。

| ドキュメント | 役割 |
|---|---|
| [estimate-model.md](estimate-model.md) | 責務・境界（What it is / is not） |
| **本書** | テーブル・フィールド・CLI 案・検証・将来 export（How we model it） |
| `estimate-implementation-plan.md`（将来） | 実装計画（How to build） |
| [expense-model.md](expense-model.md) (v1.5.0) | Expense = Actual Money（amount / currency の参照実装） |
| [expense-post-implementation-review.md](expense-post-implementation-review.md) (v1.22.0) | Expense ≠ Estimate の既存結論 |
| [itinerary-model.md](itinerary-model.md) (v1.8.0+) | Itinerary = 行動単位。Estimate の親 |
| [export-schema.md](export-schema.md) | trip export / import JSON（現行 v5） |

関連: [planning-design-principles.md](planning-design-principles.md) / [ordering-model.md](ordering-model.md) / [long-term-version-strategy.md](../long-term-version-strategy.md)

設計系列（想定）:

```text
Responsibilities Review   → estimate-model.md
Entity Design             → estimate-entity-design.md（本書）
Implementation Plan       → estimate-implementation-plan.md（未着手）
Implementation            → DB + CLI + export（未着手）
Post-Implementation Review → （未着手）
```

---

## Purpose

[estimate-model.md](estimate-model.md) で確定した責務を前提に、Estimate の **保存形・CLI 案・検証・他機能への将来影響** を具体化する。

```text
後続の Implementation Plan / 実装が迷わないよう、
DDL 骨格・フィールド責務・cascade・export フィールド案・validation を固める。
```

本書の第一ゴール:

```text
この Itinerary で見込む合計金額はいくらか
```

に **構造化データ** で答えられること。会計システム化（按分・税・単価×数量）は **初期スコープ外**。

---

## Source Responsibilities Review

[estimate-model.md](estimate-model.md) から本設計が引き継ぐ **確定前提**:

| 項目 | 前提 |
|---|---|
| **意味論** | Estimate = **Planned Money**。Expense = **Actual Money** — 別エンティティ |
| **親** | **Itinerary のみ**（Trip / Day 直下は初期対象外） |
| **1 Itinerary : N Estimate** | 許容（Expense と同型） |
| **payer / beneficiary** | 初期 **非対象** |
| **unit_amount × quantity** | 初期 **非対象** — 1 行 = 1 見積総額 |
| **Planned vs Actual 差分** | DB 列は持たない — 集計レイヤーで導出 |
| **replicate** | 将来 **コピー候補**（本フェーズでは実装しない） |
| **為替換算** | Expense と同様 v1 系 **非対象** |

本書は上記を **破らない** 範囲で DDL・CLI・export 案・検証を確定する。

---

## Entity Definition

### エンティティ関係（初期実装）

```text
Trip
 └─ Day
      └─ Itinerary
           ├─ Estimate[]     ← 本書（Planned Money）
           ├─ Expense[]
           ├─ Reservation[]
           └─ Note[]
```

| 階層 | Estimate |
|---|---|
| Trip 直下 | **なし** |
| Day 直下 | **なし** |
| Itinerary 配下 | **あり** — `estimates.itinerary_id` |

### テーブル名・CLI 接頭辞（確定）

| 層 | 名前 |
|---|---|
| DB テーブル | `estimates` |
| CLI | `estimate add` / `list` / `show` / `update` / `delete` |

---

## 1. `estimates` テーブル

### DDL 案（migration 参考 — 本フェーズでは実装しない）

```sql
CREATE TABLE estimates (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    itinerary_id    INTEGER NOT NULL,
    title           TEXT,
    amount          INTEGER NOT NULL,
    currency        TEXT NOT NULL,
    note            TEXT,
    sort_order      INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_estimates_itinerary
    ON estimates(itinerary_id);
```

### フィールド責務

| 列 | 必須 | 型 | 説明 |
|---|---|---|---|
| `id` | — | INTEGER | 主キー（AUTOINCREMENT） |
| `itinerary_id` | ✓ | INTEGER | 親 Itinerary。存在をアプリ側で検証 |
| `title` | — | TEXT NULL | 見積項目名（例: `ホテル朝食`）。**省略可**（§2） |
| `amount` | ✓ | INTEGER | **最小通貨単位**の整数（§3） |
| `currency` | ✓ | TEXT | ISO 4217 コード（例: `JPY`, `USD`）。大文字正規化（§3） |
| `note` | — | TEXT NULL | 任意メモ（例: `5人分`） |
| `sort_order` | ✓ | INTEGER | 同一 Itinerary 内の表示順（§4）。デフォルト `0` |
| `created_at` | ✓ | TEXT | ISO 8601 UTC（既存エンティティ同型） |
| `updated_at` | ✓ | TEXT | ISO 8601 UTC |

### 初期実装に **含めない** 列

以下は **v1 Estimate では導入しない**:

```text
unit_amount, quantity, payer, beneficiaries, participant_id,
tax, service_charge, planned_vs_actual_delta, reservation_id
```

理由: 最初から細かくしすぎると Estimate が会計システム化する。[estimate-model.md §Non-goals](estimate-model.md#non-goals) を維持。

---

## 2. `title` の扱い（案 A vs 案 B）

### 比較

| 観点 | 案 A: `title` **任意** | 案 B: `title` **必須** |
|---|---|---|
| Expense との一貫性 | **一致** — Expense も title 任意 | 不一致 |
| 最小入力 | `--amount` + `--currency` のみで add 可 | 項目名を毎回考える必要 |
| 複数 Estimate | title があると分かりやすいが、**必須でなくても note / amount で区別可** | 常にラベルあり |
| CLI 表示 | title 未設定時は `-` または `(Estimate)` | 常に項目名表示 |
| DB | `TEXT NULL` | `TEXT NOT NULL` + 空文字拒否 |

### **確定: 案 A — `title` は任意**

Expense と同型とし、**必須は `amount` + `currency` のみ** とする。

| 理由 | 説明 |
|---|---|
| 入力容易性 | 旅行前のざっくり見積は「14,000 円くらい」だけ先に入れたい |
| 既存慣習 | [expense-model.md §3](expense-model.md) — title / note NULL 許可 |
| 表示 | `list` / `show` で title 未設定時は `-` または `(Estimate)` を表示すれば十分 |

1 Itinerary に複数 Estimate を置く場合（`チケット` / `昼食` / `駐車場`）は **title 推奨** だが、CLI は **強制しない**。Implementation Plan で表示フォーマットを詳細化する。

---

## 3. `amount` / `currency`

Expense 実装（[expense-model.md §3](expense-model.md) / `src/expense.rs`）と **同一方針** を採用する。

### amount（DB）

| 通貨 | 例 | DB `amount` |
|---|---|---|
| JPY | 14,000 円 | `14000` |
| USD | 12.50 ドル | `1250`（セント） |

- DB 列は **常に最小通貨単位の INTEGER**
- 負の amount は **v1 系では拒否**（Expense と同型）

### amount（CLI 入力）

| 入力 | currency | 保存値 |
|---|---|---|
| `--amount 14000` | `JPY` | `14000` |
| `--amount 12.50` | `USD` | `1250` |
| `--amount 12.5` | `USD` | `1250` |

- `--amount` は **文字列** として受け取り、通貨ごとの小数桁（JPY=0、USD=2 等）に基づき整数へ変換
- 変換ロジックは Expense の `parse_amount_for_currency(input, currency)` を **共用** する（Implementation Plan で `expense.rs` から共通化モジュールへ切り出しを検討）

### currency

- `validate_currency_code(code)` を **Expense と共用**
- 大文字 3 文字（`JPY`, `USD`）を基本。小文字入力は正規化
- 未知コード（`XXX` 等）は Expense と同様 **許容**（将来 doctor / validate-export で warning 可）

### 表示（CLI / export-md 将来）

- 人間向け表示は通貨に応じた小数桁でフォーマット（Expense `list` / `show` と同型）
- export JSON の `amount` は **DB 整数そのまま**（Expense export と同型）

---

## 4. `sort_order`

### 並び順

同一 Itinerary 内:

```text
ORDER BY sort_order ASC, id ASC
```

[ordering-model.md](ordering-model.md) の sequence-first 原則に従う。Estimate は Itinerary **配下** の子エンティティであり、Day 内 Itinerary の `sort_order` とは **別軸**。

### 初期実装の方針

| 論点 | 方針 |
|---|---|
| sparse ordering（1000 刻み） | **初期は不要** — Expense と同様の素朴な `sort_order` |
| 未指定時 | **`0`**（Expense / Note と同型） |
| `--sort-order` on add | **任意** — 指定がなければ `0` |
| reorder 専用 CLI | **初期なし** — `update --sort-order` のみ |
| tie-break | 同一 `sort_order` は **`id ASC`** |

Itinerary 本体の sparse ordering（1000 刻み）は [ordering-model.md](ordering-model.md) の Day 内操作向け。Estimate は件数が少ない想定のため、**Expense と同じ単純モデル** で開始する。必要になれば Implementation Plan 以降で `estimate normalize` 等を検討。

---

## 5. 外部キー / cascade 方針

[expense-model.md §6](expense-model.md) / [note-model.md](note-model.md) と同型: **SQLite FK 制約なし + アプリ側 cascade**。

| トリガー | `estimates` |
|---|---|
| `estimate delete` | 当該 Estimate **のみ** 削除。親 Itinerary は **削除しない** |
| `itinerary delete` | 当該 `itinerary_id` の Estimate を **すべて削除** |
| `trip delete` | Trip 配下 Itinerary 経由で Estimate を **すべて削除** |
| `day delete`（Itinerary 連鎖） | Itinerary 削除に伴い Estimate も削除 |
| `trip update`（期間短縮で Day 削除） | Itinerary 削除に伴い Estimate も削除 |

実装イメージ:

```text
delete_estimates_for_itinerary(itinerary_id)
delete_estimates_for_trip(trip_id)
```

を `itinerary delete` / `trip delete` から **同一トランザクション内** で呼び出す（Expense / Reservation / Note cascade と同型）。

create 時（`estimate add`）: `itinerary_id` が存在することを検証（Expense `add` と同型）。`update` は Estimate ID 指定の部分更新のため親 Itinerary は変更しない。

---

## 6. CLI 設計案（未実装）

Expense CLI と **並列** に配置する。以下は Entity Design 上の確定案。

### コマンド一覧

```bash
caglla estimate add --itinerary 12 --amount 14000 --currency JPY
caglla estimate add --itinerary 12 --amount 14000 --currency JPY --title "ホテル朝食"
caglla estimate list --itinerary 12
caglla estimate list --trip 1
caglla estimate show 3
caglla estimate update 3 --amount 15000
caglla estimate update 3 --title "ホテル朝食 revised" --note "5人分"
caglla estimate delete 3
```

### `estimate add`

| オプション | 必須 | 説明 |
|---|---|---|
| `--itinerary` | ✓ | 親 Itinerary ID |
| `--amount` | ✓ | 見積金額（小数可 — §3） |
| `--currency` | ✓ | 通貨コード |
| `--title` | — | 見積項目名 |
| `--note` | — | 補足 |
| `--sort-order` | — | 省略時 `0` |

Trip / Day 直下への add は **不可**。

### `estimate list`

| オプション | 必須 | 説明 |
|---|---|---|
| `--itinerary` | いずれか 1 つ | 当該 Itinerary 配下のみ |
| `--trip` | いずれか 1 つ | Trip 配下 **すべて** の Estimate を集約表示 |

- `--itinerary` と `--trip` は **排他**（両方・両方なしはエラー）
- Trip 指定時: Day → Itinerary 経由で収集。表示順は **Day 順 → Itinerary sort_order → Estimate sort_order → id**（Expense `list --trip` と同型）
- `--json` は `list` / `show` で対応（Expense / Note と同型 — Implementation Plan で詳細化）

### `estimate show`

- **Estimate ID** で 1 件表示
- 親 Itinerary / Day / Trip のコンテキストを人間向け表示に含める（Expense `show` と同型）

### `estimate update`

- **Estimate ID** で指定
- 指定されたフィールド **のみ** 更新（部分更新）
- **更新項目が 0 件** → **エラー**
- 更新可能: `--title`, `--note`, `--amount`, `--currency`, `--sort-order`
- `--clear-title` / `--clear-note` の要否は Implementation Plan で決定（Expense に `--clear-*` がある場合は揃える）

### `estimate delete`

- **Estimate ID** で 1 件削除
- 親 Itinerary は **削除しない**

### owner 指定（確定）

| 操作 | owner 指定 |
|---|---|
| `add` | **`--itinerary` 必須** |
| `update` / `delete` | **Estimate ID** |
| `list` | **`--itinerary` または `--trip` のいずれか 1 つ必須** |

---

## 7. Validation

### create（`estimate add`）

| 項目 | ルール |
|---|---|
| `itinerary_id` | 存在する Itinerary を参照すること |
| `amount` | **必須**。`parse_amount_for_currency` 経由。負数拒否 |
| `currency` | **必須**。`validate_currency_code` 経由 |
| `title` | 任意。空文字のみは **NULL として保存**（Expense と同型） |
| `note` | 任意。空文字 → NULL |
| `sort_order` | 省略時 `0` |

### update（`estimate update`）

| 項目 | ルール |
|---|---|
| 更新フィールド | **1 件以上必須** — 0 件はエラー |
| `amount` | 指定時のみ `parse_amount_for_currency`（`currency` 未変更時は既存 currency を使用） |
| `currency` | 指定時のみ `validate_currency_code` |
| `title` / `note` | 任意。クリア semantics は Implementation Plan |

### delete

- 存在しない ID → エラー
- cascade なし（Estimate 単体のみ）

### import / validate-export（将来）

- nested `estimates[]` の `amount` / `currency` 必須
- `currency` 形式検証（Expense と同型）
- `title` / `note` 省略可（null 許容）

---

## 8. Export / Import（将来）

**本フェーズでは export schema を変更しない。** 以下は Implementation Plan / export 実装時の確定案。

### ネスト位置

Expense / Reservation と同型 — **`days[].itineraries[].estimates[]`** のみ。

```json
{
  "schema_version": 6,
  "trip": {},
  "days": [
    {
      "day_number": 2,
      "itineraries": [
        {
          "title": "ホテルで朝食",
          "sort_order": 1000,
          "estimates": [
            {
              "title": "ホテル朝食",
              "amount": 14000,
              "currency": "JPY",
              "note": "5人分",
              "sort_order": 0
            }
          ],
          "expenses": []
        }
      ]
    }
  ]
}
```

top-level `estimates[]` は **採用しない**（Expense と同型の親子構造維持）。

### Estimate オブジェクト（export 案）

| フィールド | 必須 | 説明 |
|---|---|---|
| `title` | 任意 | 省略 / null 可 |
| `amount` | ✓ | 最小通貨単位 INTEGER |
| `currency` | ✓ | 大文字 3 文字 |
| `note` | 任意 | |
| `sort_order` | ✓ | 省略時 import は `0` |

export 時 **`id` / `created_at` / `updated_at` は出力しない**（Expense export と同型 — 再 import で新 ID 採番）。

### schema version bump

**Estimate を export に含める段階では `schema_version` の bump が必要 — 第一候補は `6`。**

| 理由 | 説明 |
|---|---|
| 新配列 | `itineraries[].estimates[]` |
| v5 互換 | v5 importer は未知フィールド `estimates` を **読まない** — v5 export 生成は Estimate 実装まで継続 |
| 明確な境界 | [shared-expense-entity-design.md §4](shared-expense-entity-design.md) の v4→v5 と同型の判断 |

| 方向 | 方針 |
|---|---|
| **v5 export → v6 import** | **可** — `estimates` 省略 = 空配列 |
| **v6 export → v5 import** | **不可** — 想定どおり |
| **v6 import** | v5 import 能力を **包含** + `estimates[]` 処理 |

### import 時の Itinerary 解決

Expense / Note と同型 — **`day_number` + itinerary `sort_order`**（または `itinerary_key`）で親 Itinerary を解決し、配下に Estimate を INSERT。

### trip diff（将来）

`trip diff` に Estimate 比較を追加する場合:

```text
added / removed / amount / currency / title / note / sort_order changed
```

**両 export が schema v6+ の場合のみ** Estimate を比較（Expense v5+ ルールと同型）。

---

## 9. Markdown export / trip stats（将来影響）

**本フェーズでは実装しない。** 影響範囲の整理のみ。

### `trip stats` 拡張案

| 表示 | データ源 |
|---|---|
| **Planned total** | Trip 配下 Estimate 合計（通貨別） |
| **Actual total** | 現行 Expense 合計 |
| **Difference** | 表示レイヤーで導出（DB 列なし） |

Itinerary 単位の Planned subtotal も将来表示可能（Estimate 行の合算）。

### `trip export-md` 拡張案

| 表示 | 内容 |
|---|---|
| Itinerary セクション | 配下 Estimate 一覧（予定費用） |
| Trip 末尾 | Planned 合計 / Actual 合計 / 差分（旅行前は Actual `-`） |

[estimate-model.md §Aggregation vision](estimate-model.md#aggregation-vision将来) の GUI 想定と整合。

---

## 10. `itinerary replicate`（将来）

現行 [itinerary-model.md §14](itinerary-model.md#14-itinerary-の複製itinerary-replicate):

| コピーする（現行） | コピーしない |
|---|---|
| Itinerary 本体、Itinerary-level notes | Expense、Reservation |

**Estimate 実装後の方針（Implementation Plan で replicate 変更）:**

| replicate でコピー | replicate でコピーしない |
|---|---|
| Itinerary 本体 | Expense（実績） |
| Itinerary-level notes | Reservation（予約番号等） |
| **Estimate / Planned Budget** | |

Estimate は **予定の一部** なので定型パターン複製時に一緒に持ち運ぶ。Expense は実績、Reservation は予約情報のため **コピーしない**（[estimate-model.md](estimate-model.md) 維持）。

replicate 実装時の詳細（新 Estimate の ID 採番、`sort_order` 維持、トランザクション境界）は Implementation Plan で記載。

---

## 11. amount ロジックの共通化（Implementation Plan 向けメモ）

現行 Expense 実装（`src/expense.rs`）:

```text
validate_currency_code(code) -> Result<String>
parse_amount_for_currency(input, currency) -> Result<i64>
```

Estimate 実装時は **同一関数を共用** する方針。

| 案 | 説明 |
|---|---|
| **A（推奨）** | `src/money.rs` 等へ切り出し、`expense.rs` / `estimate.rs` から import |
| B | `expense.rs` の pub(crate) 関数を estimate から直接呼ぶ（短期） |

Entity Design 段階では **方針のみ確定**。リファクタのタイミングは Implementation Plan で決定。

---

## 12. Open questions（Implementation Plan で確定）

| # | 論点 | 本書の暫定 |
|---|---|---|
| 1 | `update --clear-title` / `--clear-note` | [Implementation Plan §1.5](estimate-implementation-plan.md#15-update--clear-semantics確定) — **Phase 1 で採用** |
| 2 | `estimate list --trip` の Day ヘッダ表示 | Expense list と **同型** |
| 3 | doctor / validate-export の Estimate 検査 | 初版は **任意** — amount/currency/itinerary 整合 |
| 4 | export schema v6 の正確なフィールド名 | `estimates[]` で確定。alias なし |
| 5 | replicate 時の Estimate コピー default | **コピーする**（`--without-estimates` は初期不要） |
| 6 | Trip 全体予算上限（Budget エンティティ） | Estimate 合計とは **別概念** — defer |

---

## Deferred scope summary

本 Entity Design **およびユーザー指定の今回 PR** では以下を **行わない**:

```text
- estimates テーブル / migration
- estimate CLI 実装
- export schema v6 実装
- export-md / trip stats 変更
- itinerary replicate の Estimate コピー
- amount 共通化リファクタ（expense.rs 切り出し）
- release 作業
```

次ステップ: **Implementation Plan**（`estimate-implementation-plan.md`）→ Phase 1 実装。

---

## References

| 用途 | パス |
|---|---|
| 責務整理 | [estimate-model.md](estimate-model.md) |
| Expense amount / currency | [expense-model.md §3](expense-model.md) |
| Itinerary 親子 | [itinerary-model.md](itinerary-model.md) |
| replicate 現行 | [itinerary-model.md §14](itinerary-model.md#14-itinerary-の複製itinerary-replicate) |
| Export 現行 v5 | [export-schema.md](export-schema.md) |
| Ordering | [ordering-model.md](ordering-model.md) |
| 入力過多回避 | [planning-design-principles.md](planning-design-principles.md) |
