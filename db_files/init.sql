-- initial_setup.sql
-- here be dragons
-- drop table if exists punches, users, departments, admin_users, auth_devices, checklist_status cascade;
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
  user_id VARCHAR(36),
  in_out punch NOT NULL,
  device_id VARCHAR(255) DEFAULT NULL,
  punch_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT punches_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(user_id) ON UPDATE CASCADE
);

-- Insert default department (check if exists before inserting)
INSERT INTO departments (name)
SELECT 'Dept1'
WHERE NOT EXISTS (SELECT 1 FROM departments WHERE name = 'Default');

-- Insert department 2 (check if exists before inserting)
INSERT INTO departments (id, name)
SELECT 2, 'Dept2'
WHERE NOT EXISTS (SELECT 1 FROM departments WHERE id = 2);

-- Insert department 3 (check if exists before inserting)
INSERT INTO departments (id, name)
SELECT 3, 'Dept3'
WHERE NOT EXISTS (SELECT 1 FROM departments WHERE id = 3);

-- Insert department 4 (check if exists before inserting)
INSERT INTO departments (id, name)
SELECT 4, 'Dept4'
WHERE NOT EXISTS (SELECT 1 FROM departments WHERE id = 4);

-- Checklist Status Table
CREATE TABLE IF NOT EXISTS checklist_status (
  id SERIAL PRIMARY KEY,
  user_id INTEGER NOT NULL,
  drill_id INTEGER NOT NULL,
  found BOOLEAN DEFAULT FALSE,
  check_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT checklist_status_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(user_id) ON UPDATE CASCADE,
  CONSTRAINT checklist_status_user_id_drill_id_key UNIQUE (user_id, drill_id)
);

-- Admin Users Table
CREATE TABLE IF NOT EXISTS admin_users (
  id SERIAL PRIMARY KEY,
  user_id VARCHAR(36) NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT admin_users_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(user_id) ON UPDATE CASCADE
);


-- auth devices table
CREATE TABLE IF NOT EXISTS auth_devices (
  id SERIAL PRIMARY KEY,
  device_id VARCHAR(255) NOT NULL,
  device_name VARCHAR(255) Default 'No Name',
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- registrations table
CREATE TABLE IF NOT EXISTS registrations (
  id SERIAL PRIMARY KEY,
  name VARCHAR(36) NOT NULL,
  email VARCHAR(255) NOT NULL,
  device_id VARCHAR(255) DEFAULT 'No Device ID',
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

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
  IF NOT EXISTS (
    SELECT 1
    FROM information_schema.table_constraints 
    WHERE constraint_name = 'fk_dept_id'
  ) THEN
    ALTER TABLE users
    ADD CONSTRAINT fk_dept_id
    FOREIGN KEY (dept_id)
    REFERENCES departments(id);
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


