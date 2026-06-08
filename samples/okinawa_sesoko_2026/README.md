# 沖縄・瀬底 2026 canonical sample (v0)

実旅行データ（`EstimateTrip_20260426.pdf`）由来の canonical sample です。  
観光地デモではなく、高速道路・駐車場・空港内買い物・レンタカー・給油・買い出し・チェックイン/アウト・個人負担・領収書番号まで含む **実イベント台帳** を現行モデルで表現する検証用データです。

| 項目 | 値 |
|---|---|
| Trip | 沖縄 瀬底 4日間 |
| 期間 | 2026-04-26 〜 2026-04-29 |
| Itinerary | 58 件 |
| Expense | 49 件 |
| 合計（JPY） | ¥561,780（PDF 会計合計と一致） |
| Checklist | 4 件 |

## 投入

リポジトリルートから:

```bash
bash samples/okinawa_sesoko_2026/seed.sh
```

`caglla.db` をリセットしたうえで Trip ID `1` を作成します（約 1〜2 分）。

## 確認コマンド

```bash
cargo run -- trip stats 1
cargo run -- trip export-md 1
cargo run -- trip export 1 --output /tmp/okinawa-export.json
cargo run -- trip validate-export /tmp/okinawa-export.json
cargo run -- trip import /tmp/okinawa-export.json
```

## seed 化ルール（要約）

### Itinerary

- PDF のスケジュール行を原則 Itinerary として登録
- 時刻あり → `--time`、なし → `sort_order` のみ
- 買い物の追加購入・レシート分割行などは **直前の Itinerary に Expense を追加**（例: 昼食追加、土産屋さんの複数レシート）

### Expense

- 金額がある行は対応 Itinerary 配下に登録
- `旅費` / `食費` / `個別：…` は `note` に保持
- `個別：知弘` / `個別：節子` は `paid_by_name` にも反映
- 領収書 `R-xxx` は `note` に `領収書: R-xxx` 形式で保持

### 意図的に省略したもの

- 金額なし行（ロイズ R-033、有料道路「700円ぐらい？」など）
- Note エンティティの大量投入（備考は Itinerary / Expense `note` に集約）
- Participant / Settlement / Expense category

## ファイル

| ファイル | 説明 |
|---|---|
| `seed.sh` | CLI でデータ投入 |
| `expected-export-v3.json` | seed 後 export の正規化 golden file（metadata 除く） |

### golden file の再生成

```bash
bash samples/okinawa_sesoko_2026/seed.sh
cargo run -- trip export 1 --output /tmp/okinawa-export.json
jq '{
  schema_version: .schema_version,
  trip: (.trip | {name, start_date, end_date}),
  days: .days,
  checklist_items: [.checklist_items[] | {title, is_done, sort_order}],
  notes: .notes
}' /tmp/okinawa-export.json > samples/okinawa_sesoko_2026/expected-export-v3.json
```

## 検証

`tests/okinawa_sesoko_seed_cli.rs` が seed → export → golden 比較と `validate-export` を実行します。
