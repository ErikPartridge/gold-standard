-- Your SQL goes here

CREATE TABLE products (
  id SERIAL PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  author VARCHAR(255) NOT NULL,
  url VARCHAR(255),
  purchase_name VARCHAR(255),
  medium VARCHAR(255) NOT NULL,
  description TEXT NOT NULL,
  source TEXT NOT NULL,
  reasoning TEXT NOT NULL,
  blurb TEXT NOT NULL,
  isbn VARCHAR(13),
  year_of_creation VARCHAR(4) NOT NULL,
  slug VARCHAR(255) NOT NULL,
  flags TEXT[] NOT NULL,
  field_id INTEGER REFERENCES fields(id),
  created_at DATE NOT NULL default now(),
  updated_at DATE NOT NULL default now()
);
