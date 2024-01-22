CREATE TABLE favourite_slots (
    user_id uuid NOT NULL REFERENCES users ON DELETE CASCADE,
    slot_id bigint NOT NULL REFERENCES slots ON DELETE CASCADE,
    timestamp timestamp DEFAULT CURRENT_TIMESTAMP NOT NULL,
    PRIMARY KEY (user_id, slot_id)
);