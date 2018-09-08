-- Your SQL goes here

CREATE TABLE fields (
  id SERIAL PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  synonyms TEXT[] NOT NULL,
  created_at DATE NOT NULL default now(),
  updated_at DATE NOT NULL default now()
);
