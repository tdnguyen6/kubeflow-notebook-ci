-- Add up migration script here
CREATE TABLE IF NOT EXISTS ci_jobs (
  id SERIAL PRIMARY KEY,
  uri VARCHAR(255),
  secret_namespace VARCHAR(255),
  private_repo BOOLEAN,
  repo_credential_secret VARCHAR(255),
  build_log TEXT,
  last_build_log TEXT,
  track_log TEXT,
  last_track_log TEXT,
  dockerfile VARCHAR(255) NOT NULL DEFAULT 'Dockerfile',
  digest VARCHAR(255),
  last_digest VARCHAR(255),
  updating BOOLEAN DEFAULT FALSE,
  should_track BOOLEAN DEFAULT TRUE
);
CREATE TABLE IF NOT EXISTS notebooks (
  id SERIAL PRIMARY KEY,
  namespace VARCHAR(255),
  name VARCHAR(255),
  repo_id INT,
  image VARCHAR(255),
  push_log TEXT,
  last_push_log TEXT,
  registry_credential_secret VARCHAR(255),
  private_registry BOOLEAN,
  registry VARCHAR(255),
  syncing BOOLEAN DEFAULT FALSE,
  auto_sync BOOLEAN,
  FOREIGN KEY(repo_id) REFERENCES ci_jobs(id) ON DELETE CASCADE
);
CREATE UNIQUE INDEX idx_1_name_per_ns ON notebooks (namespace, name);
CREATE UNIQUE INDEX idx_1_repo_uri_per_ns_and_dockerfile ON ci_jobs (uri, secret_namespace, dockerfile);
