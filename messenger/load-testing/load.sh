#!/bin/bash

TOKEN=$1
INSTANCES=("http://localhost:3001" "http://localhost:3002" "http://localhost:3003")
DIALOG_ID="b9b0f7cf-aa97-47b9-a046-6450874cd8b8"

echo "Loading... [CTRL+C] to break"

COUNTER=0
while true; do
  INSTANCE=${INSTANCES[$((COUNTER % 3))]}
  URL="${INSTANCE}/dialog/${DIALOG_ID}/send"
  PAYLOAD="{\"text\": \"Hope just fine! ${COUNTER}\"}"
  curl -s -X POST "$URL" \
    -H "Authorization: Bearer ${TOKEN}" \
    -H "Content-Type: application/json" \
    -H "Accept: application/json" \
    -d "$PAYLOAD" > /dev/null &

  COUNTER=$((COUNTER + 1))
  sleep 0.01
done
