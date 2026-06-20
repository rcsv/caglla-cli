# Ordering モデル

本ドキュメントは、Caglla CLI における Itinerary / Expense / Note の **ordering 責務** と、現在の **Sequence-first 実装方針** を整理する。Itinerary は Travel Activity Unit（行動単位）として扱う（[Itinerary モデル](itinerary-model.md)）。

現行 CLI では次が実装済みです。

```text
- Sequence-first ordering
- sparse ordering（1000 刻み）
- itinerary add --after / --before
- itinerary move
- itinerary normalize
- itinerary replicate
- day swap plan payload
```

関連: [Itinerary モデル](itinerary-model.md) / [Export Schema](export-schema.md) / [Note モデル](note-model.md) / [Expense モデル](expense-model.md)

検証データ: [沖縄・瀬底 canonical sample](../../samples/okinawa_sesoko_2026/README.md)

---

## 1. 背景

Itinerary は Calendar Event ではなく **Travel Activity Unit（行動単位）** です（[Itinerary モデル](itinerary-model.md)）。

旅行計画・旅行実施・旅行振り返りのいずれでも、利用者が管理したい中心は often:

```text
次に何をするか（行動シーケンス）
```

であり、

```text
厳密に何時だったか（カレンダー時刻）
```

だけではありません。

`start_time` は有用な **任意ラベル** ですが、行動順序の **正（source of truth）** である必要はありません。

---

## 2. 設計原則

### Sequence-first ordering

| 概念 | 役割 |
|---|---|
| **`sort_order`（sequence）** | Day 内の **行動順序の主情報**。「次に何をするか」の並び |
| **`start_time`** | 行動に付随する **任意の時刻ラベル**。表示・集計・将来のカレンダー連携用 |

**採用方針:**

```text
Time-first ordering ではなく Sequence-first ordering
```

同一 Day 内の既定ソート:

```text
day_number 昇順
→ sort_order 昇順
→ id 昇順
（必要に応じて start_time は表示のみ、または二次キー）
```

### Time-first ordering（採用しない方向）

カレンダーアプリに近い並び:

```text
start_time ありを先 → start_time 昇順 → sort_order
```

Itinerary を **行動台帳** として扱う Caglla では、計画段階で時刻未確定の行が多く、時刻なし行が一覧末尾に押し出される問題が出やすい（旧 Time-first 実装の課題。現行は Sequence-first）。

---

## 3. フィールド責務

| フィールド | Ordering 上の位置づけ |
|---|---|
| `day` / `day_number` | Day コンテナ。最上位の分割 |
| `sort_order` | **主キー。** 明示的な行動順序。CLI `--order`。未指定時は Day 末尾へ sparse ordering（1000 刻み） |
| `start_time` | **任意。** 付いていれば見出し・統計に使う。順序の主決定因子にしない |
| `id` | タイブレーク（作成順）。ユーザー向け意味は薄い |

### Expense

| フィールド | 並び |
|---|---|
| `sort_order` | 同一 Itinerary 内の主順序 |
| `id` | タイブレーク |

Expense は Itinerary 配下のみ。Trip / Day 直下には並びません。

### Note（Itinerary）

Export / import の `itinerary_key` は **`day_number` + `sort_order`** を第一解決キーとする（[Export Schema](export-schema.md)）。  
これは **sequence が安定参照** である設計と整合します。`start_time` は fallback 解決にのみ使われます。

---

## 4. Travel Ledger Model との適合

**Travel Ledger（行動台帳）** では:

- PDF / Excel の **スケジュール行** → Itinerary（順序が意味を持つ）
- **会計行** → Expense（親 Itinerary の文脈で並ぶ）
- 時刻が未記載の行も **序列上の位置** を持つ（例: 「移動 高速道路」）

したがって **Sequence-first** が台帳モデルに自然です。

`start_time` は台帳に書かれた時刻の **転記** であり、順序が時刻と矛盾する場合（時刻調整・概算時刻・時刻なし行）は **sequence を優先** する運用が望ましい。

---

## 5. 現行実装

CLI 各出力の sort は **Sequence-first** に統一済みです。

