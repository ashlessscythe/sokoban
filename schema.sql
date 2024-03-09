-- here be dragons
-- drop table if exists punch_records, users, departments cascade;
-- INSERT INTO departments(name) VALUES ('Default'); -- comment out
-- CREATE TYPE punch AS ENUM('in', 'out'); -- comment out
-- CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
-- here end dragons

-- make departments table --
CREATE TABLE IF NOT EXISTS departments (
  id SERIAL PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- make users table --
CREATE TABLE IF NOT EXISTS users (
  user_id VARCHAR(36) PRIMARY KEY DEFAULT uuid_generate_v4(),
  name VARCHAR(255) NOT NULL,
  email VARCHAR(255) NOT NULL,
  dept_id INT REFERENCES departments(id) DEFAULT 1,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);


-- create punch records table --
CREATE TABLE IF NOT EXISTS punch_records (
  id SERIAL PRIMARY KEY,
  user_id VARCHAR(36) REFERENCES users(user_id),
  in_out punch NOT NULL,
  punch_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

