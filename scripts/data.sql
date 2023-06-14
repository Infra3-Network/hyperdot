DROP TABLE block;
DROP TABLE raw_event;
DROP TABLE transfer;
DROP TABLE withdraw;

CREATE TABLE IF NOT EXISTS block (
    block_number bigint NOT NULL,
    block_hash TEXT NOT NULL,
    parent_hash TEXT NOT NULL,
    state_root TEXT NOT NULL,
    extrinsics_root TEXT NOT NULL,
    -- Additional columns for the digest if needed
    -- digest_column_name data_type,
    PRIMARY KEY (block_number)
);

CREATE INDEX  IF NOT EXISTS  idx_parent_hash ON block (parent_hash);
CREATE INDEX  IF NOT EXISTS  idx_block_hash ON block (block_hash);


CREATE TABLE IF NOT EXISTS raw_event (
    block_number bigint NOT NULL,
    block_hash BYTEA NOT NULL,
    block_time bigint,
    extrinsic_hash BYTEA NOT NULL,
    data BYTEA,
    "index" INT NOT NULL,
    topic0 BYTEA,
    topic1 BYTEA,
    topic2 BYTEA,
    topic3 BYTEA,
    topic4 BYTEA,
    phase TEXT NOT NULL,
    pallet_name TEXT NOT NULL,
    pallet_index SMALLINT NOT NULL,
    event_name TEXT NOT NULL,
    PRIMARY KEY (block_number, index)
);

CREATE INDEX  IF NOT EXISTS  index_raw_event_block_index ON raw_event (block_number);
CREATE INDEX  IF NOT EXISTS  index_raw_event_block_hash ON raw_event (block_hash);


CREATE TABLE IF NOT EXISTS transfer (
    block_number BIGINT NOT NULL,
    block_hash BYTEA NOT NULL,
    "index" INT NOT NULL,
    pallet_index SMALLINT NOT NULL,
    pallet_name TEXT NOT NULL,
    event_name TEXT NOT NULL,
    extrinsic_hash BYTEA NOT NULL,
    "from" BYTEA NOT NULL,
    "to" BYTEA NOT NULL,
    amount NUMERIC(38, 0) NOT NULL,
    success BOOLEAN NOT NULL,
    PRIMARY KEY (block_number, index)
);

CREATE INDEX  IF NOT EXISTS  index_raw_event_block_index ON raw_event (block_number);
CREATE INDEX  IF NOT EXISTS  index_transfer_block_hash ON transfer (block_hash);


CREATE TABLE IF NOT EXISTS withdraw (
    block_number bigint NOT NULL,
    block_hash BYTEA NOT NULL,
    "index" INT NOT NULL,
    pallet_index SMALLINT NOT NULL,
    pallet_name TEXT NOT NULL,
    event_name TEXT NOT NULL,
    extrinsic_hash BYTEA NOT NULL,
    who BYTEA NOT NULL,
    amount NUMERIC(38, 0) NOT NULL,
    success BOOLEAN NOT NULL,
    PRIMARY KEY (block_number, index)
);

CREATE INDEX  IF NOT EXISTS  index_raw_event_block_index ON raw_event (block_number);
CREATE INDEX  IF NOT EXISTS  index_withdraw_block_hash ON withdraw (block_hash);



