-- Add migration script here
-- For all `status` = NULL in subscriptions make it as 'confirmed'
BEGIN;
	UPDATE subscriptions
		SET status = 'confirmed'
		WHERE status is NULL;

	ALTER TABLE subscriptions ALTER COLUMN status SET NOT NULL;
COMMIT;
