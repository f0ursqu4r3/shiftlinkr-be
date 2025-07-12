pub mod activity;
pub mod company_repository;
pub mod invite_repository;
pub mod location_repository;
pub mod password_reset_repository;
pub mod pto_balance_repository;
pub mod shift_claim_repository;
pub mod shift_repository;
pub mod shift_swap_repository;
pub mod stats_repository;
pub mod time_off_repository;
pub mod user_company_repository;
pub mod user_repository;

// Re-export all repositories for easy importing
pub use activity::ActivityRepository;
pub use company_repository::CompanyRepository;
pub use invite_repository::InviteRepository;
pub use location_repository::LocationRepository;
pub use password_reset_repository::PasswordResetTokenRepository;
pub use pto_balance_repository::PtoBalanceRepository;
pub use shift_claim_repository::ShiftClaimRepository;
pub use shift_repository::ShiftRepository;
pub use shift_swap_repository::ShiftSwapRepository;
pub use stats_repository::StatsRepository;
pub use time_off_repository::TimeOffRepository;
pub use user_company_repository::UserCompanyRepository;
pub use user_repository::UserRepository;
