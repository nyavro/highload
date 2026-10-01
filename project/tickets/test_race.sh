#!/bin/bash

EVENT_ID="00000000-0000-0000-0000-000000000001"
SEAT_ID="seat_5"
URL="http://localhost:3003/events/$EVENT_ID/reserve"

echo "Sending two requests reserving one seat $SEAT_ID..."

# Run two requests in parallel
curl -s -X POST "$URL" \
  -H "Content-Type: application/json" \
  -d "{\"seat_id\": \"$SEAT_ID\", \"user_id\": \"user_A\"}" &
PID1=$!

curl -s -X POST "$URL" \
  -H "Content-Type: application/json" \
  -d "{\"seat_id\": \"$SEAT_ID\", \"user_id\": \"user_B\"}" &
PID2=$!

# Wait both to complete
wait $PID1
echo ""
wait $PID2
echo ""

echo "Test completed."