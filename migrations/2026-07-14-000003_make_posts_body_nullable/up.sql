-- Allow nulls
ALTER TABLE posts ALTER COLUMN body DROP NOT NULL;

-- Transform empty strings into NULL
UPDATE posts SET body = NULL WHERE body = '';

-- Drop custom default of empty string
ALTER TABLE posts ALTER COLUMN body DROP DEFAULT;

-- Reject empty strings when body is provided
ALTER TABLE posts ADD CONSTRAINT posts_body_not_empty CHECK (body IS NULL OR body <> '');
