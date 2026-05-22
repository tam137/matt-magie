#!/bin/bash
MM_EXEC="./target/release/Matt-Magie"
ENGINE_OLD="engines/suprah-0.2.2"
ENGINE_NEW="engines/suprah-0.4.2"
PGN="verification_v0.4.2.pgn"
LOG="verification_v0.4.2.log"

rm -f "$PGN" "$LOG"
touch "$PGN"

echo "Starting 10-game tournament verification..."
for r in {1..5}; do
  echo "=== Round $r.1: $(basename $ENGINE_OLD) (White) vs $(basename $ENGINE_NEW) (Black) ==="
  $MM_EXEC "$ENGINE_OLD" "$ENGINE_NEW" "$LOG" "$PGN" "Verification-Match" "local" "$r" "10000" "10" "log_on" "debug_on"
  
  echo "=== Round $r.2: $(basename $ENGINE_NEW) (White) vs $(basename $ENGINE_OLD) (Black) (Colors swapped) ==="
  $MM_EXEC "$ENGINE_NEW" "$ENGINE_OLD" "$LOG" "$PGN" "Verification-Match" "local" "$r" "10000" "10" "log_on" "debug_on"
done

echo "Tournament finished! Results:"
python3 summary.py "$PGN"
