CREATE TABLE texts_tmp(    
    text VARCHAR NOT NULL
);

COPY texts_tmp FROM '/docker-entrypoint-initdb.d/posts.txt';

ALTER TABLE texts_tmp ADD COLUMN id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY;

INSERT INTO posts (user_id, text)
SELECT 
    u.id, 
    t.text
FROM (
    SELECT id, (row_number() OVER () % (SELECT count(*) FROM texts_tmp)) + 1 as text_id
    FROM users    
) u
JOIN texts_tmp t ON t.id = u.text_id
WHERE u.text_id % 2 = 1;

INSERT INTO posts (user_id, text)
SELECT 
    u.id, 
    t.text
FROM (
    SELECT id, (row_number() OVER () % (SELECT count(*) FROM texts_tmp)) + 1 as text_id
    FROM users    
) u
JOIN texts_tmp t ON t.id = u.text_id
WHERE u.text_id % 3 = 1;

DROP TABLE texts_tmp;