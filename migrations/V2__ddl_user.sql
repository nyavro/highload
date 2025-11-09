DROP TABLE users;
CREATE TABLE users(
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    first_name VARCHAR(255) NOT NULL,
    second_name VARCHAR(255) UNIQUE NOT NULL,
    birthdate DATE,
    biography VARCHAR(255),
    city VARCHAR(255),
    pwd VARCHAR(255) NOT NULL
);