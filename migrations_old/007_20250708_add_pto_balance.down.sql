-- Remove PTO balance fields from users table
ALTER TABLE
    users DROP COLUMN pto_balance_hours;

ALTER TABLE
    users DROP COLUMN sick_balance_hours;

ALTER TABLE
    users DROP COLUMN personal_balance_hours;

ALTER TABLE
    users DROP COLUMN pto_accrual_rate;

ALTER TABLE
    users DROP COLUMN hire_date;

ALTER TABLE
    users DROP COLUMN last_accrual_date;

-- Drop PTO balance history table
DROP TABLE IF EXISTS pto_balance_history;
