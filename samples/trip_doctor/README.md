# trip doctor 検証用サンプル

`trip doctor` の各チェックパターンを個別に検証するためのスクリプトと実出力です。

## 再生成

```bash
bash samples/trip_doctor/generate_outputs.sh
```

各シナリオは `db reset` 後に trip ID 1 として投入し、`trip doctor 1` の出力を `outputs/` に保存します。

## 出力一覧

| ファイル | パターン | 期待する主な出力 |
|---|---|---|
| [`outputs/01-clean-trip.txt`](outputs/01-clean-trip.txt) | 問題なし | `No major issues found.` |
| [`outputs/02-empty-itinerary.txt`](outputs/02-empty-itinerary.txt) | itinerary 0件 | `Info` / `No itinerary found.` |
| [`outputs/03-overloaded-day.txt`](outputs/03-overloaded-day.txt) | 予定過多 | `Day 1 has many itineraries (8)` |
| [`outputs/04-no-restaurant.txt`](outputs/04-no-restaurant.txt) | 食事不足 | `Day 1 has no restaurant` + suggestion |
| [`outputs/05-high-travel-time.txt`](outputs/05-high-travel-time.txt) | 移動時間超過 | `Day 1 has high travel time (3h25m)` |
| [`outputs/06-missing-duration.txt`](outputs/06-missing-duration.txt) | 所要時間未設定 | `1 itinerary has no duration estimate` |
| [`outputs/07-combined-issues.txt`](outputs/07-combined-issues.txt) | 複合 | 上記複数が同時に表示 |

## 手動確認

個別シナリオを対話的に試す場合は、スクリプト内の該当ブロックを参考にコマンドを実行してください。
