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

UPDATE users 
SET pwd='$argon2id$v=19$m=19456,t=2,p=1$4xL9Oxd2iri+P/wnq8euQA$8B5A0lAEfgbzz2Iocgs9haEI7vnB0y4aq084gs9SmCA';
