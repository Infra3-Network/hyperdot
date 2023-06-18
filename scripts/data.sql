DROP TABLE block;
DROP TABLE raw_event;
DROP TABLE transfer;
DROP TABLE withdraw;

CREATE TABLE IF NOT EXISTS blocks (
    "number" bigint NOT NULL,
    "hash" TEXT NOT NULL,
    parent_hash TEXT NOT NULL,
    extrinsics_root TEXT NOT NULL,
    state_root TEXT NOT NULL,
    hash_bytes BYTEA NOT NULL,
    parent_hash_bytes BYTEA NOT NULL,
    extrinsics_root_bytes BYTEA NOT NULL,
    state_root_bytes BYTEA NOT NULL,
    PRIMARY KEY (number)
);

CREATE INDEX  IF NOT EXISTS  idx_blocks_hash ON blocks ("hash");
CREATE INDEX  IF NOT EXISTS  idx_blocks_parent_hash ON blocks (parent_hash);


CREATE TABLE IF NOT EXISTS extrinsics (
    block_number bigint NOT NULL,
    block_hash TEXT NOT NULL,
    "index" INT NOT NULL,
    is_signed BOOLEAN NOT NULL,
    pallet_name TEXT,
    variant_name TEXT,
    pallet_index SMALLINT NOT NULL,
    variant_index SMALLINT NOT NULL,
    signed_address TEXT,
    block_hash_bytes BYTEA NOT NULL,
    signed_address_bytes BYTEA
);

CREATE UNIQUE INDEX  IF NOT EXISTS  index_extrinsics ON extrinsics (block_number, "index");
CREATE INDEX  IF NOT EXISTS  index_extrinsics_block_hash ON extrinsics (block_hash);



CREATE TABLE IF NOT EXISTS raw_event (
    block_number bigint NOT NULL,
    block_hash TEXT NOT NULL,
    block_time bigint,
    extrinsic_hash TEXT NOT NULL,
    data TEXT,
    "index" INT NOT NULL,
    topic0 TEXT,
    topic1 TEXT,
    topic2 TEXT,
    topic3 TEXT,
    topic4 TEXT,
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
    block_hash TEXT NOT NULL,
    "index" INT NOT NULL,
    pallet_index SMALLINT NOT NULL,
    pallet_name TEXT NOT NULL,
    event_name TEXT NOT NULL,
    extrinsic_hash TEXT NOT NULL,
    "from" TEXT NOT NULL,
    "to" TEXT NOT NULL,
    amount NUMERIC(38, 0) NOT NULL,
    success BOOLEAN NOT NULL,
    PRIMARY KEY (block_number, index)
);

CREATE INDEX  IF NOT EXISTS  index_transfer_block_number ON raw_event (block_number);
CREATE INDEX  IF NOT EXISTS  index_transfer_block_hash ON transfer (block_hash);


CREATE TABLE IF NOT EXISTS withdraw (
    block_number bigint NOT NULL,
    block_hash TEXT NOT NULL,
    "index" INT NOT NULL,
    pallet_index SMALLINT NOT NULL,
    pallet_name TEXT NOT NULL,
    event_name TEXT NOT NULL,
    extrinsic_hash TEXT NOT NULL,
    who TEXT NOT NULL,
    amount NUMERIC(38, 0) NOT NULL,
    success BOOLEAN NOT NULL,
    PRIMARY KEY (block_number, index)
);

CREATE INDEX  IF NOT EXISTS  index_withdraw_block_number ON raw_event (block_number);
CREATE INDEX  IF NOT EXISTS  index_withdraw_block_hash ON withdraw (block_hash);



