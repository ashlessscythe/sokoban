-- simple table with list of punches
-- run with pgcli using \i

SELECT 
  u.name AS username,
  p.in_out,
  p.punch_time::date AS punch_date,
  p.punch_time::time as punch_time
FROM 
  users AS u
JOIN 
  punches AS p ON u.user_id = p.user_id
GROUP BY 
  username, 
  punch_date, 
  punch_time,
  p.in_out
ORDER BY 
  punch_date DESC, 
  punch_time DESC,
  username, 
  p.in_out;
