# Note Post-Implementation Review（責務整理 — 実装後レビュー）

Caglla.Travel CLI の **Travel Ledger Model** における **Note** の責務を、**v1.3.0 実装後**（export v2 / diff 含む）に整理・検証するレビューです。

**v1.21.0 時点: 仕様整理のみ（v1 Hardening 第三弾）。** 本書は実装変更を伴わない。改善候補は §9 に記録する。

| ドキュメント | 役割 |
|---|---|
| [note-model.md](note-model.md) (v1.3.0) | 設計草案 + 初回実装 — **上書きしない** |
| [travel-ledger-responsibilities.md](travel-ledger-responsibilities.md) (v1.10.0) | Summary / Remark / Note / Reservation の横断比較 |
| [summary-post-implementation-review.md](summary-post-implementation-review.md) (v1.20.0) | Summary = Abstract、Journal 分離 |
| **本書** (v1.21.0) | **実装後**の責務再定義 — Annotation Layer、Narrative 境界 |

関連: [reservation-responsibilities-review.md](reservation-responsibilities-review.md) / [export-schema.md](export-schema.md) / [long-term-version-strategy.md](../long-term-version-strategy.md)

設計系列:

```text
v1.3.0   note-model.md + DB / CLI
v1.4.0   Export schema v2（notes[]）
v1.4.1   trip diff Note support
v1.21.0  Post-Implementation Review  ← this document
```

---

## 1. Goals / Non-goals

### Goals（Note が担うべきこと）

| 課題 | 解決イメージ |
|---|---|
| **対象への補足** | Trip / Day / Itinerary に付与する **Annotation**（0..N） |
| **計画・実行の文脈** | Fact / Observation / Decision / Reminder を自由記述で保持 |
| **Summary / Journal との分離** | Abstract（生成要旨）・Story（体験の語り）と混同しない |
| **バックアップ・移行** | export v2+ `notes[]`、import roundtrip、`trip diff` |
| **Travel Book 入力素材** | 将来、しおりの付録・詳細節のソースの一つ |

### レビュー結論（Goals の核心）

```text
Note は Fact だけを保存する場所ではない。
Note は Trip / Day / Itinerary に付与される Annotation Layer である。
```

Note は対象エンティティに対する補足情報を保持し、以下を含む。

```text
Fact
Observation
Decision
Reminder
```

Note は **Narrative を持たない**。体験・感情・時系列の語りは **Travel Journal**（将来）の領域。

### Non-goals

| 概念 | 理由 | 正しい置き場 |
|---|---|---|
| **旅行全体の要旨** | 俯瞰 Abstract | **Trip / Day Summary** |
| **体験・感情・旅行記** | Narrative あり | **Travel Journal**（将来） |
| **Itinerary 行内短文** | インライン補足 | **Remark**（`itinerary_items.note`） |
| **予約番号・確認 URL** | 構造化正本 | **Reservation** |
| **チェック可能な準備項目** | 実行トラッキング | **Checklist** |
| **金額** | 費用正本 | **Expense** |
| **行動の時系列** | 旅程正本 | **Itinerary** + Sequence |

---

## 2. Note の責務再定義

### 定義

```text
Note is an Annotation Layer — supplementary information attached to
Trip, Day, or Itinerary. It does not carry narrative.
```

日本語:

```text
Note = Annotation Layer（注釈層）
対象エンティティへの補足・説明・検討・記録。Narrative を持たない。
```

### Annotation の内訳

| 種別 | 説明 | 例 |
|---|---|---|
| **Fact** | 事実・制約 | レンタカー返却は16:00まで |
| **Observation** | 観察・所見 | 美ら海水族館は朝イチ推奨 |
| **Decision** | 判断・方針 | 古宇利島は体力を見て判断 |
| **Reminder** | 注意・持参 | サンダルを持参する |

v1.x では `note_type` 列は **持たない**。本文の意味論として上記を包含する。型付けは将来候補（§9）。

