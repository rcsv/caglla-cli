#!/usr/bin/env bash
# Reservation 専用サンプル（okinawa canonical sample は変更しない）
set -euo pipefail

cargo run -- db reset
cargo run -- trip add "Reservation Demo" --start 2026-04-26 --end 2026-04-29

cargo run -- itinerary add 1 --day 1 --time 16:40 "Check-in"
cargo run -- itinerary add 1 --day 1 --time 14:30 "NU045 NGO ⇒ OKA"
cargo run -- itinerary add 1 --day 2 --time 10:00 "Rental car pickup"

cargo run -- reservation add --itinerary 1 \
  --reservation-type hotel \
  --provider "Hilton Sesoko Resort" \
  --confirmation ABC123 \
  --site-url "https://example.com/booking/abc123" \
  --remark "Twin room" \
  --start-at "2026-04-26T16:40" \
  --end-at "2026-04-29T10:00"

cargo run -- reservation add --itinerary 2 \
  --reservation-type flight \
  --provider "JAL" \
  --confirmation JAL123

cargo run -- reservation add --itinerary 3 \
  --reservation-type rental_car \
  --provider "KS Rent A Car" \
  --confirmation XYZ987 \
  --remark "ETC card required"

cargo run -- reservation list --trip 1
cargo run -- reservation list --itinerary 1
cargo run -- reservation show 1
cargo run -- trip export-md 1 --output /tmp/reservation-demo.md
cargo run -- trip export 1 --output /tmp/reservation-demo.json

echo "Reservation demo complete."
