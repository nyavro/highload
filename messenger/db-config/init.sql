CREATE EXTENSION IF NOT EXISTS citus;

SELECT citus_add_node('messenger_worker1', 5432);
SELECT citus_add_node('messenger_worker2', 5432);
