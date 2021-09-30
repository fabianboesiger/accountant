DROP TABLE pairs;
DROP TABLE positions;
DROP TABLE trades;

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

INSERT INTO trades (exchange, market, side, size, price, date, bot)
VALUES ('1', '2', 'BUY', 0, 0, NOW(), 'c')
RETURNING id;