### 性質

| 属性 | 方針 |
|---|---|
| **作者** | ユーザー入力 |
| **対象** | Trip / Day / Itinerary（各 0..N） |
| **構造** | `title`（任意）+ `body`（必須） |
| **Narrative** | **なし** — 対象への補足であり、体験の語りではない |
| **文脈** | 旅行前・中・後いずれも可（計画メモも実行メモも Annotation） |

### Note が載る場面

```text
note add / list / show（CLI）
export JSON notes[]
trip diff（added / removed / changed）
将来: export-md の Note 節、Travel Book 付録
```

### Note が載らない場面

```text
「子供が喜んでいた」（感情・体験）     → Travel Journal
「4日間で美ら海と瀬底を訪問」（俯瞰）   → Summary
「要: ETCカード」（行内短文）           → Remark
「予約番号 ABC123」                   → Reservation
```

---

## 3. Travel Ledger Responsibility Model（三層）

v1 系における責務整理の **最終形**:

```text
Note           = Annotation Layer
Summary        = Abstract Layer
Travel Journal = Story Layer
```

| レイヤー | 入力 | Narrative | 件数 | 例 |
|---|---|---|---|---|
| **Note** | ユーザー | なし | 0..N per target | 朝イチ推奨、返却16:00まで |
| **Summary** | 生成（主）/ 手動 override | なし（俯瞰） | Trip/Day 各 0..1 | 滞在型家族旅行、主な訪問地は美ら海 |
| **Travel Journal** | ユーザー | **あり** | 0..N（想定） | ジンベエザメが印象的だった |

v1.20 の `Fact / Abstract / Story` 略記は、本書で **Annotation / Abstract / Story** に精緻化する。`Fact` は Annotation の一種として残る。

---

## 4. Summary との関係

```text
Summary = Abstract Layer   — Trip / Day 全体を俯瞰する要約
Note    = Annotation Layer — 個別対象への補足
```

| 観点 | **Summary** | **Note** |
|---|---|---|
| **スコープ** | Trip または Day 全体 | 特定 Trip / Day / Itinerary |
| **視点** | 俯瞰・統合 | 付箋・補足 |
| **作者** | Summary Generator（主） | ユーザー |
| **関係** | 階層ではなく **参照** — Generator が Note を入力に読みうる |

矛盾はない。Summary が全体を説明し、Note が個別対象を補足する。

---

## 5. Travel Journal との関係（Narrative 境界）

### 判定基準

**Narrative の有無** とする。

### Note（Narrative なし）

```text
Fact / Observation / Decision / Reminder
```

対象への補足。主語は **場所・予定・旅行の事実** に近い。

### Travel Journal（Narrative あり）

```text
Experience / Emotion / Story
```

体験の語り。主語は **自分／同行者の体験・感情** に近い。

```text
子供が喜んでいた
夕日が綺麗だった
今日は雨だった
```

### グレーゾーンの扱い

v1.20 で Journal 候補とした Day Note 例:

```text
美ら海は混雑していた。古宇利島のランチは予想以上に良かった。
```

| 文 | 再分類 |
|---|---|
| 美ら海は混雑していた | **Observation**（Note 可） |
| 予想以上に良かった | 評価・感情が入る → **Journal 寄り** |
| 子供が喜んでいた | **Emotion** → Journal |

複合段落は **文・エントリ単位** で判定する。1 つの Note フィールドに Journal 寄りの語りを混在させても v1.x では拒否しないが、**責務上は分離を推奨** する。

### v1.14 / travel-ledger の「振り返り」例

[note-model.md](note-model.md) および [travel-ledger-responsibilities.md](travel-ledger-responsibilities.md) §4 にあった「日別の振り返り」「旅行後の感想」は、Narrative 境界により再分類する。

| 旧表現 | v1.21 再分類 |
|---|---|
| 日別の振り返り（体験叙述） | **Travel Journal** |
| 旅行後の感想 | **Travel Journal** |
| 雨天時の代替案 | **Decision**（Note） |
| 予約時のやり取りメモ（背景） | **Fact**（Note） |

