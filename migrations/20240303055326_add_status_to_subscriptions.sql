-- Add migration script here
-- status col keeping it as optional
ALTER TABLE subscriptions
ADD COLUMN status TEXT NULL;
