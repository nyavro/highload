#!/bin/bash

docker exec -it redis_app_tickets redis-cli -a secure_password TTL event:00000000-0000-0000-0000-000000000001:seat:seat_5:lock