**note-model.md は設計履歴として残す。** 本書は実装後の責務再定義である。

---

## 6. Remark / Checklist との境界

Note を Annotation と広げても、既存の Remark / Checklist との境界は維持する。

### Remark（インライン Annotation）

| | **Remark** | **Note entity** |
|---|---|---|
| **粒度** | Itinerary 1 件に 1 フィールド | 1 target に 0..N |
| **長さ** | 短文（旅程表の行内） | 長文可 |
| **構造** | 単一文字列 | `title` + `body` |
| **例** | 要: ETCカード | 駐車場は北側が便利（詳細） |

Remark は **インライン Annotation** の特殊形。Long-form へ昇格させる自動変換は行わない（[travel-ledger-responsibilities.md](travel-ledger-responsibilities.md) §3）。

### Checklist vs Note (Reminder)

| | **Checklist** | **Note (Reminder)** |
|---|---|---|
| **性質** | チェック可能な準備項目 | 文脈付きの注意 |
| **例** | サンダル | 砂浜用にサンダルを持参（ビーチ日のみ） |
| **優先** | 実行トラッキングが主目的なら Checklist | 理由・条件の説明が主なら Note |

---

## 7. v1.3+ 実装との整合確認

### 結論

```text
Note を Annotation Layer として再定義しても、v1.3+ 実装は破綻していない。
```

`notes` テーブルは **作者中立・型なしの自由記述** ストレージであり、Fact 限定でも Annotation 全般でも transport 層は同一である。

### DB

| 項目 | 実装 | レビュー |
|---|---|---|
| `notes` テーブル | `owner_type` + `owner_id` + `title` + `body` | **十分** |
| `note_type` | なし | **v1.21 では不要** — 将来候補（§9） |
| cascade | trip / day / itinerary 削除時に手動 cascade | **維持** |
| Migration | v1.3 追加済み | **変更不要** |

### CLI

| コマンド | 実装 | レビュー |
|---|---|---|
| `note add` | `--trip` / `--day` / `--itinerary` | **維持** — Annotation 追加の正本 |
| `note list` / `show` / `update` / `delete` | CRUD | **維持** |
| `note list --trip` | Trip 直下のみ | **維持** — `--all` は将来候補 |

v1.21 では **非推奨化・削除しない**。

---

## 8. Export / Import / Markdown / Diff レビュー

### Export / Import（schema v2+）

```json
{
  "notes": [
    {
      "owner_type": "itinerary",
      "owner_itinerary_ref": { "day": 1, "title": "美ら海水族館" },
      "title": null,
      "body": "朝イチ推奨"
    }
  ]
}
```

| 論点 | 判定 |
|---|---|
| Annotation 種別の区別なし | **問題なし** — 本文 transport は型に依存しない |
| owner 参照（day_number / itinerary ref） | **維持** |
| import roundtrip | **維持** |
| `validate-export` | **維持** |

**Export Schema 変更: v1.21 では不要。**

### Markdown export

| 項目 | 現状 | レビュー |
|---|---|---|
| Long-form Note entity | **export-md 未組み込み** | 改善候補（§9）— 責務再定義とは独立 |
| `itinerary_items.note`（Remark） | 行内「メモ: …」表示 | **維持** |

Annotation 再定義は export-md 未実装の理由を変えない。将来 Travel Book 付録として Note 節を追加する。

### Diff

| 項目 | 判定 |
|---|---|
| `notes` added / removed / changed | **維持**（v1.4.1） |
| 意味の変化 | Annotation 内容の変更として自然 |
| 実装変更 | **v1.21 では不要** |

---

## 9. Travel Book との関係

### パイプライン（目標像）

Summary を唯一の上流とする階層ではなく、**並列ソースを編集・集約** する。

