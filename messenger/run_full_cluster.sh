#!/bin/bash
set -e

PROJECT_NAME="${COMPOSE_PROJECT_NAME:-citus}"

echo "================================================="
echo "Starging cluster: $PROJECT_NAME"
echo "================================================="
echo ""

echo "[1/5] Stop previous..."
docker-compose -p $PROJECT_NAME down 2>/dev/null || true
echo "      Done"

echo ""
echo "[2/5] Building Docker image messenger..."
docker-compose -p $PROJECT_NAME build messenger_1 messenger_2 messenger_3
echo "      Done"

echo ""
echo "[3/5] Run Citus cluster (master + 3 workers)..."
docker-compose -p $PROJECT_NAME up -d master manager
echo "      awiting master..."
sleep 5
docker-compose -p $PROJECT_NAME up -d --scale worker=3
echo "      Workers launched"

echo ""
echo "awaiting Citus workers..."
sleep 15
echo "      Done"

echo ""
echo "[4/5] Running Tarantool + HAProxy..."
docker-compose -p $PROJECT_NAME up -d tarantool haproxy
echo "      Done"

echo ""
echo "[5/5] Running 3x Messenger + Nginx..."
docker-compose -p $PROJECT_NAME up -d messenger_1 messenger_2 messenger_3 nginx
echo "      Done"

echo ""
echo "Awaiting services..."
sleep 10

echo ""
echo "================================================="
echo "The cluster is up!"
echo "================================================="
echo ""
echo "Available endpoints:"
echo "  Nginx :                     http://localhost:80"
echo "  HAProxy stats:              http://localhost:8404/stats (admin/admin123)"
echo "  PostgreSQL master (extern): localhost:5532"
echo "  PostgreSQL replicas (ext):  localhost:5433"
echo "  Tarantool:                  localhost:3301"
echo ""
docker-compose -p $PROJECT_NAME ps
echo ""
echo "To stop: docker-compose -p \$PROJECT_NAME down"
echo "To load: ./load_test.sh [concurrent] [duration]"
echo "================================================="
