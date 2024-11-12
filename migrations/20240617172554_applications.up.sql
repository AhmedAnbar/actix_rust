-- Add up migration script here
CREATE TABLE applications (
  id CHAR(36) PRIMARY KEY NOT NULL,
  app_name VARCHAR(255) NOT NULL UNIQUE,
  app_version VARCHAR(255) NOT NULL DEFAULT '1',
  app_key VARCHAR(255) NOT NULL UNIQUE,
  app_secret VARCHAR(255) NOT NULL UNIQUE,
  app_requests BIGINT NOT NULL DEFAULT 0,
  record_state TINYINT (1) NOT NULL DEFAULT 0,
  protected TINYINT (1) NOT NULL DEFAULT 0,
  created_by CHAR(36) NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  deleted_at TIMESTAMP NULL,
  CONSTRAINT fk_created_by_applications FOREIGN KEY (created_by) REFERENCES users (id)
);

INSERT INTO
  applications (
    id,
    app_name,
    app_version,
    app_key,
    app_secret,
    app_requests,
    record_state,
    protected,
    created_by,
    created_at,
    updated_at,
    deleted_at
  )
VALUES
  (
    '1a2b3c4d-5e6f-7g8h-9i0j-1k2l3m4n5o6p',
    'actix.test',
    '1',
    'fe1zev3u5aubxn46j71aijfy5h44wgq882hjunw6qc',
    'k1iw4eh4yeswu96eplhzv3gsviydawbos3rorvuz33j',
    1,
    1,
    0,
    'a3f45b67-8c3d-4f8b-9e1f-2b7a3e1c7e2b',
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP,
    NULL
  );
