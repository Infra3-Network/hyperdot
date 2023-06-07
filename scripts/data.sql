CREATE TABLE IF NOT EXISTS block (
    block_number bigint NOT NULL,
    block_hash BYTEA NOT NULL,
    parent_hash BYTEA NOT NULL,
    state_root BYTEA NOT NULL,
    extrinsics_root BYTEA NOT NULL,
    -- Additional columns for the digest if needed
    -- digest_column_name data_type,
    PRIMARY KEY (block_number)
);

CREATE INDEX  IF NOT EXISTS  idx_parent_hash ON block (parent_hash);
CREATE INDEX  IF NOT EXISTS  idx_block_hash ON block (block_hash);