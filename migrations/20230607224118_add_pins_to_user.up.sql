ALTER TABLE users
  ADD COLUMN awards bigint[] DEFAULT '{}' NOT NULL,
  ADD COLUMN progress bigint[] DEFAULT '{}' NOT NULL,
  ADD COLUMN profile_pins bigint[3] DEFAULT '{}' NOT NULL;