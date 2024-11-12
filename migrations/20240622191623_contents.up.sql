-- Add up migration script here
CREATE TABLE IF NOT EXISTS contents (
  id CHAR(36) PRIMARY KEY NOT NULL,
  content_type VARCHAR(15) NOT NULL,
  title VARCHAR(255) NOT NULL UNIQUE,
  summary TEXT DEFAULT NULL,
  details TEXT DEFAULT NULL,
  content_image TEXT DEFAULT NULL,
  record_state TINYINT (1) NOT NULL DEFAULT 0,
  protected TINYINT (1) NOT NULL DEFAULT 0,
  created_by CHAR(36) NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  deleted_at TIMESTAMP NULL,
  CONSTRAINT fk_created_by_content FOREIGN KEY (created_by) REFERENCES users (id)
);

-- Seed data for contents table
INSERT INTO
  contents (
    id,
    content_type,
    title,
    summary,
    details,
    record_state,
    protected,
    created_by,
    created_at,
    updated_at,
    deleted_at
  )
VALUES
  (
    '1f34e48a-d5b1-4bfa-9f10-9345d0a66a1d',
    'page',
    'content-1',
    Null,
    '<p>Lorem ipsum dolor sit amet, officia excepteur ex fugiat reprehenderit enim labore culpa sint ad nisi Lorem pariatur mollit ex esse exercitation amet. Nisi anim cupidatat excepteur officia. Reprehenderit nostrud nostrud ipsum Lorem est aliquip amet voluptate voluptate dolor minim nulla est proident. Nostrud officia pariatur ut officia. Sit irure elit esse ea nulla sunt ex occaecat reprehenderit commodo officia dolor Lorem duis laboris cupidatat officia voluptate. Culpa proident adipisicing id nulla nisi laboris ex in Lorem sunt duis officia eiusmod. Aliqua reprehenderit commodo ex non excepteur duis sunt velit enim. Voluptate laboris sint cupidatat ullamco ut ea consectetur et est culpa et culpa duis.</p>',
    1,
    0,
    'a3f45b67-8c3d-4f8b-9e1f-2b7a3e1c7e2b',
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP,
    NULL
  ),
  (
    '2a34b54e-e2b1-4d6a-91f4-3b5e6f4a7c1e',
    'page',
    'content-2',
    Null,
    '<p>Lorem ipsum dolor sit amet, officia excepteur ex fugiat reprehenderit enim labore culpa sint ad nisi Lorem pariatur mollit ex esse exercitation amet. Nisi anim cupidatat excepteur officia. Reprehenderit nostrud nostrud ipsum Lorem est aliquip amet voluptate voluptate dolor minim nulla est proident. Nostrud officia pariatur ut officia. Sit irure elit esse ea nulla sunt ex occaecat reprehenderit commodo officia dolor Lorem duis laboris cupidatat officia voluptate. Culpa proident adipisicing id nulla nisi laboris ex in Lorem sunt duis officia eiusmod. Aliqua reprehenderit commodo ex non excepteur duis sunt velit enim. Voluptate laboris sint cupidatat ullamco ut ea consectetur et est culpa et culpa duis.</p>',
    1,
    0,
    'a3f45b67-8c3d-4f8b-9e1f-2b7a3e1c7e2b',
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP,
    NULL
  ),
  (
    '3c45e69f-f3b2-4e6c-8f4d-2d6f7e8a9b3d',
    'page',
    'content-3',
    Null,
    '<p>Lorem ipsum dolor sit amet, officia excepteur ex fugiat reprehenderit enim labore culpa sint ad nisi Lorem pariatur mollit ex esse exercitation amet. Nisi anim cupidatat excepteur officia. Reprehenderit nostrud nostrud ipsum Lorem est aliquip amet voluptate voluptate dolor minim nulla est proident. Nostrud officia pariatur ut officia. Sit irure elit esse ea nulla sunt ex occaecat reprehenderit commodo officia dolor Lorem duis laboris cupidatat officia voluptate. Culpa proident adipisicing id nulla nisi laboris ex in Lorem sunt duis officia eiusmod. Aliqua reprehenderit commodo ex non excepteur duis sunt velit enim. Voluptate laboris sint cupidatat ullamco ut ea consectetur et est culpa et culpa duis.</p>',
    1,
    0,
    'a3f45b67-8c3d-4f8b-9e1f-2b7a3e1c7e2b',
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP,
    NULL
  );
