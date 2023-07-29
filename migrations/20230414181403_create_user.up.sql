CREATE TABLE users (
    id uuid PRIMARY KEY NOT NULL,
    online_id varchar(16) NOT NULL UNIQUE,
    psn_id numeric UNIQUE,
    rpcn_id numeric UNIQUE,
    created_at timestamp DEFAULT CURRENT_TIMESTAMP NOT NULL
);