CREATE TABLE user (
    id TEXT PRIMARY KEY NOT NULL,
    online_id TEXT NOT NULL UNIQUE,
    psn_id BIGINT UNIQUE,
    rpcn_id BIGINT UNIQUE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL
);