| 出力 / 操作 | 実装 | アルゴリズム | Ledger 適合 | 備考 |
|---|---|---|---|---|
| `itinerary list` | `src/itinerary.rs` | **Sequence-first** | ○ | `ITINERARY_LIST_ORDER_BY` |
| `itinerary timeline` | 同上（list 結果を表示） | **Sequence-first** | ○ | list と同一 |
| `day show` | `list_itinerary_items_for_day` | **Sequence-first** | ○ | list と同一 |
| `trip export` v1/v2 | `list_itinerary_items` | **Sequence-first** | ○ | フラット `itinerary_items[]` |
| `trip export` v3 | `build_trip_export_v3` | **Sequence-first** | ○ | `days[].itineraries[]` |
| `trip export-md` | `list_itinerary_items` 委譲 | **Sequence-first** | ○ | list と同一 |
| `trip import` | JSON の `sort_order` を DB に反映 | sequence 維持 | ○ | 配列順ではなくフィールド値 |
| Expense `list` | `expense.rs` | `sort_order → id` | ○ | Itinerary 内は sequence-first |
| Note export key | `note.rs` | `sort_order` 優先解決 | ○ | 設計と一致 |

### 共通 SQL

`list_itinerary_items` / `list_itinerary_items_for_day` / `trip export-md`（`src/itinerary.rs`）:

```text
ORDER BY day_number, sort_order, id
```

`start_time` は sort キーに含めない。表示・stats 用の任意ラベルとして扱う。

---

## 6. Canonical sample での示唆

[`samples/okinawa_sesoko_2026/seed.sh`](../../samples/okinawa_sesoko_2026/seed.sh) は、多くの行に **`--time` と `--order` の両方** を付与しています。

| パターン | 例 | 挙動 |
|---|---|---|
| 時刻 + order | Day 1 出発 `--time 06:00 --order 1` | `sort_order` 順。時刻と整合していれば従来と同順 |
| **order のみ** | `観光:首里城 --order 10`（時刻なし） | **`sort_order: 10` の位置**（Day 内中位）に表示 |
| order のみ | `移動 高速道路 --order 11` | 首里城の直後（`sort_order: 11`） |

`expected-export-v3.json`（golden）は **export v3 の Sequence-first 並び** を正とします。

---

## 7. ユーザー向け操作と ordering

| CLI | ordering への影響 |
|---|---|
| `itinerary add --order N` | `sort_order` を明示設定（**主操作**） |
| `itinerary add --after` / `--before` | 参照 Itinerary の直後 / 直前へ挿入（sparse ordering） |
| `itinerary add --time HH:MM` | `start_time` ラベル（**副操作**） |
| `itinerary update --order` / `--time` | 各フィールドを独立更新 |
| `itinerary move` | 既存 Itinerary を `--after` / `--before` 基準で Day 内（または参照先 Day）へ移動 |
| `itinerary normalize` | Day 内 `sort_order` を 1000 刻みに再採番 |
| `itinerary replicate` | 既存 Itinerary を指定 Day 群へ **独立した複製** として追加。元の `sort_order` を維持（recurring リンクなし） |
| `day swap` | Day 間で plan payload を交換（Itinerary / title / summary / Day-level notes）。`sort_order` / `start_time` は行ごとに維持 |

**UX:** ユーザーは `--order` で「次に何をするか」を編集し、`--time` はあれば付ける。一覧順は **order と一致** する。

---

## 8. 将来フェーズ（未実装）

| 項目 | 方針案 |
|---|---|
| `itinerary reorder` CLI | Day 内 `--order` の一括編集 |
| timezone フィールド | export / import 拡張（別バージョンで検討） |

### 履歴メモ

- v1.9.0: list / timeline / day show / export v3 / export-md の Sequence-first 統一
- export JSON の配列順変更は **behavioral breaking change**（DB 破壊変更ではない）。`schema_version: 3` は据え置き。詳細は [v1.9.0 release notes](../releases/v1.9.0-notes.md)

---

## 9. 実装参照

| 用途 | パス |
|---|---|
| Itinerary list / timeline | `src/itinerary.rs` — `list_itinerary_items`, `print_itinerary_timeline` |
| Day 配下一覧 | `src/itinerary.rs` — `list_itinerary_items_for_day` |
| Markdown export | `src/markdown.rs` — `list_itinerary_items_for_markdown` |
| JSON export v3 | `src/trip.rs` — `build_trip_export_v3` |
| Expense 一覧 | `src/expense.rs` |
| Note itinerary_key | `src/note.rs`, `docs/specifications/export-schema.md` |

---

## 10. 用語

| 用語 | 意味 |
|---|---|
| **Sequence-first** | `sort_order` を Day 内ソートの主キーとする |
| **Time-first** | ~~`start_time` を Day 内ソートの主キーとする（v1.8.1 まで list / export v3）~~ |
| **Travel Ledger** | 行動台帳。時系列の **行動列** + 紐づく Expense |
