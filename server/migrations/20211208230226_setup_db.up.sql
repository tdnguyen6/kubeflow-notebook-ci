-- Add up migration script here
CREATE TABLE IF NOT EXISTS ci_jobs (
  id SERIAL PRIMARY KEY,
  git VARCHAR(50) UNIQUE,
  log TEXT,
  last_log TEXT,
  digest VARCHAR(50),
  last_digest VARCHAR(50),
  update_time TIMESTAMPTZ,
  last_update_time TIMESTAMPTZ,
  updating BOOLEAN DEFAULT FALSE
);
CREATE TABLE IF NOT EXISTS notebooks (
  id SERIAL PRIMARY KEY,
  namespace VARCHAR(50) UNIQUE,
  name VARCHAR(50),
  repo_id INT,
  image VARCHAR(50),
  FOREIGN KEY(repo_id) REFERENCES ci_jobs(id) ON DELETE CASCADE
);
CREATE UNIQUE INDEX idx_1_name_per_ns ON notebooks (namespace, name);
