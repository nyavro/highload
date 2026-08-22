#!/bin/bash

set -e

NGINX_HOST="${NGINX_HOST:-localhost}"
NGINX_PORT="${NGINX_PORT:-80}"
DIALOG_ID="${DIALOG_ID:?Err: not set DIALOG_ID}"
TOKEN="${TOKEN:?Err: not set TOKEN}"

ENDPOINT="/dialog/${DIALOG_ID}/send"
REQUEST_URL="http://${NGINX_HOST}:${NGINX_PORT}${ENDPOINT}"

CONCURRENT="${1:-10}"
DURATION="${2:-30}"

echo "Endpoint: $REQUEST_URL"
echo "Concurrent: $CONCURRENT requests"
echo "Duration:   ${DURATION}s"
echo "---------------------------------------"

echo ""
echo "Checking availability..."
CHECK_HTTP=$(curl -s -o /dev/null -w "%{http_code}" --connect-timeout 3 "$REQUEST_URL" 2>/dev/null) || CHECK_HTTP="000"
if [ "$CHECK_HTTP" = "000" ]; then
    echo "[ERROR] failed to connect to $REQUEST_URL"
    exit 1
fi
echo "[OK] Available (HTTP $CHECK_HTTP)"

START_TIME=$(date +%s)
SUCCESS=0
FAILED=0
TOTAL_TIME=0
MIN_TIME=999999
MAX_TIME=0

make_request() {
    local req_id=$1
    local start end elapsed http_code

    start=$(date +%s%N)
    http_code=$(curl -s -o /dev/null -w "%{http_code}" \
        -X POST \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer ${TOKEN}" \
        --connect-timeout 5 \
        --max-time 10 \
        "$REQUEST_URL" 2>/dev/null) || http_code="000"
    end=$(date +%s%N)

    elapsed=$(( (end - start) / 1000000 ))  # в мс

    if [ "$http_code" = "200" ] || [ "$http_code" = "201" ]; then
        SUCCESS=$((SUCCESS + 1))
    else
        FAILED=$((FAILED + 1))
    fi
    TOTAL_TIME=$((TOTAL_TIME + elapsed))

    if [ $elapsed -lt $MIN_TIME ]; then MIN_TIME=$elapsed; fi
    if [ $elapsed -gt $MAX_TIME ]; then MAX_TIME=$elapsed; fi
}

echo ""
echo "Loading ($CONCURRENT in parallel)..."
echo ""

PIDS=()
for i in $(seq 1 $CONCURRENT); do
    make_request $i &
    PIDS+=($!)
done

for pid in "${PIDS[@]}"; do
    wait "$pid" 2>/dev/null || true
done

END_TIME=$(date +%s)
ELAPSED=$((END_TIME - START_TIME))

echo ""
echo "========================================="
TOTAL=$((SUCCESS + FAILED))
echo "Total: $TOTAL"
echo "Succeeded:       $SUCCESS"
echo "Failed:          $FAILED"
if [ $TOTAL -gt 0 ]; then
    AVG_TIME=$((TOTAL_TIME / TOTAL))
    echo "Avg time: ${AVG_TIME}ms"
    echo "min time:        ${MIN_TIME}ms"
    echo "max time:        ${MAX_TIME}ms"
    if [ $ELAPSED -gt 0 ]; then
        RPS=$((TOTAL / ELAPSED))
        echo "RPS:             $RPS req/s"
    fi
fi
echo "Took: ${ELAPSED}s"
echo "========================================="

if [ $FAILED -gt 0 ]; then
    exit 1
fi
exit 0
