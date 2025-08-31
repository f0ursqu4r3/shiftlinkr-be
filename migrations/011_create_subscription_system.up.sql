-- Subscription and payment management system
-- This migration adds Stripe integration for subscription tracking
-- Subscription plans table
CREATE TABLE
    subscription_plans (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
        stripe_price_id VARCHAR(255) NOT NULL UNIQUE,
        name VARCHAR(255) NOT NULL,
        description TEXT,
        price_cents INTEGER NOT NULL, -- Price in cents
        currency VARCHAR(3) NOT NULL DEFAULT 'usd',
        INTERVAL VARCHAR(20) NOT NULL, -- 'month', 'year', etc.
        interval_count INTEGER NOT NULL DEFAULT 1,
        is_active BOOLEAN NOT NULL DEFAULT TRUE,
        max_users INTEGER, -- NULL for unlimited
        max_companies INTEGER DEFAULT 1,
        features JSONB, -- Store plan features as JSON
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
    );

-- User subscriptions table (now company-based)
CREATE TABLE
    company_subscriptions (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
        company_id UUID NOT NULL REFERENCES companies (id) ON DELETE CASCADE,
        stripe_customer_id VARCHAR(255) NOT NULL UNIQUE,
        stripe_subscription_id VARCHAR(255) UNIQUE,
        subscription_plan_id UUID REFERENCES subscription_plans (id),
        status VARCHAR(50) NOT NULL DEFAULT 'incomplete', -- incomplete, incomplete_expired, trialing, active, past_due, canceled, unpaid
        current_period_start TIMESTAMPTZ,
        current_period_end TIMESTAMPTZ,
        cancel_at_period_end BOOLEAN NOT NULL DEFAULT FALSE,
        canceled_at TIMESTAMPTZ,
        trial_start TIMESTAMPTZ,
        trial_end TIMESTAMPTZ,
        price_cents INTEGER,
        currency VARCHAR(3) DEFAULT 'usd',
        INTERVAL VARCHAR(20),
        interval_count INTEGER DEFAULT 1,
        metadata JSONB, -- Additional Stripe metadata
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
    );

-- Payment methods table (company-based)
CREATE TABLE
    payment_methods (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
        company_id UUID NOT NULL REFERENCES companies (id) ON DELETE CASCADE,
        stripe_payment_method_id VARCHAR(255) NOT NULL UNIQUE,
        type VARCHAR(50) NOT NULL, -- 'card', 'bank_account', etc.
        last4 VARCHAR(4), -- Last 4 digits for cards
        brand VARCHAR(50), -- 'visa', 'mastercard', etc.
        expiry_month INTEGER,
        expiry_year INTEGER,
        is_default BOOLEAN NOT NULL DEFAULT FALSE,
        metadata JSONB,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
    );

-- Invoices table for billing history (company-based)
CREATE TABLE
    invoices (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
        company_id UUID NOT NULL REFERENCES companies (id) ON DELETE CASCADE,
        stripe_invoice_id VARCHAR(255) NOT NULL UNIQUE,
        subscription_id UUID REFERENCES company_subscriptions (id),
        amount_due INTEGER NOT NULL, -- Amount in cents
        amount_paid INTEGER NOT NULL DEFAULT 0,
        currency VARCHAR(3) NOT NULL DEFAULT 'usd',
        status VARCHAR(50) NOT NULL, -- 'draft', 'open', 'paid', 'void', 'uncollectible'
        invoice_pdf VARCHAR(500),
        hosted_invoice_url VARCHAR(500),
        period_start TIMESTAMPTZ,
        period_end TIMESTAMPTZ,
        due_date TIMESTAMPTZ,
        paid_at TIMESTAMPTZ,
        metadata JSONB,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
    );

-- Add subscription-related fields to companies table
ALTER TABLE companies
ADD COLUMN stripe_customer_id VARCHAR(255) UNIQUE,
ADD COLUMN subscription_status VARCHAR(50) DEFAULT 'inactive',
ADD COLUMN subscription_plan_id UUID REFERENCES subscription_plans (id),
ADD COLUMN trial_ends_at TIMESTAMPTZ,
ADD COLUMN subscription_ends_at TIMESTAMPTZ,
ADD COLUMN max_users INTEGER DEFAULT 10,
ADD COLUMN is_active BOOLEAN NOT NULL DEFAULT TRUE;

-- Add owner role to user_company table
ALTER TABLE user_company
ADD COLUMN is_owner BOOLEAN NOT NULL DEFAULT FALSE;

-- Indexes for performance
CREATE INDEX idx_subscription_plans_stripe_price_id ON subscription_plans (stripe_price_id);

CREATE INDEX idx_subscription_plans_active ON subscription_plans (is_active);

CREATE INDEX idx_company_subscriptions_company_id ON company_subscriptions (company_id);

CREATE INDEX idx_company_subscriptions_stripe_customer_id ON company_subscriptions (stripe_customer_id);

CREATE INDEX idx_company_subscriptions_stripe_subscription_id ON company_subscriptions (stripe_subscription_id);

CREATE INDEX idx_company_subscriptions_status ON company_subscriptions (status);

CREATE INDEX idx_company_subscriptions_plan_id ON company_subscriptions (subscription_plan_id);

CREATE INDEX idx_payment_methods_company_id ON payment_methods (company_id);

CREATE INDEX idx_payment_methods_stripe_id ON payment_methods (stripe_payment_method_id);

CREATE INDEX idx_payment_methods_default ON payment_methods (company_id, is_default);

CREATE INDEX idx_invoices_company_id ON invoices (company_id);

CREATE INDEX idx_invoices_stripe_id ON invoices (stripe_invoice_id);

CREATE INDEX idx_invoices_status ON invoices (status);

CREATE INDEX idx_invoices_subscription_id ON invoices (subscription_id);

CREATE INDEX idx_companies_stripe_customer_id ON companies (stripe_customer_id);

CREATE INDEX idx_companies_subscription_status ON companies (subscription_status);

CREATE INDEX idx_companies_subscription_plan_id ON companies (subscription_plan_id);

CREATE INDEX idx_companies_active ON companies (is_active);

CREATE INDEX idx_user_company_is_owner ON user_company (is_owner);
