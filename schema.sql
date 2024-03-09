
-- make departments table --
CREATE TABLE IF NOT EXISTS departments (
  id SERIAL PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- first run only
-- initialize default department --
-- INSERT INTO departments(name) VALUES ('Default');

-- make users table --
CREATE TABLE IF NOT EXISTS users (
  id SERIAL PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  uuid VARCHAR(36) UNIQUE NOT NULL,
  email VARCHAR(255) NOT NULL,
  dept_id INT REFERENCES departments(id) DEFAULT 1,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);


-- first run only
-- create enum type for punch in/out --
-- CREATE TYPE punch AS ENUM('in', 'out');

-- drop table punch_records; -- careful here...
-- create punch records table --
CREATE TABLE IF NOT EXISTS punch_records (
  id SERIAL PRIMARY KEY,
  user_id INT REFERENCES users(id),
  in_out punch NOT NULL,
  punch_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

