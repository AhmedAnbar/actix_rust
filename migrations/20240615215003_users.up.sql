-- Add up migration script here
CREATE TABLE IF NOT EXISTS users (
  id CHAR(36) PRIMARY KEY NOT NULL,
  name VARCHAR(255) NOT NULL UNIQUE,
  mobile VARCHAR(15) NOT NULL UNIQUE,
  mobile_token VARCHAR(5),
  mobile_token_expire_at TIMESTAMP,
  email VARCHAR(255) UNIQUE,
  gender VARCHAR(10),
  role_id INT NOT NULL DEFAULT 2,
  active TINYINT (1) NOT NULL DEFAULT 0,
  protected TINYINT (1) NOT NULL DEFAULT 0,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);

-- Sample seed data for users table
INSERT INTO
  users (
    id,
    name,
    mobile,
    mobile_token,
    email,
    gender,
    role_id,
    active,
    protected
  )
VALUES
  (
    'a3f45b67-8c3d-4f8b-9e1f-2b7a3e1c7e2b',
    'Ahmed Anbar',
    '+201018898522',
    '12345',
    'ahmed.anbar@example.com',
    'male',
    1,
    1,
    1
  ),
  (
    'a3f45b67-8c3d-4f8b-9e1f-2b7a3e1c7e22',
    'Mohamed Anbar',
    '098765432109',
    '12345',
    'mohamed.nabar@example.com',
    'male',
    1,
    0,
    0
  ),
  (
    'a3f45b67-8c3d-4f8b-9e1f-2b7a3e1c7e24',
    'Lamar Anbar',
    '555123456789',
    '12345',
    'lamar.anbar@example.com',
    'female',
    2,
    1,
    0
  ),
  (
    'a3f45b67-8c3d-4f8b-9e1f-2b7a3e1c7e27',
    'Maha Atef',
    '444987654321',
    '12345',
    'maha.atef@example.com',
    'female',
    2,
    0,
    0
  ),
  (
    'a3f45b67-8c3d-4f8b-9e1f-2b7a3e1c7e25',
    'May Atef',
    '333456789012',
    '12345',
    'may.atef@example.com',
    'female',
    2,
    1,
    1
  );
