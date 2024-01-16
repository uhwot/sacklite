ALTER TABLE slots
    DROP CONSTRAINT slots_author_fkey,
    ADD CONSTRAINT slots_author_fkey
        FOREIGN KEY (author)
            REFERENCES users(id);

ALTER TABLE comments
    DROP CONSTRAINT comments_author_fkey,
    ADD CONSTRAINT comments_author_fkey
        FOREIGN KEY (author)
            REFERENCES users(id),
    DROP CONSTRAINT comments_deleted_by_fkey,
    ADD CONSTRAINT comments_deleted_by_fkey
        FOREIGN KEY (deleted_by)
            REFERENCES users(id),
    DROP CONSTRAINT comments_target_user_fkey,
    ADD CONSTRAINT comments_target_user_fkey
        FOREIGN KEY (target_user)
            REFERENCES users(id),
    DROP CONSTRAINT comments_target_slot_fkey,
    ADD CONSTRAINT comments_target_slot_fkey
        FOREIGN KEY (target_slot)
            REFERENCES slots(id);