-- Add up migration script here
CREATE TABLE IF NOT EXISTS ci_jobs (
  id SERIAL PRIMARY KEY,
  uri VARCHAR(255),
  secret_namespace VARCHAR(50),
  private_repo BOOLEAN,
  repo_credential_secret VARCHAR(255),
  build_log TEXT,
  last_build_log TEXT,
  track_log TEXT,
  last_track_log TEXT,
  digest VARCHAR(50),
  last_digest VARCHAR(50),
  updating BOOLEAN DEFAULT FALSE,
  should_track BOOLEAN DEFAULT TRUE
);
CREATE TABLE IF NOT EXISTS notebooks (
  id SERIAL PRIMARY KEY,
  namespace VARCHAR(50),
  name VARCHAR(50),
  repo_id INT,
  image VARCHAR(50),
  registry_credential_secret VARCHAR(255),
  private_registry BOOLEAN,
  registry VARCHAR(255),
  auto_sync BOOLEAN,
  FOREIGN KEY(repo_id) REFERENCES ci_jobs(id) ON DELETE CASCADE
);
CREATE UNIQUE INDEX idx_1_name_per_ns ON notebooks (namespace, name);
CREATE UNIQUE INDEX idx_1_repo_uri_per_ns ON ci_jobs (uri, secret_namespace);
