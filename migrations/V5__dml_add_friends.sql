CREATE TEMP TABLE friend_pool AS
SELECT id, row_number() OVER (ORDER BY random()) as rn
FROM users
LIMIT 500009;

CREATE INDEX idx_pool_rn ON friend_pool(rn);

INSERT INTO friends (user_id, friend_id, status)
(
    SELECT 
        u.id as user_id,
        p.id as friend_id,
        'accepted'
    UNION
    SELECT 
        p.id as user_id,
        u.id as friend_id,
        'accepted'
)
FROM (
    SELECT id, row_number() OVER () as user_rn
    FROM users    
) u
JOIN friend_pool p ON 
    p.rn > (u.user_rn * 3 % 499990) AND 
    p.rn <= (u.user_rn * 3 % 499990) + floor(random() * 5 + 1)::int
WHERE u.id != p.id; 