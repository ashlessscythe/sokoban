-- anytime.sql

-- Insert default department (check if exists before inserting)
INSERT INTO departments (name)
SELECT 'Default'
WHERE NOT EXISTS (SELECT 1 FROM departments WHERE name = 'Default');

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

-- Inserting data, updating data, or any other operation that can be run multiple times.

