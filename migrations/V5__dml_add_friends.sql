CREATE TEMP TABLE friend_pool AS
SELECT id, row_number() OVER (ORDER BY random()) as rn
FROM users
LIMIT 500009;

CREATE INDEX idx_pool_rn ON friend_pool(rn);

INSERT INTO friends (user_id, friend_id)
WITH pairs AS (
    SELECT 
        u.id as uid,
        p.id as fid
    FROM (
        SELECT id, row_number() OVER () as user_rn
        FROM users
    ) u
    JOIN friend_pool p ON 
        p.rn > (u.user_rn * 3 % 499990) AND 
        p.rn <= (u.user_rn * 3 % 499990) + floor(random() * 5 + 1)::int
    WHERE u.id != p.id
)
SELECT uid, fid FROM pairs
UNION 
SELECT fid, uid FROM pairs
ON CONFLICT DO NOTHING; 