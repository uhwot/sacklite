ALTER TABLE users
  ADD COLUMN location_x integer DEFAULT 0 NOT NULL CHECK (location_x >= 0 AND location_x <= 65535),
  ADD COLUMN location_y integer DEFAULT 0 NOT NULL CHECK (location_y >= 0 AND location_y <= 65535);