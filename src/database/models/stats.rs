use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub total_shifts: i64,
    pub upcoming_shifts: i64,
    pub pending_time_off_requests: i64,
    pub pending_swap_requests: i64,
    pub approved_time_off: i64,
    pub total_hours: f64,
    pub team_coverage: f64,
}

#[derive(Debug, Serialize)]
pub struct ShiftStats {
    pub total_shifts: i64,
    pub assigned_shifts: i64,
    pub unassigned_shifts: i64,
    pub completed_shifts: i64,
    pub cancelled_shifts: i64,
}

#[derive(Debug, Serialize)]
pub struct TimeOffStats {
    pub total_requests: i64,
    pub approved_requests: i64,
    pub denied_requests: i64,
    pub pending_requests: i64,
    pub cancelled_requests: i64,
}

// Request/Response DTOs for approvals
#[derive(Debug, serde::Deserialize)]
pub struct ApprovalRequest {
    pub notes: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct DenialRequest {
    pub notes: String, // Required for denials
}
