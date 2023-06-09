CREATE TABLE comments (
    id bigserial PRIMARY KEY NOT NULL,
    author uuid REFERENCES users(id) NOT NULL,
    posted_at timestamp DEFAULT CURRENT_TIMESTAMP NOT NULL,
    target_user uuid REFERENCES users(id),
    content varchar NOT NULL,
    deleted_by uuid REFERENCES users(id),
    deleted_by_mod bool DEFAULT false NOT NULL
);