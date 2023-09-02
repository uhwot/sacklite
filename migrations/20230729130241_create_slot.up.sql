CREATE TABLE slots (
    id bigint GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY NOT NULL,
    name varchar(64) NOT NULL,
    author uuid REFERENCES users(id) NOT NULL,
    published_at timestamp DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp DEFAULT CURRENT_TIMESTAMP NOT NULL,
    mmpicked_at timestamp,
    description varchar(512) DEFAULT '' NOT NULL,
    icon varchar(40) DEFAULT '' NOT NULL,
    gamever smallint NOT NULL,
    root_level char(40) NOT NULL,
    resources varchar(40)[] DEFAULT '{}' NOT NULL,
    location_x integer DEFAULT 0 NOT NULL CHECK (location_x >= 0 AND location_x <= 65535),
    location_y integer DEFAULT 0 NOT NULL CHECK (location_x >= 0 AND location_x <= 65535),
    initially_locked bool DEFAULT FALSE NOT NULL,
    is_sub_level bool DEFAULT FALSE NOT NULL CHECK (gamever != 0 OR is_sub_level = FALSE),
    is_lbp1_only bool DEFAULT FALSE NOT NULL CHECK (gamever = 0 OR is_lbp1_only = FALSE),
    shareable bool DEFAULT FALSE NOT NULL,
    level_type varchar DEFAULT '' NOT NULL,
    min_players smallint DEFAULT 1 NOT NULL CHECK (min_players >= 1 AND min_players <= 4),
    max_players smallint DEFAULT 4 NOT NULL CHECK (max_players >= 1 AND min_players <= 4 AND max_players >= min_players),
    move_required bool DEFAULT FALSE NOT NULL CHECK (gamever != 0 OR move_required = FALSE),
    vita_cc_required bool DEFAULT FALSE NOT NULL CHECK (gamever = 1 OR vita_cc_required = FALSE)
);