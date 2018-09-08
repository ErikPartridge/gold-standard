-- Your SQL goes here

CREATE TABLE submissions(
  id SERIAL PRIMARY KEY,
  reference_code VARCHAR(255) NOT NULL,
  name VARCHAR(255) NOT NULL,
  email VARCHAR(255) NOT NULL,
  bio TEXT NOT NULL,
  reference TEXT NOT NULL,
  title TEXT NOT NULL,
  author TEXT NOT NULL,
  category TEXT NOT NULL,
  message TEXT NOT NULL,
  created_at DATE NOT NULL,
  updated_at DATE NOT NULL
);
