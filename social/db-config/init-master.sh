#!/bin/bash
set -e

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL    
    CREATE USER replicator WITH REPLICATION ENCRYPTED PASSWORD 'replicator_password';        
EOSQL

echo "Initializing replication slots..."  
if [ -n "$PG_REPLICA_SLOTS" ]; then
    IFS=',' read -ra SLOTS <<< "$PG_REPLICA_SLOTS"
    for slot in "${SLOTS[@]}"; do
        echo "Creating slot: $slot"
        psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" -c "SELECT pg_create_physical_replication_slot('$slot');"
    done
else
    echo "Variable PG_REPLICA_SLOTS is not set. No replication slots created"
fi
echo "...done initializing replication slots"

echo "listen_addresses = '*'" >> /var/lib/postgresql/data/postgresql.conf

echo 'host replication replicator all md5' >> /var/lib/postgresql/data/pg_hba.conf
echo 'host all         all        0.0.0.0/0 md5' >> /var/lib/postgresql/data/pg_hba.conf
echo "Master setup complete."

