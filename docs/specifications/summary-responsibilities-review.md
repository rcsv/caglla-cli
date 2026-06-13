# Summary Responsibilities Review（責務整理）

Caglla.Travel CLI の **Travel Ledger Model** における **Summary** の責務を整理するレビューです。

**v1.14.0 時点: 仕様整理のみ（設計前レビュー）。** DB migration、CLI、export schema、Markdown export の変更は行いません。

> **実装後レビュー:** v1.17.0 実装後の責務再定義は [summary-post-implementation-review.md](summary-post-implementation-review.md)（v1.20.0）。本書（v1.14.0）は設計履歴として残す。

| ドキュメント | 役割 |
|---|---|
| [travel-ledger-responsibilities.md](travel-ledger-responsibilities.md) (v1.10.0) | Summary / Remark / Note / Reservation の横断比較 |
| **本書** (v1.14.0) | Summary に焦点 — 定義・境界・Trip/Day 役割・将来統合 |

関連: [Note モデル](note-model.md) / [Itinerary モデル](itinerary-model.md) / [Day モデル](day-model.md) / [Export Schema](export-schema.md) / [Reservation モデル](reservation-model.md)

---

## 1. Goals

Summary が解決する課題。

| 課題 | 解決イメージ |
|---|---|
| **旅行全体の概要を短く伝える** | Trip レベルで「この旅行は何か」を同行者・未来の自分に共有 |
| **その日の見どころを伝える** | Day レベルで詳細旅程を読まなくても「その日がどんな日か」を把握 |
| **しおりの冒頭説明を提供する** | `trip export-md` や印刷物の表紙・Day 見出し直下にスキャン向けテキスト |
| **旅行後の記録を要約する** | 振り返りの **一行〜数行の結論**（長文日記は Note の領域） |
| **一覧・検索での識別** | Trip 一覧で名前以外の文脈を付与（将来の GUI / export） |
| **Remark / Note との混同を防ぐ** | 「概要」と「備考」「詳細メモ」の UI ラベルとデータモデルを分離 |

Summary は **読む人が短時間で全体像を掴む** ための概念です。データ入力の正本（予約番号・金額・施設住所）にはなりません。

---

## 2. Non-goals

Summary が **扱わない** もの。

| 概念 | 理由 | 正しい置き場 |
|---|---|---|
| **詳細な旅行記** | 長文・複数段落の記録 | **Note entity** |
| **自由形式の長文** | 検討・経緯・雨天代替案 | **Note entity** |
| **予約情報** | 構造化された確認番号・手続き | **Reservation**（将来） |
| **費用情報** | 金額・通貨・領収書 | **Expense** |
| **施設情報** | 住所・電話・緯度経度 | **Venue** / Itinerary `location`（将来 Venue） |
| **行動の時系列** | いつ何をするかの正本 | **Itinerary** 列 + Sequence |
| **短文の行内補足** | 旅程表の備考欄 | **Remark**（`itinerary_items.note`） |
| **準備・忘れ物** | チェック項目 | **Checklist** |
| **写真・添付** | バイナリメディア | **Photo / Attachment**（将来） |

---

## 3. Summary とは何か

### 定義

```text
Summary is a short, reader-oriented overview
written for sharing, scanning, and guidebook-style reading.
```

日本語:

```text
共有・印刷・しおり向けに書かれる、読者目線の短い要約
```

Travel Ledger における Summary の **スコープは Trip と Day** を主対象とする（§6, §7）。Itinerary レベルは §8 で検討。

### 検討観点

| 観点 | 方針 |
|---|---|
| **誰に向けて書くのか** | 同行者、家族、未来の自分。**第三者が読んでも旅行の意図が伝わる** ことを想定 |
| **どの場面で読むのか** | 旅行前（計画共有・しおり）、旅行中（当日の orientation）、旅行後（一覧・振り返りの入口） |
| **どの程度の長さか** | **短い** — おおむね 1〜5 文、または箇条書き数行。A4 しおりで **スキャン可能** な分量 |
| **トーン** | 説明的・中立。日記的な一人称長文は Note へ |
| **更新頻度** | 旅行前に書き、旅行後に **一行だけ更新** する程度を想定。頻繁な編集ログは Note |

