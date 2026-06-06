#!/usr/bin/env bash
# checklist-generate 組み合わせルールの検証出力を生成する
#
# 使い方（リポジトリルートから）:
#   bash samples/checklist_generate/generate_outputs.sh
#
# 出力先:
#   samples/checklist_generate/outputs/

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="$ROOT/samples/checklist_generate/outputs"
cd "$ROOT"

mkdir -p "$OUT_DIR"

run() {
  cargo run --quiet -- "$@" 2>&1
}

write_header() {
  local file="$1"
  local title="$2"
  local description="$3"
  {
    echo "# checklist-generate validation: $title"
    echo "# Generated: $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
    echo "#"
    echo "# $description"
    echo "#"
  } >"$file"
}

setup_flight_hotel() {
  run db reset >/dev/null
  run trip add "Flight Hotel Trip" --start 2026-06-01 --end 2026-06-03 >/dev/null
  run itinerary add 1 --day 1 --time 08:00 --duration 120 --travel 60 \
    --location "Airport" "Flight to destination" >/dev/null
  run itinerary update 1 --category flight >/dev/null
  run itinerary add 1 --day 1 --time 15:00 --duration 45 --travel 20 \
    --location "Hotel Example" "Hotel check-in" >/dev/null
  run itinerary update 2 --category hotel >/dev/null
}

setup_flight_transport() {
  run db reset >/dev/null
  run trip add "Flight Transport Trip" --start 2026-06-02 --end 2026-06-04 >/dev/null
  run itinerary add 1 --day 1 --time 09:00 --duration 120 --travel 45 \
    --location "Airport" "Outbound flight" >/dev/null
  run itinerary update 1 --category flight >/dev/null
  run itinerary add 1 --day 1 --time 12:00 --duration 30 --travel 25 \
    --location "Rental Car Counter" "Pick up rental car" >/dev/null
  run itinerary update 2 --category transport >/dev/null
}

setup_beach_activity() {
  run db reset >/dev/null
  run trip add "Beach Activity Trip" --start 2026-06-03 --end 2026-06-05 >/dev/null
  run itinerary add 1 --day 1 --time 09:00 --duration 180 --travel 30 \
    --location "Emerald Beach" "Morning at beach" >/dev/null
  run itinerary update 1 --category beach >/dev/null
  run itinerary add 1 --day 1 --time 14:00 --duration 120 --travel 20 \
    --location "Blue Cave Marina" "Snorkeling tour" >/dev/null
  run itinerary update 2 --category activity >/dev/null
}

setup_museum_activity() {
  run db reset >/dev/null
  run trip add "Museum Activity Trip" --start 2026-06-04 --end 2026-06-06 >/dev/null
  run itinerary add 1 --day 1 --time 10:00 --duration 90 --travel 15 \
    --location "City Museum" "Museum visit" >/dev/null
  run itinerary update 1 --category museum >/dev/null
  run itinerary add 1 --day 1 --time 14:00 --duration 120 --travel 25 \
    --location "Adventure Park" "Zip line tour" >/dev/null
  run itinerary update 2 --category activity >/dev/null
}

capture_scenario() {
  local slug="$1"
  local setup_fn="$2"
  local description="$3"

  echo "==> $slug"
  "$setup_fn"

  write_header "$OUT_DIR/${slug}-generate-run1.txt" "$slug run 1" "$description"
  echo "# Command: cargo run -- trip checklist-generate 1" >>"$OUT_DIR/${slug}-generate-run1.txt"
  echo "" >>"$OUT_DIR/${slug}-generate-run1.txt"
  run trip checklist-generate 1 >>"$OUT_DIR/${slug}-generate-run1.txt"

  write_header "$OUT_DIR/${slug}-list-after-run1.txt" "$slug list after run 1" "$description"
  echo "# Command: cargo run -- checklist list 1" >>"$OUT_DIR/${slug}-list-after-run1.txt"
  echo "" >>"$OUT_DIR/${slug}-list-after-run1.txt"
  run checklist list 1 >>"$OUT_DIR/${slug}-list-after-run1.txt"

  write_header "$OUT_DIR/${slug}-generate-run2.txt" "$slug run 2" "2回目: 既存項目は skip される想定"
  echo "# Command: cargo run -- trip checklist-generate 1" >>"$OUT_DIR/${slug}-generate-run2.txt"
  echo "" >>"$OUT_DIR/${slug}-generate-run2.txt"
  run trip checklist-generate 1 >>"$OUT_DIR/${slug}-generate-run2.txt"
}

capture_scenario "01-flight-hotel" setup_flight_hotel \
  "flight + hotel: default_checklist + combination rule"
capture_scenario "02-flight-transport" setup_flight_transport \
  "flight + transport: default_checklist + combination rule"
capture_scenario "03-beach-activity" setup_beach_activity \
  "beach + activity: default_checklist + combination rule"
capture_scenario "04-museum-activity" setup_museum_activity \
  "museum + activity: default_checklist + combination rule"

echo "==> markdown export after checklist-generate (beach + activity)"
setup_beach_activity
run trip checklist-generate 1 >/dev/null

write_header "$OUT_DIR/05-beach-activity-export-md.txt" "beach activity markdown export" \
  "checklist-generate 後の Markdown Export"
echo "# Command: cargo run -- trip export-md 1" >>"$OUT_DIR/05-beach-activity-export-md.txt"
echo "" >>"$OUT_DIR/05-beach-activity-export-md.txt"
run trip export-md 1 >>"$OUT_DIR/05-beach-activity-export-md.txt"

run trip export-md 1 --output "$ROOT/samples/checklist_generate/export_after_generate.md" >/dev/null

echo ""
echo "==> 検証出力を生成しました: $OUT_DIR"
ls -1 "$OUT_DIR"
