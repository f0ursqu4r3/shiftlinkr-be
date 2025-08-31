-- Drop subscription and payment management system
-- Drop indexes
DROP INDEX IF EXISTS idx_user_company_is_owner;

DROP INDEX IF EXISTS idx_companies_active;

DROP INDEX IF EXISTS idx_companies_subscription_plan_id;

DROP INDEX IF EXISTS idx_companies_subscription_status;

DROP INDEX IF EXISTS idx_companies_stripe_customer_id;

DROP INDEX IF EXISTS idx_invoices_subscription_id;

DROP INDEX IF EXISTS idx_invoices_status;

DROP INDEX IF EXISTS idx_invoices_stripe_id;

DROP INDEX IF EXISTS idx_invoices_company_id;

DROP INDEX IF EXISTS idx_payment_methods_default;

DROP INDEX IF EXISTS idx_payment_methods_stripe_id;

DROP INDEX IF EXISTS idx_payment_methods_company_id;

DROP INDEX IF EXISTS idx_company_subscriptions_plan_id;

DROP INDEX IF EXISTS idx_company_subscriptions_status;

DROP INDEX IF EXISTS idx_company_subscriptions_stripe_subscription_id;

DROP INDEX IF EXISTS idx_company_subscriptions_stripe_customer_id;

DROP INDEX IF EXISTS idx_company_subscriptions_company_id;

DROP INDEX IF EXISTS idx_subscription_plans_active;

DROP INDEX IF EXISTS idx_subscription_plans_stripe_price_id;

-- Remove subscription-related fields from companies table
ALTER TABLE companies
DROP COLUMN IF EXISTS stripe_customer_id,
DROP COLUMN IF EXISTS subscription_status,
DROP COLUMN IF EXISTS subscription_plan_id,
DROP COLUMN IF EXISTS trial_ends_at,
DROP COLUMN IF EXISTS subscription_ends_at,
DROP COLUMN IF EXISTS max_users,
DROP COLUMN IF EXISTS is_active;

-- Remove owner role from user_company table
ALTER TABLE user_company
DROP COLUMN IF EXISTS is_owner;

-- Drop tables in reverse order (due to foreign key constraints)
DROP TABLE IF EXISTS invoices;

DROP TABLE IF EXISTS payment_methods;

DROP TABLE IF EXISTS company_subscriptions;

DROP TABLE IF EXISTS subscription_plans;
