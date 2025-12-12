#!/bin/bash
set -e

PGDATA_PATH="/var/lib/postgresql/data"

echo 'Waiting for master...'
until pg_isready -h postgres-master -p 5432 -U pguser -d highload; do
  sleep 2
done
echo 'Master is available'

if [ ! "$(ls -A $PGDATA_PATH)" ]; then
    echo 'Init replica with pg_basebackup...'
    echo 'Slot is:'$PG_SLOT
    PGPASSWORD=$REPLICATOR_PASSWORD pg_basebackup \
      -h postgres-master \
      -D $PGDATA_PATH \
      -U replicator \
      -P \
      -v \
      -R \
      -X stream \
      -S $PG_SLOT
    echo 'Replica initialization complete'
else
    echo 'Data directory is already exists, skipping pg_basebackup.'
fi

chown -R postgres:postgres $PGDATA_PATH
chmod -R 0700 $PGDATA_PATH

echo 'Launching PostgreSQL server on replica...'
exec gosu postgres postgres -D $PGDATA_PATH
