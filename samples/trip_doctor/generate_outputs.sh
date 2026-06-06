#!/usr/bin/env bash
# trip doctor 全パターンの検証出力を生成する
#
# 使い方（リポジトリルートから）:
#   bash samples/trip_doctor/generate_outputs.sh
#
# 出力先:
#   samples/trip_doctor/outputs/*.txt

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="$ROOT/samples/trip_doctor/outputs"
cd "$ROOT"

mkdir -p "$OUT_DIR"

run() {
  cargo run --quiet -- "$@"
}

write_header() {
  local file="$1"
  local title="$2"
  local description="$3"
  {
    echo "# trip doctor validation: $title"
    echo "# Generated: $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
    echo "#"
    echo "# $description"
    echo "#"
    echo "# Command: cargo run -- trip doctor 1"
    echo ""
  } >"$file"
}

append_doctor_output() {
  local file="$1"
  run trip doctor 1 >>"$file"
}

echo "==> 01-clean-trip"
run db reset
run trip add "Clean Trip" --start 2026-05-01 --end 2026-05-01
run itinerary add 1 --day 1 --time 12:00 --duration 60 --travel 20 \
  --location "Cafe Example" "Lunch" 
run itinerary update 1 --category restaurant
write_header "$OUT_DIR/01-clean-trip.txt" "clean trip" \
  "restaurant あり・移動時間・所要時間ともに問題なし"
append_doctor_output "$OUT_DIR/01-clean-trip.txt"

echo "==> 02-empty-itinerary"
run db reset
run trip add "Empty Trip"
write_header "$OUT_DIR/02-empty-itinerary.txt" "empty itinerary" \
  "itinerary 0件でもエラーにならない"
append_doctor_output "$OUT_DIR/02-empty-itinerary.txt"

echo "==> 03-overloaded-day"
run db reset
run trip add "Overloaded Trip" --start 2026-05-02 --end 2026-05-02
for i in $(seq 1 8); do
  run itinerary add 1 --day 1 --time "$(printf "%02d:00" $((8 + i)))" \
    --duration 30 --travel 5 --location "Spot $i" "Activity $i"
  run itinerary update "$i" --category restaurant
done
write_header "$OUT_DIR/03-overloaded-day.txt" "overloaded day" \
  "Day 1 に 8件（7件以上で warning）"
append_doctor_output "$OUT_DIR/03-overloaded-day.txt"

echo "==> 04-no-restaurant"
run db reset
run trip add "No Restaurant Trip" --start 2026-05-03 --end 2026-05-03
run itinerary add 1 --day 1 --time 10:00 --duration 120 --travel 30 \
  --location "Shurijo Castle" "Castle visit"
run itinerary update 1 --category activity
write_header "$OUT_DIR/04-no-restaurant.txt" "no restaurant" \
  "restaurant カテゴリがない日"
append_doctor_output "$OUT_DIR/04-no-restaurant.txt"

echo "==> 05-high-travel-time"
run db reset
run trip add "High Travel Trip" --start 2026-05-04 --end 2026-05-04
run itinerary add 1 --day 1 --time 09:00 --duration 60 --travel 100 \
  --location "North" "Drive north leg 1"
run itinerary update 1 --category transport
run itinerary add 1 --day 1 --time 12:00 --duration 60 --travel 90 \
  --location "South" "Drive south leg 2"
run itinerary update 2 --category transport
run itinerary add 1 --day 1 --time 18:00 --duration 90 --travel 15 \
  --location "Naha" "Dinner"
run itinerary update 3 --category restaurant
write_header "$OUT_DIR/05-high-travel-time.txt" "high travel time" \
  "Day 1 の travel_minutes 合計 205分（180分以上で warning）"
append_doctor_output "$OUT_DIR/05-high-travel-time.txt"

echo "==> 06-missing-duration"
run db reset
run trip add "Missing Duration Trip" --start 2026-05-05 --end 2026-05-05
run itinerary add 1 --day 1 --time 14:00 --travel 10 \
  --location "Unknown stop" "Unplanned stop"
run itinerary add 1 --day 1 --time 19:00 --duration 60 --travel 10 \
  --location "Restaurant" "Dinner"
run itinerary update 2 --category restaurant
write_header "$OUT_DIR/06-missing-duration.txt" "missing duration" \
  "duration_minutes 未設定の itinerary がある"
append_doctor_output "$OUT_DIR/06-missing-duration.txt"

echo "==> 07-combined-issues"
run db reset
run trip add "Combined Issues Trip" --start 2026-05-06 --end 2026-05-08
# Day 1: overloaded + no restaurant + high travel
for i in $(seq 1 8); do
  run itinerary add 1 --day 1 --time "$(printf "%02d:00" $((7 + i)))" \
    --duration 20 --travel 25 --location "Leg $i" "Transit $i"
  run itinerary update "$i" --category transport
done
# Day 2: no restaurant only
run itinerary add 1 --day 2 --time 10:00 --duration 90 --travel 20 \
  --location "Museum" "Museum visit"
run itinerary update 9 --category museum
# Day 3: missing duration + no restaurant
run itinerary add 1 --day 3 --time 11:00 --travel 30 --location "Walk" "Free time"
write_header "$OUT_DIR/07-combined-issues.txt" "combined issues" \
  "複数の warning / suggestion が同時に出る"
append_doctor_output "$OUT_DIR/07-combined-issues.txt"

echo ""
echo "==> 検証出力を生成しました: $OUT_DIR"
ls -1 "$OUT_DIR"
