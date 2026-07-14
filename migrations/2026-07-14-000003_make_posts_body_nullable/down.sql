-- Drop the empty-string constraint before altering the column
ALTER TABLE posts DROP CONSTRAINT IF EXISTS posts_body_not_empty;

-- Restore custom default of empty string
ALTER TABLE posts ALTER COLUMN body SET DEFAULT '';

-- Transform NULL into empty strings
UPDATE posts SET body = '' WHERE body IS NULL;

-- Disallow nulls
ALTER TABLE posts ALTER COLUMN body SET NOT NULL;
