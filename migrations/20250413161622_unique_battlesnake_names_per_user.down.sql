-- Remove unique constraint on user_id and name
DROP INDEX IF EXISTS unique_battlesnake_name_per_user;
