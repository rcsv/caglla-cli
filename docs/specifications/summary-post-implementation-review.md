# Summary Post-Implementation Review（責務整理 — 実装後レビュー）

Caglla.Travel CLI の **Travel Ledger Model** における **Summary** の責務を、**v1.17.0 実装後** に整理・検証するレビューです。

**v1.20.0 時点: 仕様整理のみ（v1 Hardening 第二弾）。** 本書は実装変更を伴わない。改善候補は §8 に記録する。

| ドキュメント | 役割 |
|---|---|
| [summary-responsibilities-review.md](summary-responsibilities-review.md) (v1.14.0) | **設計前**の責務整理 — 当時の前提での判断。**上書きしない** |
| [summary-entity-design.md](summary-entity-design.md) (v1.15.0) | フィールド・配置 |
| [summary-implementation-plan.md](summary-implementation-plan.md) (v1.16.0) | 実装計画 |
| **本書** (v1.20.0) | **実装後**の責務再定義 — Abstract レイヤー・Travel Book / Journal との境界 |

関連: [travel-ledger-responsibilities.md](travel-ledger-responsibilities.md) / [note-model.md](note-model.md) / [reservation-responsibilities-review.md](reservation-responsibilities-review.md) / [export-schema.md](export-schema.md)

設計系列:

```text
v1.14.0  Responsibilities Review（設計前）
v1.15.0  Entity Design
v1.16.0  Implementation Plan
v1.17.0  Implementation
v1.20.0  Post-Implementation Review  ← this document
```

---

## 1. Goals / Non-goals

### Goals（Summary が担うべきこと）

| 課題 | 解決イメージ |
|---|---|
| **Trip 全体を俯瞰する** | 旅行名だけでは伝わらない特徴・傾向を Abstract として提示 |
| **Travel Book の入力素材** | `trip export-md` 冒頭・Day 見出し直下の **要旨** |
| **Itinerary 列を読む前の orientation** | Day 単位で「その日がどんな日か」を短く把握 |
| **Remark / Note / Journal との分離** | 旅行記・感想・事実メモと混同しない |
| **将来の自動生成** | Trip Data から Summary Generator が本文を生成 |

### レビュー結論（Goals の核心）

```text
Summary はユーザーが旅行記を書く場所ではない。
Summary は Trip を俯瞰する Abstract レイヤーである。
```

将来的な主用途は **システムまたは AI による生成**。v1.17 の手入力 CLI は **interim**（bootstrap / override）として有効。

### Non-goals

| 概念 | 理由 | 正しい置き場 |
|---|---|---|
| **旅行記・感想・体験の叙述** | 時系列・一人称の Story | **Travel Journal**（将来） |
| **事実・注意・断片情報** | ユーザーが記録する運用メモ | **Note entity** |
| **予約・確認** | 構造化フィールド | **Reservation** |
| **費用** | 金額正本 | **Expense** |
| **行動の時系列** | 旅程正本 | **Itinerary** + Sequence |
| **行内短文補足** | 備考欄 | **Remark** |
| **施設・POI 正本** | Venue 領域 | Itinerary `location` / 将来 **Venue** |

---

## 2. Summary の責務再定義

### 定義

```text
Summary is an Abstract — a short, trip-level or day-level overview
derived from travel data, intended for scanning and Travel Book generation.
It is not a travel journal.
```

日本語:

```text
Summary = Abstract（要旨）
Trip / Day を俯瞰する要約。旅行記ではない。
```

### 性質

| 属性 | 方針 |
|---|---|
| **作者（target）** | Summary Generator（システム / AI） |
| **作者（interim）** | ユーザー手入力（v1.17 CLI）— bootstrap / manual override |
| **対象** | Trip / Day（各 0..1） |
| **長さ** | 短い — スキャン可能 |
| **トーン** | 説明的・中立・俯瞰（三人称に近い） |
| **例** | 「4日間で15件の Itinerary」「主な訪問地は美ら海水族館と瀬底ビーチ」「滞在型の家族旅行」 |

### Summary が載る場面

```text
trip show / export-md 冒頭
Day 見出し直下
export JSON trip.summary / days[].summary
将来: Travel Book Generator の入力
```

### Summary が載らない場面

```text
「今朝から水族館へ向かった」（体験叙述）  → Journal
「海邦丸は予約必須」（事実メモ）           → Note
「要: ETCカード」（行内補足）              → Remark
```

---

## 3. Note / Summary / Travel Journal の境界

### 三層モデル

```text
Note            = Fact      （ユーザー入力・事実・断片）
Summary         = Abstract  （生成物・俯瞰要旨）
Travel Journal  = Story     （ユーザー入力・体験・感想・時系列）
```