### Summary の性質（まとめ）

```text
対象:     Trip / Day（主）
件数:     各 0 or 1（複数 Summary は想定しない）
長さ:     短い（要約）
読者:     共有・しおり・印刷
構造:     単一テキスト（title 不要 — Trip.name / Day 番号が見出し）
```

---

## 4. Summary と Remark

| | **Summary** | **Remark** |
|---|---|---|
| **日本語ラベル** | 概要 | 備考 |
| **スコープ** | Trip / Day | **Itinerary 1 件** |
| **責務** | 要約 — 「この旅行／この日はどんな日か」 | 補足 — 「この行動について一言」 |
| **長さ** | 短い（複数文可） | **より短い**（一行が多い） |
| **表示** | しおり冒頭・Day 見出し直下 | 旅程表の **行内** |
| **件数** | Trip/Day 各 0..1 | Itinerary 各 0..1 |
| **実装** | **未実装**（将来 `trips.summary` / `days.summary`） | **既存** `itinerary_items.note` |

### 使い分けの例（沖縄旅行）

**Day Summary（将来）**

```text
Day 2 — 海洋博公園（美ら海水族館）、古宇利島、ハナサキマルシェ
```

**Itinerary Remark（現行）**

```text
チェックイン行の Remark: チェックアウトリミット：10:00
高速道路行の Remark: 要: ETCカード
```

Summary は **日全体のテーマ**、Remark は **個別行動の実務補足**。Remark を Day 全体の要約に流用しない。

### 混同を避ける理由

Remark は export v3 の itinerary `note` として既に広く使われている。ここに Day 要約を書くと、旅程表の行と混ざり **しおりの Day セクションが読めなくなる**。

---

## 5. Summary と Note

| | **Summary** | **Note entity** |
|---|---|---|
| **日本語ラベル** | 概要 | メモ |
| **責務** | 読者向け要約 | 詳細記録・検討・振り返り |
| **長さ** | 短い | **長文可** |
| **件数** | Trip/Day 各 0..1 | Trip/Day/Itinerary 各 **0..N** |
| **構造** | 単一テキスト | `title` + `body` |
| **読者** | 共有・しおり | 本人・同行者の詳細閲覧 |
| **実装** | 未実装 | **CRUD 実装済み** |

### 使い分けの例

**Trip Summary（将来）**

```text
GWちょっと手前で行くことで、飛行機の料金を格安に抑える。夏前の過ごしやすい沖縄４日間。
```

**Trip Note（現行）**

```text
title: 航空券の検討
body: JAL と ANA を比較した結果、時刻の都合で NU 便に。座席は窓側を希望したが…
（複数段落の検討過程）
```

**Day Note（現行）**

```text
title: Day 2 振り返り
body: 美ら海は混雑していた。古宇利島のランチは予想以上に良かった。次回は早朝入場を検討。
```

### 「Trip Note」呼称の整理