```text
Summary
Note
Travel Journal（将来）
Photo（将来）
Reservation
Expense
Itinerary + Remark
Checklist
        ↓ 編集・集約
Travel Book（共有用しおり）
```

| ソース | Travel Book 内の役割 |
|---|---|
| **Summary** | 冒頭 Abstract |
| **Note** | 付録・詳細補足（任意） |
| **Travel Journal** | 体験記セクション（将来） |
| **Reservation** | 予約・手続き |
| **Itinerary + Remark** | 旅程本体 |

[summary-post-implementation-review.md](summary-post-implementation-review.md) §6 および [long-term-version-strategy.md](../long-term-version-strategy.md) の v5 Travel Book と整合する。

---

## 10. v1.20 から変更された考え方

[summary-post-implementation-review.md](summary-post-implementation-review.md) (v1.20.0) は **上書きしない**。以下は **v1.21 で精緻化する解釈**。

| トピック | v1.20（略記） | v1.21（精緻化） |
|---|---|---|
| **Note ラベル** | `Fact` | **Annotation Layer**（Fact はその一種） |
| **Note 責務** | 事実・運用メモ | Fact / Observation / Decision / Reminder |
| **Journal 境界** | 振り返り → Journal | **Narrative の有無** で判定 |
| **Summary 境界** | Abstract | **変更なし** — Annotation と矛盾なし |
| **Travel Book** | Summary → Book の記述あり | **並列ソースモデル** を明示 |

### 変わらないもの

- Trip / Day / Itinerary の owner モデル
- Remark / Reservation / Expense との境界
- export v2+ `notes[]` 配置
- canonical sample に Note を大量投入しない方針

---

## 11. 改善候補（v1.21 では実装しない）

| # | 候補 | 種別 |
|---|---|---|
| 1 | `note_type` またはタグ（fact / observation / decision / reminder） | DB または export 拡張 |
| 2 | `note list --trip --all` — Day / Itinerary 配下を含む一覧 | CLI |
| 3 | `trip export-md` への Long-form Note 節 | Markdown / Travel Book |
| 4 | Travel Journal entity 設計書 | 将来 v6 系列 |
| 5 | Narrative 検出の optional doctor suggestion | doctor |
| 6 | Summary Generator が Note を参照するルール | Generator |
| 7 | GUI での「概要 / 補足 / 旅行記」タブ分離 | 製品 UI |

---

## 12. v1.21.0 スコープ（本書）

### 実施する

| 項目 | 内容 |
|---|---|
| 仕様書 | 本ドキュメント |
| 索引 | [specifications/README.md](README.md) |
| 参照更新 | travel-ledger-responsibilities、summary-post-implementation-review、note-model、long-term-version-strategy |
| v1 Hardening | Note 実装後責務再定義（第三弾） |

### 実施しない

```text
DB migration / schema 変更
CLI 変更・非推奨化
export / import schema 変更
Markdown / diff 実装変更
canonical sample 更新
テスト追加
note-model.md の上書き
summary-post-implementation-review.md の上書き
```

---

## 13. 用語

| 用語 | 意味 |
|---|---|
| **Annotation** | Note の性質 — 対象への補足（Narrative なし） |
| **Abstract** | Summary の性質 — Trip / Day の俯瞰要旨 |
| **Story** | Travel Journal の性質 — 体験の語り（Narrative あり） |
| **Narrative** | 体験・感情・時系列の語り — Journal と Note の分界軸 |
| **Remark** | Itinerary 行内の短文インライン補足 |

---

## 14. 実装参照（v1.3.0+）

| 領域 | パス |
|---|---|
| CRUD | `src/note.rs` |
| Models | `src/models.rs`（`Note`, `ExportNote`） |
| export / import | `src/trip.rs` |
| Diff | `src/diff.rs` |
| DB | `src/db.rs` |
| 統合テスト | `tests/note_cli.rs`, `tests/export_roundtrip_cli.rs` |
| 設計草案 | [note-model.md](note-model.md) (v1.3.0) |
