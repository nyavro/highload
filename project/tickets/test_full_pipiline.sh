# Read connection settings from the .env file
if [ -f .env ]; then
    export $(cat .env | grep -v '#' | xargs)
fi

# 1. Set base parameters
EVENT_ID="00000000-0000-0000-0000-000000000004"
USER_ID="00000000-0000-0000-0000-00000000000a"

echo "🔍 Step 0: Querying available seat UUID from PostgreSQL..."
# Make a quick query to the Postgres container to get the UUID for 'Sector 1', Row 1, Seat 5
SEAT_ID=$(docker exec -i postgres_tickets psql -U "$POSTGRES_USER" -d "$POSTGRES_DB_NAME" -t -A -c \
  "SELECT id FROM seats WHERE sector = 'Sector 1' AND row_number = 1 AND seat_number = 5;")

if [ -z "$SEAT_ID" ]; then
    echo "Error: Could not find the seat in the database. Please check if migrations were applied."
    exit 1
fi

echo "Found test seat with UUID: $SEAT_ID"

echo "Step 0.5: Clearing any old Redis lock for this seat just in case..."
# The key is formed according to our new pattern using the seat UUID
docker exec -i redis_app_tickets redis-cli -a secure_password DEL "event:$EVENT_ID:seat:$SEAT_ID:lock" > /dev/null

echo "Step 1: Reserving the seat in Redis (/reserve)..."
RESERVE_RES=$(curl -s -X POST "http://localhost:$APPLICATION_PORT/events/$EVENT_ID/reserve" \
  -H "Content-Type: application/json" \
  -d "{\"seat_id\": \"$SEAT_ID\", \"user_id\": \"$USER_ID\"}")

echo "Response from /reserve: $RESERVE_RES"

# Extract reservation_id from the JSON response
RESERVATION_ID=$(echo $RESERVE_RES | grep -o '"reservation_id":"[^"]*' | grep -o '[^"]*$')

if [ -z "$RESERVATION_ID" ]; then
    echo "Failed to get reservation_id. Please check the tickets-service application logs."
    exit 1
fi

echo "Reservation successfully created in Redis memory! Reservation ID: $RESERVATION_ID"
echo "Step 2: Submitting the order for asynchronous payment (/checkout)..."

curl -i -X POST "http://localhost:$APPLICATION_PORT/orders/checkout" \
  -H "Content-Type: application/json" \
  -d "{\"event_id\": \"$EVENT_ID\", \"seat_id\": \"$SEAT_ID\", \"reservation_id\": \"$RESERVATION_ID\", \"user_id\": \"$USER_ID\", \"amount_cents\": 15000}"

echo -e "\n\n🚀 End-to-end pipeline started! Check the terminal with the Background Worker and PostgreSQL DBMS."
