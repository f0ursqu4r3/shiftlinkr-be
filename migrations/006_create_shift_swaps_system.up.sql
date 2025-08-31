-- Shift swaps system: shift swaps and responses
-- This migration creates the shift swap functionality
-- Shift swaps
CREATE TABLE
    shift_swaps (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
        requesting_user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
        original_shift_id UUID NOT NULL REFERENCES shifts (id) ON DELETE CASCADE,
        target_user_id UUID REFERENCES users (id) ON DELETE SET NULL,
        target_shift_id UUID REFERENCES shifts (id) ON DELETE SET NULL,
        notes TEXT,
        response VARCHAR(50),
        type VARCHAR(50) NOT NULL DEFAULT 'open',
        status VARCHAR(50) NOT NULL DEFAULT 'open',
        actioned_by UUID REFERENCES users (id) ON DELETE SET NULL,
        action_notes TEXT,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
    );

-- Shift swap responses
CREATE TABLE
    shift_swap_responses (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
        swap_id UUID NOT NULL REFERENCES shift_swaps (id) ON DELETE CASCADE,
        responding_user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
        response_type VARCHAR(50) NOT NULL,
        notes TEXT,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        UNIQUE (swap_id, responding_user_id)
    );

-- Indexes for performance
CREATE INDEX idx_shift_swaps_requesting_user_id ON shift_swaps (requesting_user_id);

CREATE INDEX idx_shift_swaps_original_shift_id ON shift_swaps (original_shift_id);

CREATE INDEX idx_shift_swaps_target_user_id ON shift_swaps (target_user_id);

CREATE INDEX idx_shift_swaps_target_shift_id ON shift_swaps (target_shift_id);

CREATE INDEX idx_shift_swaps_status ON shift_swaps (status);

CREATE INDEX idx_shift_swap_responses_swap_id ON shift_swap_responses (swap_id);

CREATE INDEX idx_shift_swap_responses_user_id ON shift_swap_responses (responding_user_id);

CREATE INDEX idx_shift_swap_responses_response_type ON shift_swap_responses (response_type);
