/* Drop all tables */
DROP TABLE payments;
DROP TABLE accounts;
DROP TYPE payment_type;

DROP TABLE pairs;
DROP TABLE positions;
DROP TABLE trades;
DROP TYPE side;

/* Init database */
CREATE TYPE side AS ENUM ('BUY', 'SELL');

CREATE TABLE trades (
    id BIGSERIAL PRIMARY KEY,
    side side NOT NULL,
    size DECIMAL NOT NULL,
    price DECIMAL NOT NULL,
    date TIMESTAMP WITH TIME ZONE NOT NULL,
    exchange VARCHAR(16) NOT NULL,
    market VARCHAR(16) NOT NULL,
    bot VARCHAR(16)
);

CREATE TABLE positions (
    id BIGSERIAL PRIMARY KEY,
    enter BIGINT NOT NULL REFERENCES trades(id),
    exit BIGINT NOT NULL REFERENCES trades(id)
);

CREATE TABLE pairs (
    id BIGSERIAL PRIMARY KEY,
    long BIGINT NOT NULL REFERENCES positions(id),
    short BIGINT NOT NULL REFERENCES positions(id)
);

CREATE TABLE accounts (
    id BIGSERIAL PRIMARY KEY,
    full_name VARCHAR(32) NOT NULL,
    balance DECIMAL NOT NULL,
    fee DECIMAL NOT NULL
);

CREATE TYPE payment_type AS ENUM ('TRANSFER', 'INVESTMENT', 'FEE');

CREATE TABLE payments (
    id BIGSERIAL PRIMARY KEY,
    account BIGINT NOT NULL REFERENCES accounts(id),
    amount DECIMAL NOT NULL,
    date TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    payment_type payment_type NOT NULL
);


