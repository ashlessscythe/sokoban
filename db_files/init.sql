-- initial_setup.sql
-- here be dragons
-- drop table if exists punches, users, departments cascade;
-- here end dragons

-- Extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Enums
-- Check if the punch enum does not exist before creating it.
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'punch') THEN
        CREATE TYPE punch AS ENUM ('in', 'out');
    END IF;
END $$;

-- Users Table
CREATE TABLE IF NOT EXISTS users (
  user_id VARCHAR(36) PRIMARY KEY DEFAULT uuid_generate_v4(),
  name VARCHAR(255) NOT NULL,
  email VARCHAR(255) NOT NULL,
  dept_id INT DEFAULT 1,
  profile_picture VARCHAR(255),
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Departments Table
CREATE TABLE IF NOT EXISTS departments (
  id SERIAL PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  bossId VARCHAR(36),
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Punches Table
CREATE TABLE IF NOT EXISTS punches (
  id SERIAL PRIMARY KEY,
  user_id VARCHAR(36) REFERENCES users(user_id),
  in_out punch NOT NULL,
  punch_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Insert default department (check if exists before inserting)
INSERT INTO departments (name)
SELECT 'Default'
WHERE NOT EXISTS (SELECT 1 FROM departments WHERE name = 'Default');

-- add relations bossId relates to user_id in users table
-- Check if the foreign key does not exist before adding it.
DO $$ BEGIN
  IF NOT EXISTS (
    SELECT 1
    FROM information_schema.table_constraints 
    WHERE constraint_name = 'departments_bossId_fkey'
  ) THEN
    ALTER TABLE departments
    ADD FOREIGN KEY (bossId) REFERENCES users(user_id);
  END IF;
END $$;

-- Views
-- If you want to recreate the view, you can DROP it first if it exists.
DROP VIEW IF EXISTS punches_with_user;
CREATE VIEW punches_with_user AS
SELECT
  p.id as punch_id,
  p.user_id,
  p.in_out,
  p.punch_time,
  u.name AS user_name
FROM
  punches p
JOIN
  users u ON p.user_id = u.user_id;