| | **Note** | **Summary** | **Travel Journal**（将来） |
|---|---|---|---|
| **入力** | ユーザー | **生成（主）** / 手動 override（interim） | ユーザー |
| **責務** | メモ・注意・事実 | Trip/Day の Abstract | 旅行記・振り返り・体験 |
| **件数** | 0..N | Trip/Day 各 0..1 | 0..N（想定） |
| **構造** | title + body | 単一テキスト | 時系列エントリ（想定） |
| **例** | 駐車場は北側が便利 | 主な訪問地は美ら海と瀬底ビーチ | ジンベエザメが印象的だった |

### なぜ Journal と分離するか

v1.14 では Summary を「旅行前の趣旨」「旅行後の一行振り返り」として扱う余地があり、将来 **Travel Journal** を導入すると **Summary と Journal の責務が近づく** 懸念が判明した。

```text
説明しないと違いが分からない機能は、長期的に機能しない。
```

したがって v1.20 以降の正は:

- **振り返り・感想・体験叙述** → Journal（将来）
- **俯瞰・統計的・要旨的な要約** → Summary（生成）
- **事実・運用メモ** → Note

### v1.14 Day Note 例の再分類

v1.14 / travel-ledger にあった Day Note「振り返り」例:

```text
美ら海は混雑していた。古宇利島のランチは予想以上に良かった。
```

| 旧置き場 | v1.20 再分類 |
|---|---|
| Day Note | **Travel Journal 候補** — Summary には載せない |
| Trip 一行説明（計画意図） | **Note** または将来 Generator 入力 — Summary 正本は生成結果 |

**v1.14 文書は設計履歴として残す。** 本書は実装後の責務再定義である。

---

## 4. v1.17 実装との整合確認

### 結論

```text
Summary を生成物（Abstract）として再定義しても、v1.17 実装は破綻していない。
```

ストレージは **作者に中立** な TEXT 列であり、本文の transport 層は作者に依存しない。

### DB

| 項目 | v1.17 実装 | レビュー |
|---|---|---|
| `trips.summary` | TEXT NULL | **十分** — Abstract 本文の正本 |
| `days.summary` | TEXT NULL | **十分** |
| `generated_at` 等 | なし | **v1.20 では不要** — 将来メタデータ候補（§8） |
| Migration | 追加済み | **変更不要** |

### CLI

| コマンド | v1.17 | 長期位置付け |
|---|---|---|
| `trip add --summary` | あり | **manual bootstrap** |
| `trip update --summary` / `--clear-summary` | あり | **manual override** |
| `day update --summary` | あり | **manual override** |
| `trip show` / `--json` | summary 表示 | **確認・連携** — 継続 |

v1.20 では **非推奨化・削除しない**。将来 `trip summary-generate` が主入口になる想定（§8）。

### Validation（`src/summary.rs`）

| ルール | 生成文との整合 |
|---|---|
| trim / 空 → NULL | **OK** |
| Trip 2000 / Day 1000 文字上限 | **OK** |
| 超過時「Note を使え」 | 長文は Note / Journal へ — **方向性と一致** |

---

## 5. Export / Import / Markdown / Diff レビュー

### Export / Import（schema v3）

```json
{
  "trip": { "summary": "…" },
  "days": [{ "day_number": 1, "summary": "…" }]
}
```

| 論点 | 判定 |
|---|---|
| 人間入力 vs 生成文 | **どちらも同じフィールド** — 問題なし |
| 省略時 / NULL | 後方互換 — **維持** |
| `validate-export` | 長さ検証 — **維持** |
| `trip duplicate` | roundtrip — **維持** |

**Export Schema 変更: v1.20 では不要。**

### Markdown export

| 項目 | v1.17 実装 | Travel Book との関係 |
|---|---|---|
| Trip summary | `# Title` 直下 | Abstract 冒頭 — **入力素材として妥当** |
| Day summary | `## Day N` 直下 | Day orientation — **妥当** |
| プレーンテキスト | Markdown 解釈なし | 生成文も同様 — **OK** |

### Diff

| 項目 | 判定 |
|---|---|
| `trip.summary` / Day summary 比較 | **維持** |
| 意味の変化 | 手修正 diff から **Trip Data 変化に追随する再生成 diff** へ解釈を更新 |
| 実装変更 | **v1.20 では不要** |

Trip Data が変わり Summary Generator が再実行された場合、diff で Summary が変わるのは **自然** である。

---

## 6. Travel Book との関係

### パイプライン（目標像）

