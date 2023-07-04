DROP TABLE IF EXISTS block_logs;
CREATE TABLE IF NOT EXISTS block_logs (
    id TEXT NOT NULL,
    block_number bigint NOT NULL,
    "type" TEXT,
    "data" TEXT,
    engine TEXT
);
CREATE UNIQUE INDEX  IF NOT EXISTS  unique_index_block_logs ON block_logs (id);

DROP TABLE IF EXISTS extrinsics;
CREATE TABLE IF NOT EXISTS extrinsics (
    id TEXT NOT NULL,
    block_number BIGINT NOT NULL,
    extrinsic_hash TEXT NOT NULL,
    is_signed BOOLEAN NOT NULL,
    mod_name TEXT,
    call_name TEXT,
    result BOOLEAN,
    call_params JSON,
    extrinsic_hash_bytes BYTEA NOT NULL
);
CREATE UNIQUE INDEX  IF NOT EXISTS  unique_index_extrinsics ON extrinsics (id);


DROP TABLE IF EXISTS events;
CREATE TABLE IF NOT EXISTS events (
    id TEXT NOT NULL,
    block_number BIGINT NOT NULL,
    extrinsic_id TEXT,
    mod_name TEXT,
    event_name TEXT,
    phase SMALLINT NOT NULL,
    values JSON
);
CREATE UNIQUE INDEX  IF NOT EXISTS  unique_index_events ON events (id);



DROP TABLE IF EXISTS blocks;
CREATE TABLE IF NOT EXISTS blocks (
    "number" bigint NOT NULL,
    "timestamp" BIGINT,
    "hash" TEXT NOT NULL,
    parent_hash TEXT NOT NULL,
    extrinsics_root TEXT NOT NULL,
    state_root TEXT NOT NULL,
    is_finalized BOOLEAN NOT NULL,
    validator TEXT,
    spec_version INT NOT NULL,

    hash_bytes BYTEA NOT NULL,
    parent_hash_bytes BYTEA NOT NULL,
    extrinsics_root_bytes BYTEA NOT NULL,
    state_root_bytes BYTEA NOT NULL,
    validator_bytes BYTEA,
    PRIMARY KEY (number)
);

CREATE INDEX  IF NOT EXISTS  idx_blocks_hash ON blocks ("hash");
CREATE INDEX  IF NOT EXISTS  idx_blocks_parent_hash ON blocks (parent_hash);


-- CREATE TABLE IF NOT EXISTS raw_event (
--     block_number bigint NOT NULL,
--     block_hash TEXT NOT NULL,
--     block_time bigint,
--     extrinsic_hash TEXT NOT NULL,
--     data TEXT,
--     "index" INT NOT NULL,
--     topic0 TEXT,
--     topic1 TEXT,
--     topic2 TEXT,
--     topic3 TEXT,
--     topic4 TEXT,
--     phase TEXT NOT NULL,
--     pallet_name TEXT NOT NULL,
--     pallet_index SMALLINT NOT NULL,
--     event_name TEXT NOT NULL,
--     PRIMARY KEY (block_number, index)
-- );

-- CREATE INDEX  IF NOT EXISTS  index_raw_event_block_index ON raw_event (block_number);
-- CREATE INDEX  IF NOT EXISTS  index_raw_event_block_hash ON raw_event (block_hash);


-- CREATE TABLE IF NOT EXISTS transfer (
--     block_number BIGINT NOT NULL,
--     block_hash TEXT NOT NULL,
--     "index" INT NOT NULL,
--     pallet_index SMALLINT NOT NULL,
--     pallet_name TEXT NOT NULL,
--     event_name TEXT NOT NULL,
--     extrinsic_hash TEXT NOT NULL,
--     "from" TEXT NOT NULL,
--     "to" TEXT NOT NULL,
--     amount NUMERIC(38, 0) NOT NULL,
--     success BOOLEAN NOT NULL,
--     PRIMARY KEY (block_number, index)
-- );

-- CREATE INDEX  IF NOT EXISTS  index_transfer_block_number ON raw_event (block_number);
-- CREATE INDEX  IF NOT EXISTS  index_transfer_block_hash ON transfer (block_hash);


-- CREATE TABLE IF NOT EXISTS withdraw (
--     block_number bigint NOT NULL,
--     block_hash TEXT NOT NULL,
--     "index" INT NOT NULL,
--     pallet_index SMALLINT NOT NULL,
--     pallet_name TEXT NOT NULL,
--     event_name TEXT NOT NULL,
--     extrinsic_hash TEXT NOT NULL,
--     who TEXT NOT NULL,
--     amount NUMERIC(38, 0) NOT NULL,
--     success BOOLEAN NOT NULL,
--     PRIMARY KEY (block_number, index)
-- );

-- CREATE INDEX  IF NOT EXISTS  index_withdraw_block_number ON raw_event (block_number);
-- CREATE INDEX  IF NOT EXISTS  index_withdraw_block_hash ON withdraw (block_hash);



