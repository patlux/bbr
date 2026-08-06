CREATE TABLE profiles (legacy_value TEXT NOT NULL);
INSERT INTO profiles VALUES ('preserve me');
CREATE TABLE marker (value TEXT NOT NULL);
INSERT INTO marker VALUES ('unchanged');
PRAGMA user_version = 0;
