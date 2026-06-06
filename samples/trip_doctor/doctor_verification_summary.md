# trip doctor 改善確認サマリー (v0.6.1)

生成日: 2026-06-06  
再生成: `bash samples/trip_doctor/generate_outputs.sh`

## 01-clean-trip

- **expected purpose**: 問題のない旅行。警告・提案・Info なし。
- **actual output file**: [`outputs/01-clean-trip.txt`](outputs/01-clean-trip.txt)
- **confirmed points**:
  - `No major issues found.` が表示される
  - `Warnings` / `Suggestions` / `Info` セクションは出ない

## 02-empty-itinerary

- **expected purpose**: itinerary 0件でもエラーにならず、Info で案内する (v0.6.1 改善点)。
- **actual output file**: [`outputs/02-empty-itinerary.txt`](outputs/02-empty-itinerary.txt)
- **confirmed points**:
  - `No major issues found.` ではない
  - `Info` セクションが表示される
  - `- No itinerary found.` が表示される
  - `Warnings` / `Suggestions` は出ない

## 03-overloaded-day

- **expected purpose**: 1日7件以上の予定過多を検出。
- **actual output file**: [`outputs/03-overloaded-day.txt`](outputs/03-overloaded-day.txt)
- **confirmed points**:
  - `Warnings` セクションが表示される
  - `- Day 1 has many itineraries (8)` が表示される

## 04-no-restaurant

- **expected purpose**: restaurant カテゴリがない日を検出。
- **actual output file**: [`outputs/04-no-restaurant.txt`](outputs/04-no-restaurant.txt)
- **confirmed points**:
  - `- Day 1 has no restaurant` が `Warnings` に表示される
  - `- Consider adding a lunch or dinner plan to Day 1` が `Suggestions` に表示される

## 05-high-travel-time

- **expected purpose**: 1日の travel_minutes 合計 180分以上を検出。
- **actual output file**: [`outputs/05-high-travel-time.txt`](outputs/05-high-travel-time.txt)
- **confirmed points**:
  - `- Day 1 has high travel time (3h25m)` が表示される（205分）
  - `- Consider reducing travel time on Day 1` が `Suggestions` に表示される

## 06-missing-duration

- **expected purpose**: duration 未設定を件数付きで表示 (v0.6.1 改善点)。
- **actual output file**: [`outputs/06-missing-duration.txt`](outputs/06-missing-duration.txt)
- **confirmed points**:
  - `Some itineraries have no duration estimate` ではない
  - `- 1 itinerary has no duration estimate` が単数形で表示される

## 07-combined-issues

- **expected purpose**: 複数問題が同時に表示される。
- **actual output file**: [`outputs/07-combined-issues.txt`](outputs/07-combined-issues.txt)
- **confirmed points**:
  - `Warnings` に overloaded / no restaurant / high travel time / duration 未設定が同時表示
  - `Suggestions` に restaurant 追加提案と travel time 削減提案が複数日分表示
  - 表示順は `Warnings` → `Suggestions`（Info なし）
  - duration 未設定は `- 1 itinerary has no duration estimate`
