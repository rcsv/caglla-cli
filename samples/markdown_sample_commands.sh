#!/usr/bin/env bash
# Markdown Export / trip stats の手動確認用サンプルデータ投入スクリプト
#
# 使い方（リポジトリルートから）:
#   bash samples/markdown_sample_commands.sh
#
# 投入後の確認:
#   cargo run -- trip stats 1
#   cargo run -- trip export-md 1
#   cargo run -- trip export-md 1 --output sample-trip.md

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

run() {
  cargo run --quiet -- "$@"
}

echo "==> DB を初期化"
run db reset

echo "==> 旅行を作成"
run trip add "Okinawa Sample Trip" --start 2026-04-26 --end 2026-04-29

echo "==> 日程を追加（15件 / 4日間）"

# Day 1
run itinerary add 1 --day 1 --time 08:30 --duration 120 --travel 60 \
  --location "Chubu Centrair International Airport" \
  --note "JL便 座席 12A" \
  "Flight to Naha"

run itinerary add 1 --day 1 --time 11:30 --duration 30 --travel 25 \
  --location "Naha Airport Rental Car Counter" \
  "Pick up rental car"

run itinerary add 1 --day 1 --time 15:00 --duration 45 --travel 20 \
  --location "Hotel Monterey Okinawa" \
  --note "荷物預けのみ" \
  "Hotel check-in"

run itinerary add 1 --day 1 --time 19:00 --duration 90 --travel 15 \
  --location "Makishi Public Market" \
  --note "沖縄そば" \
  "Dinner at Makishi"

# Day 2
run itinerary add 1 --day 2 --time 09:00 --duration 180 --travel 35 \
  --location "Emerald Beach" \
  --note "日焼け止め必須" \
  "Morning at Emerald Beach"

run itinerary add 1 --day 2 --time 12:30 --duration 60 --travel 20 \
  --location "Cafe Ocean Blue" \
  "Lunch near the beach"

run itinerary add 1 --day 2 --time 15:00 --duration 120 --travel 15 \
  --location "Kokusai Street" \
  "Shopping at Kokusai Street"

run itinerary add 1 --day 2 --time 17:30 --duration 120 --travel 30 \
  --location "Blue Cave Marina" \
  "Snorkeling tour"

# Day 3
run itinerary add 1 --day 3 --time 09:00 --duration 120 --travel 40 \
  --location "Shurijo Castle" \
  "Visit Shurijo Castle"

run itinerary add 1 --day 3 --time 12:00 --duration 75 --travel 25 \
  --location "Okinawa Soba Eibun" \
  "Lunch: Okinawa soba"

run itinerary add 1 --day 3 --time 14:30 --duration 90 --travel 50 \
  --location "Nago City" \
  "Drive to northern Okinawa"

run itinerary add 1 --day 3 --time 18:00 --duration 60 \
  --location "Bise Fukugi Tree Road" \
  "Sunset walk"

# Day 4
run itinerary add 1 --day 4 --time 08:00 --duration 30 --travel 20 \
  --location "Hotel Monterey Okinawa" \
  "Hotel checkout"

run itinerary add 1 --day 4 --time 10:00 --duration 150 --travel 45 \
  --location "Naha Airport" \
  --note "JL便 帰り" \
  "Return flight to Nagoya"

run itinerary add 1 --day 4 --time 14:00 --duration 45 --travel 10 \
  --location "AEON Mall Okinawa Rycom" \
  "Last-minute souvenir shopping"

echo "==> カテゴリを設定（itinerary update --category）"
run itinerary update 1 --category flight
run itinerary update 2 --category transport
run itinerary update 3 --category hotel
run itinerary update 4 --category restaurant
run itinerary update 5 --category beach
run itinerary update 6 --category restaurant
run itinerary update 7 --category shopping
run itinerary update 8 --category activity
run itinerary update 9 --category activity
run itinerary update 10 --category restaurant
run itinerary update 11 --category transport
# ID 12: Sunset walk — カテゴリ未設定（uncategorized 確認用）
run itinerary update 13 --category hotel
run itinerary update 14 --category flight
run itinerary update 15 --category shopping

echo "==> チェックリストを追加（10件）"
run checklist add 1 "パスポート"
run checklist add 1 "航空券（印刷またはアプリ）"
run checklist add 1 "充電器"
run checklist add 1 "水着"
run checklist add 1 "タオル"
run checklist add 1 "日焼け止め"
run checklist add 1 "レンタカー予約確認"
run checklist add 1 "ETCカード"
run checklist add 1 "カメラ"
run checklist add 1 "現金（小銭）"

echo "==> チェックリストの一部を完了にする"
run checklist check 1
run checklist check 2
run checklist check 7
run checklist check 10

echo ""
echo "==> サンプルデータの投入が完了しました"
echo ""
echo "確認コマンド:"
echo "  cargo run -- trip stats 1"
echo "  cargo run -- trip export-md 1"
echo "  cargo run -- trip export-md 1 --output sample-trip.md"
echo "  cargo run -- checklist list 1"
echo "  cargo run -- itinerary timeline 1"