過去に「Trip Note」と呼ばれていた用途のうち、**共有向け一行説明** は Summary が正しい（[travel-ledger-responsibilities.md §2](travel-ledger-responsibilities.md#2-summary-と-note-は分ける)）。Note entity は **Long-form** に専念させる。

### 昇格・降格

| 方向 | 方針 |
|---|---|
| Summary → Note | 要約が長文化したら **手動で Note に移す**。自動昇格はしない |
| Note → Summary | Note から要約を **抜き出して** Trip/Day Summary に書く運用は可 |
| Remark → Summary | **不可** — スコープが異なる |

---

## 6. Trip Summary

### 役割

```text
旅行全体の概要
旅行の目的・狙い
旅行の特徴・トーン
同行者向けの一文説明
```

Trip Summary は **Trip の表紙・一覧・しおり冒頭** に載るテキストです。`trips.name`（旅行名）を補完し、名前だけでは伝わらない **意図と文脈** を渡します。

### 例（計画共有資料より）

```text
GWちょっと手前で行くことで、飛行機の料金を格安に抑える。
夏前の過ごしやすい沖縄４日間。
```

### フィールド案（将来 — 未実装）

| 候補 | 説明 |
|---|---|
| `trips.summary` | 単一 TEXT 列。NULL 可 |
| `trips.description` | `summary` の別名候補 — 実装時に **どちらか一方** に統一 |

本書では **`summary`** を用語として統一する。`description` は GUI ラベル候補（「旅行の説明」）として併記可。

### Trip Summary が載る場面（将来）

```text
trip show（概要セクション）
trip list（サブタイトル的表示 — 任意）
trip export-md（# タイトル直下）
export JSON（trip オブジェクト内）
```

### Trip Summary が載らない場面

- 予約一覧 → Reservation 集約
- 費用合計 → Expense / stats
- 詳細検討ログ → Note

---

## 7. Day Summary

### 役割

```text
その日のテーマ
主な行先（スキャン向け）
行動の要約（詳細旅程の前に読む）
```

Day Summary は **Day 見出しの直下** に置き、Itinerary 列を読む前に「今日はどんな日か」を伝えます。

### 例（計画共有資料より）

```text
■ 主な行先

Day 1  首里城、瀬底島（瀬底大橋）
Day 2  海洋博公園（美ら海水族館、ドリームセンター）、古宇利島、沖縄ハナサキマルシェ
Day 3  伊江島（リリーフィールド、ハイビスカス園、城山、湧出、ニャティア洞）、瀬底ビーチ
Day 4  御菓子御殿、万座毛、美浜アメリカンビレッジ
```

実装時は **1 日あたり 1 テキスト**（上記の 1 行相当）を想定。複数行・箇条書きも可。

### フィールド案（将来 — 未実装）

| 候補 | 説明 |
|---|---|
| `days.summary` | 単一 TEXT 列。NULL 可 |

### Day Summary と Itinerary 列の関係

| 情報 | 正本 |
|---|---|
| 行動の順序・時刻 | **Itinerary** 列（Sequence-first） |
| その日の **読み物としての要約** | **Day Summary** |
| 個別行動の補足 | **Remark** |

Day Summary は Itinerary 列の **代替ではない**。要約は人が書き、旅程の正本は Itinerary のまま維持する。

### 旅行前 vs 旅行後

| フェーズ | Day Summary の典型内容 |
|---|---|
| **Before trip** | 主な行先・テーマ（計画共有） |
| **After trip** | 一日の締めの一言（「伊江島ドライブ＋瀬底ビーチで充実」）— 任意 |

旅行後の詳細振り返りは **Day Note** が主。Summary は **入口** に留める。

---

## 8. Itinerary Summary は必要か

### 結論: **初手では不要**

| 観点 | 判断 |
|---|---|
| **title の役割** | Itinerary `title` は既に **行動の一行ラベル** として Summary 的機能を担っている |
| **Remark の役割** | 実務補足は Remark で足りる |
| **Note の役割** | 行動単位の詳細は Itinerary Note で 0..N 件 |
| **しおり** | 旅程表は `start_time` + `title` + Remark で構成 — 別 Summary 列は冗長になりやすい |

```text
Itinerary Summary 専用フィールド → v1.x では採用しない
title が Summary の役割を果たす
```

### title と Summary の違い（Itinerary レベル）

| | Itinerary `title` | （仮）Itinerary Summary |
|---|---|---|
| 例 | `チェックイン` / `NU045 NGO ⇒ OKA` | `ヒルトン瀬底に到着し荷物を預ける` |
| 性質 | **行動名**（旅程表の行見出し） | 説明文 |
| 必要性 | **必須** | 低 — 説明が要るなら Note 1 件で足りる |

### 例外（将来検討）

GUI で「タイトルと説明を分けたい」需要が強い場合:

- **Option A:** 専用 `summary` 列は作らず、**Itinerary Note（title なし、短文 body）** で代用
- **Option B:** `itinerary_items.description` を任意列として追加 — **優先度低**

Reservation 実装計画と同様、**フィールドを増やす前に既存概念で足りるか** を優先する。

---

## 9. Export / Markdown

将来 Summary を実装した場合の方針。**本段階では未確定** — Reservation と同様、実装フェーズで export schema v3 拡張か **v4** かを再評価する。

### Export JSON（案）

**schema v3 拡張候補** — `trip` / `days[]` に summary フィールドを追加:

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

| 論点 | 方針案 |
|---|---|
| **省略時** | `summary` キーなしまたは `null` — import 時は NULL 扱い |
| **後方互換** | 既存 v3 export に `summary` なし → 問題なし |
| **内部 ID** | summary は trip / day に埋め込み — 別 ID 不要 |
| **schema v4** | Summary + Reservation + Participant 等をまとめて切る選択肢は残す |
| **import 順序** | Trip（summary 含む）→ Day（summary 含む）→ 既存フロー |

### Import / validate-export

- `summary` は **任意文字列**。長さ上限は実装時（例: 4000 文字）— 本段階では規定しない
- 空文字は NULL と同等に扱うか、実装時に決定

### Markdown export（`trip export-md`）案

**Trip レベル** — `# 旅行名` 直下:

```markdown
# 沖縄 瀬底 4日間

GWちょっと手前で行くことで、飛行機の料金を格安に抑える。夏前の過ごしやすい沖縄４日間。

**期間:** 2026-04-26 〜 2026-04-29
```

**Day レベル** — `## Day N` 直下:

```markdown
## Day 2 — 2026-04-27

海洋博公園（美ら海水族館）、古宇利島、沖縄ハナサキマルシェ

### 09:00 美ら海水族館
...
```

| 論点 | 方針 |
|---|---|
| Summary なし | 現行どおり Day 見出しのみ — セクション省略 |
| Trip Note / Day Note | Summary 節の **後**、Itinerary 列の **前** に既存 Note 節を配置（Note モデルと整合） |
| Remark | Itinerary 行内 — 変更なし |

### trip diff（将来）

- `trip.summary` / `day.summary` の追加・変更・削除を比較対象に含める
- 比較キー: Trip は単一フィールド、Day は `day_number`

---

## 10. Canonical Sample

対象: [`samples/okinawa_sesoko_2026/`](../../samples/okinawa_sesoko_2026/)

### 現状

| 項目 | 状態 |
|---|---|
| 主目的 | **旅行後台帳・清算・export roundtrip** |
| Trip/Day Summary | **意図的に省略** |
| 概要に相当する情報 | 計画共有 PDF には存在するが、seed には含めていない |

### Summary を入れる価値

| 観点 | 評価 |
|---|---|
| **清算・export 検証** | Summary なしで十分 — golden file 変更のメリット小 |
| **しおり・計画共有検証** | Summary があれば `export-md` の冒頭が richer に — **別フェーズの関心** |
| **seed 複雑度** | Trip 1 件 + Day 4 件の短文追加は軽いが、**主目的から外れる** |
| **旅行前 vs 旅行後** | 本 sample は After trip 中心 — Day Summary（主な行先）は **計画資料の再現** には有用だが必須ではない |

### 方針

| 段階 | 方針 |
|---|---|
| **Summary 実装直後** | okinawa への **一括投入はしない**（Reservation と同型） |
| **最小検証** | 計画共有 PDF 由来の 4 行 Day Summary を **手動または seed オプション** で入れるテストは将来可 |
| **別サンプル** | 旅行前しおり検証用に **小規模 Trip** を新設する方が、canonical sample の清算検証と分離できて安全 |

### 参考: 投入するならこの内容（実装フェーズ）

```text
Trip summary:
  GWちょっと手前で行くことで、飛行機の料金を格安に抑える。夏前の過ごしやすい沖縄４日間。

Day summaries（計画共有資料より）:
  Day 1  首里城、瀬底島（瀬底大橋）
  Day 2  海洋博公園（美ら海水族館、ドリームセンター）、古宇利島、沖縄ハナサキマルシェ
  …
```

---

## 11. Future Relationship

Summary と今後整理・実装が進む概念との関係。

### 概念マップ（目標像）

```text
Trip
 ├─ summary              ← 本書（将来）
 ├─ Note[]               ← 詳細記録（実装済み）
 ├─ Checklist            ← 準備（実装済み）
 ├─ Participant[]        ← 将来 — 同行者・精算
 └─ Day
      ├─ summary         ← 本書（将来）
      ├─ Note[]
      └─ Itinerary
           ├─ title      ← 行動ラベル（Summary 代替しうる）
           ├─ remark      ← 備考（既存 note 列）
           ├─ Expense     ← 費用（実装済み）
           ├─ Reservation ← 予約（仕様済み・未実装）
           ├─ Note[]
           ├─ Photo[]     ← 将来
           └─ Attachment[]← 将来
```

### 他概念との境界（再掲）

| 概念 | Summary との関係 |
|---|---|
| **Note** | Summary = 短い入口、Note = 詳細。同一 Trip に両方共存可 |
| **Photo** | 写真キャプションの正本は Photo / Note。Summary に画像参照を埋め込まない |
| **Attachment** | PDF 等の添付 — Summary とは独立。しおりに「資料あり」は Checklist や Note で |
| **Participant** | 同行者名・精算 — Summary の読者指定ではない。`paid_by_name` 等は Expense 側 |
| **Reservation** | 予約番号・手続き — Summary に書かない。しおりでは **別セクション** |
| **Remark** | 行内補足 — Day Summary の代替にしない |

### Export Schema 全体との位置づけ

Summary は **軽量フィールド**（trip / day に各 1 文字列）。Reservation / Participant / Photo より先に実装しやすいが、export では **v3 拡張と v4 新設のどちらにも載せやすい**。

実装順序の候補（未確定）:

```text
1. Summary（Trip + Day）— DB + CLI + export
2. Reservation — 別計画書参照
3. Participant — 精算フェーズ
4. Photo / Attachment — メディアフェーズ
```

### GUI 表示の割り切り（推奨）

[travel-ledger-responsibilities.md §4](travel-ledger-responsibilities.md#4-note-entitylong-form-note) と同型:

```text
Trip 画面:  概要（Summary） + メモ（Note）
Day 画面:   概要（Summary） + メモ（Note）
Itinerary:  備考（Remark）  + メモ（Note）— Summary 欄なし
```

---

## 12. v1.14.0 スコープ（本書）

### 実施する

| 項目 | 内容 |
|---|---|
| 仕様書 | 本ドキュメント |
| 索引 | [specifications/README.md](README.md) |
| 参照 | travel-ledger-responsibilities、note-model、itinerary-model 等 |

### 実施しない

```text
DB migration
trips.summary / days.summary カラム追加
CLI 実装
export / import schema 変更
Markdown export 変更
canonical sample 更新
```

---

## 13. 用語

| 用語 | 意味 |
|---|---|
| **Summary** | Trip / Day の短い共有向け要約 |
| **Remark** | Itinerary 行の備考（`itinerary_items.note`） |
| **Note** | Long-form 自由記述 entity（`title` + `body`） |
| **Travel Ledger** | 行動台帳 — Sequence-first Itinerary + 紐づく補助情報 |

---

## 14. 実装参照（現行）

| 概念 | 状態 |
|---|---|
| Trip Summary | **未実装** |
| Day Summary | **未実装** |
| Remark | `itinerary_items.note` — `src/itinerary.rs` |
| Note entity | `src/note.rs` |
| 横断責務 | [travel-ledger-responsibilities.md](travel-ledger-responsibilities.md) |
| Markdown | `src/markdown.rs` — Summary 節なし（現行） |
| Export | [export-schema.md](export-schema.md) — `summary` フィールドなし（現行） |
