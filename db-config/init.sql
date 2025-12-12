CREATE TABLE users(
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    first_name VARCHAR(255) NOT NULL,
    last_name VARCHAR(255) NOT NULL,
    birthdate DATE,
    biography VARCHAR(255),
    city VARCHAR(255),
    pwd VARCHAR(255)
);

COPY users(last_name, first_name, birthdate, city)
    FROM '/docker-entrypoint-initdb.d/data.csv'
    DELIMITER ',';