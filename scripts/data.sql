CREATE TABLE IF NOT EXISTS block_header (
    block_number bigint NOT NULL,
    block_hash bit(256) NOT NULL,
    parent_hash bit(256) NOT NULL,
    state_root bit(256) NOT NULL,
    extrinsics_root bit(256) NOT NULL,
    -- Additional columns for the digest if needed
    -- digest_column_name data_type,
    PRIMARY KEY (block_number)
);

CREATE INDEX  IF NOT EXISTS  idx_parent_hash ON block_header (parent_hash);
CREATE INDEX  IF NOT EXISTS  idx_block_hash ON block_header (block_hash);