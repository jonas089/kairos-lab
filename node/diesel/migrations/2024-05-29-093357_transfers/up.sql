-- Your SQL goes here
CREATE TYPE TrxType AS ENUM ('Transfer', 'Withdrawal', 'Deposit');
CREATE TABLE transactions (
    "timestamp" timestamp DEFAULT CURRENT_TIMESTAMP,
    public_key varchar NOT NULL,
    nonce bigint NOT NULL,
    trx TrxType NOT NULL,
    amount bigint NOT NULL,
    recipient varchar,
    PRIMARY KEY ("timestamp", amount, public_key)
);