# checklist-generate 検証用サンプル

v0.7.0 のカテゴリ組み合わせルール確認用スクリプトと実出力です。

## 再生成

```bash
bash samples/checklist_generate/generate_outputs.sh
```

## 確認サマリー

[`checklist_generate_verification_summary.md`](checklist_generate_verification_summary.md)

## 出力一覧

| シナリオ | カテゴリ | 主な確認 |
|---|---|---|
| `01-flight-hotel` | flight + hotel | combination + dedup + 2回目 skip |
| `02-flight-transport` | flight + transport | ETC / 免許 / レンタカー |
| `03-beach-activity` | beach + activity | サンダル + 着替え/防水/酔い止め |
| `04-museum-activity` | museum + activity | 事前予約 / 入場チケット |
| `05-beach-activity-export-md` | beach + activity | Markdown Checklist 反映 |
