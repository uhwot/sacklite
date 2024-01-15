ALTER TABLE comments
    ADD COLUMN target_slot bigint REFERENCES slots;