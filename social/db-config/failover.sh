#!/bin/bash

FALLING_NODE_ID=$1
FALLING_HOST=$2

if [ $FALLING_NODE_ID -ne 0 ]; then
    exit 0;
fi

export PGPASSWORD='pgpassword'
psql -h postgres-replica-1 -U pguser -d postgres -c "SELECT pg_promote();"

exit 0;