```text
Trip Data
  （Itinerary, Expense, Reservation, Checklist, Note, stats…）
        ↓
Summary Generator          ← 将来（CLI / 外部 AI）
        ↓
Summary（Abstract レイヤー）
        ↓
Travel Book Generator      ← export-md 拡張 / PDF 等（製品 v5 想定）
        ↓
Travel Book（共有用しおり）
```

Summary は **Travel Book そのものではない**。**Book を生成するための要約中間レイヤー** である。

### v1.17 / v1.18 との組み合わせ

| レイヤー | Travel Book 内の役割 |
|---|---|
| **Summary** | 冒頭 Abstract |
| **Reservation** | 予約・手続きセクション（v1.18） |
| **Itinerary + Remark** | 旅程本体 |
| **Checklist** | 準備 |
| **Note** | 詳細付録（任意） |
| **Expense / stats** | 費用・Overview |

[long-term-version-strategy.md](../long-term-version-strategy.md) の **v5 Travel Book** と整合する。

---

## 7. v1.14 から変更された考え方

v1.14 [summary-responsibilities-review.md](summary-responsibilities-review.md) は **設計前レビュー** として有効なまま残す。以下は **v1.20 で更新する解釈**（v1.14 本文の上書きではない）。

| トピック | v1.14（当時前提） | v1.20（実装後） |
|---|---|---|
| **作者** | ユーザーが書く短い要約 | **生成が主** — 手入力は interim |
| **旅行後の一行** | Summary に載せる想定 | **Journal** が Story — Summary は **データ由来の Abstract** |
| **Day Note 振り返り** | Note の例として記載 | **Journal 候補** — Summary と混同しない |
| **「共有向けに書かれる」** | 能動的執筆 | **生成結果を共有** |
| **Note → Summary 抜き出し** | 手動運用可 | Generator が Trip Data + Note を **参照しうる** — 正本は生成結果 |
| **Itinerary Summary** | 初手不要（維持） | **変更なし** — title が行動ラベル |

### 変わらないもの

- Trip / Day スコープ（Itinerary レベル Summary 専用列は不要）
- Remark / Expense / Reservation との境界（予約・金額は Summary に書かない）
- export v3 フィールド配置
- canonical sample に Summary を大量投入しない方針

---

## 8. 改善候補（v1.20 では実装しない）

| # | 候補 | 種別 |
|---|---|---|
| 1 | `trip summary-generate` / `day summary-generate` | CLI — Generator 入口 |
| 2 | `generated_at` / `generator` / `source_revision` メタデータ | DB または export 拡張 |
| 3 | ルールベース Generator（itinerary 件数・カテゴリ・主要 location） | 第一世代（AI なし） |
| 4 | Summary 手入力 CLI の deprecated 候補 | GUI 成熟後 — **v1.20 では非推奨化しない** |
| 5 | Travel Journal entity 設計書 | 将来 v6 系列 |
| 6 | export-md を Travel Book Generator v0 として拡張 | v5 連携 |
| 7 | canonical 以外の **しおり検証用小規模 Trip** + 生成 Summary | sample |
| 8 | `trip doctor` — Summary 未生成の optional suggestion | doctor |

---

## 9. v1.20.0 スコープ（本書）

### 実施する

| 項目 | 内容 |
|---|---|
| 仕様書 | 本ドキュメント |
| 索引 | [specifications/README.md](README.md) |
| 参照更新 | travel-ledger-responsibilities、long-term-version-strategy |
| v1 Hardening | Summary 実装後責務再定義（第二弾） |

### 実施しない

```text
DB migration / schema 変更
CLI 変更・非推奨化
export / import schema 変更
Markdown / diff 実装変更
canonical sample 更新
テスト追加
v1.14 summary-responsibilities-review.md の上書き
```

---

## 10. 用語

| 用語 | 意味 |
|---|---|
| **Abstract** | Summary の性質 — 俯瞰要旨（旅行記ではない） |
| **Summary Generator** | Trip Data から Summary を生成する将来コンポーネント |
| **Travel Book** | 共有用しおり — Summary を入力素材とする出力物 |
| **Travel Journal** | ユーザー体験記（将来）— Story |
| **interim** | v1.17 手入力 CLI — bootstrap / override |

---

## 11. 実装参照（v1.17.0）

| 領域 | パス |
|---|---|
| Validation | `src/summary.rs` |
| Trip CRUD | `src/trip.rs` |
| Day update | `src/day.rs` |
| export / import | `src/trip.rs` |
| Markdown | `src/markdown.rs` |
| Diff | `src/diff.rs` |
| 統合テスト | `tests/summary_cli.rs` |
| 設計前レビュー | [summary-responsibilities-review.md](summary-responsibilities-review.md) (v1.14.0) |
