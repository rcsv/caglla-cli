# checklist-generate 強化確認サマリー (v0.7.0)

生成日: 2026-06-06  
再生成: `bash samples/checklist_generate/generate_outputs.sh`

## 01-flight-hotel

- **categories used**: `flight`, `hotel`
- **expected combination rule**: flight + hotel → 宿泊予約確認, 身分証明書, 充電器
- **generated checklist items** (run 1): 航空券確認, 身分証明書確認, 空港到着時刻確認, 宿泊予約確認, チェックイン時間確認, 住所確認, 身分証明書, 充電器
- **skipped items on second run**: 上記8件すべて skip（追加 0 件）
- **confirmed points**:
  - flight / hotel の default_checklist が生成される
  - 組み合わせルール由来の `身分証明書`, `充電器` が追加される
  - `宿泊予約確認` は hotel default で追加済みのため combination 側は skip
  - checklist list の ID 1〜8 が連番（sort_order 破綻なし）
  - 2回目実行で重複追加なし

- **output files**:
  - [`outputs/01-flight-hotel-generate-run1.txt`](outputs/01-flight-hotel-generate-run1.txt)
  - [`outputs/01-flight-hotel-list-after-run1.txt`](outputs/01-flight-hotel-list-after-run1.txt)
  - [`outputs/01-flight-hotel-generate-run2.txt`](outputs/01-flight-hotel-generate-run2.txt)

## 02-flight-transport

- **categories used**: `flight`, `transport`
- **expected combination rule**: flight + transport → ETCカード, 運転免許証, レンタカー予約確認
- **generated checklist items** (run 1): 航空券確認, 身分証明書確認, 空港到着時刻確認, 移動手段確認, 所要時間確認, ETCカード, 運転免許証, レンタカー予約確認
- **skipped items on second run**: 8件すべて skip（追加 0 件）
- **confirmed points**:
  - default_checklist と combination rule の両方が反映
  - 重複 title なし
  - 2回目は全件 skip

- **output files**:
  - [`outputs/02-flight-transport-generate-run1.txt`](outputs/02-flight-transport-generate-run1.txt)
  - [`outputs/02-flight-transport-list-after-run1.txt`](outputs/02-flight-transport-list-after-run1.txt)
  - [`outputs/02-flight-transport-generate-run2.txt`](outputs/02-flight-transport-generate-run2.txt)

## 03-beach-activity

- **categories used**: `beach`, `activity`
- **expected combination rule**: beach → サンダル / beach + activity → 着替え, 防水バッグ, 酔い止め
- **generated checklist items** (run 1): 水着, タオル, 日焼け止め, 予約確認, 所要時間確認, 服装確認, サンダル, 着替え, 防水バッグ, 酔い止め
- **skipped items on run 1 (combination dedup)**: 水着, タオル, 日焼け止め（beach rule 側で default と重複）
- **skipped items on second run**: 10件すべて skip（追加 0 件）
- **confirmed points**:
  - beach / activity の default_checklist が生成される
  - beach 単体 rule から `サンダル` が追加される
  - beach + activity rule から `着替え`, `防水バッグ`, `酔い止め` が追加される
  - 同一 title の重複登録なし
  - Markdown Export 確認対象（後述）

- **output files**:
  - [`outputs/03-beach-activity-generate-run1.txt`](outputs/03-beach-activity-generate-run1.txt)
  - [`outputs/03-beach-activity-list-after-run1.txt`](outputs/03-beach-activity-list-after-run1.txt)
  - [`outputs/03-beach-activity-generate-run2.txt`](outputs/03-beach-activity-generate-run2.txt)

## 04-museum-activity

- **categories used**: `museum`, `activity`
- **expected combination rule**: museum + activity → 事前予約確認, 入場チケット
- **generated checklist items** (run 1): 営業時間確認, チケット確認, 予約確認, 所要時間確認, 服装確認, 事前予約確認, 入場チケット
- **skipped items on second run**: 7件すべて skip（追加 0 件）
- **confirmed points**:
  - museum / activity の default_checklist が生成される
  - combination rule 由来の `事前予約確認`, `入場チケット` が追加される
  - 重複 title なし

- **output files**:
  - [`outputs/04-museum-activity-generate-run1.txt`](outputs/04-museum-activity-generate-run1.txt)
  - [`outputs/04-museum-activity-list-after-run1.txt`](outputs/04-museum-activity-list-after-run1.txt)
  - [`outputs/04-museum-activity-generate-run2.txt`](outputs/04-museum-activity-generate-run2.txt)

## Markdown Export 連携 (beach + activity)

- **output files**:
  - [`outputs/05-beach-activity-export-md.txt`](outputs/05-beach-activity-export-md.txt)
  - [`export_after_generate.md`](export_after_generate.md)
- **confirmed points**:
  - 自動生成された 10 件が `## Checklist` セクションに出力される
  - checklist title の重複なし
  - Overview / Day 見出し / 空行整形は従来どおり維持
