--- with tz and group by
-- run with pgcli using \i

SELECT 
  pwu.user_name AS username,
  pwu.in_out,
  (pwu.punch_time AT TIME ZONE 'UTC' AT TIME ZONE 'MDT')::date AS punch_date,
  (pwu.punch_time AT TIME ZONE 'UTC' AT TIME ZONE 'MDT')::time AS punch_time,
  COUNT(*) OVER (PARTITION BY pwu.user_id) AS punches_last_15_days
FROM 
  punches_with_user AS pwu
WHERE 
  pwu.punch_time >= NOW() - INTERVAL '15 days'
GROUP BY 
  username, 
  punch_date, 
  punch_time,
  pwu.in_out,
  pwu.user_id
ORDER BY 
  punch_date DESC, 
  punch_time DESC,
  username, 
  pwu.in_out;