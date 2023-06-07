ALTER TABLE users
  ADD COLUMN icon varchar(40) DEFAULT '' NOT NULL,
  ADD COLUMN lbp2_planets varchar(40) DEFAULT '' NOT NULL,
  ADD COLUMN lbp3_planets varchar(40) DEFAULT '' NOT NULL,
  ADD COLUMN cross_control_planet varchar(40) DEFAULT '' NOT NULL,
  ADD COLUMN yay2 varchar(40) DEFAULT '' NOT NULL,
  ADD COLUMN meh2 varchar(40) DEFAULT '' NOT NULL,
  ADD COLUMN boo2 varchar(40) DEFAULT '' NOT NULL;