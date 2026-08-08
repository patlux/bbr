-- A legacy pre-v2 store: no version stamp (user_version 0) but real data.
-- The write path discards it as StoreDiscardReason::Unstamped and recreates
-- an empty v2 store.
CREATE TABLE profiles (legacy_value TEXT NOT NULL);
INSERT INTO profiles VALUES ('preserve me');
CREATE TABLE marker (value TEXT NOT NULL);
INSERT INTO marker VALUES ('unchanged');
PRAGMA user_version = 0;
