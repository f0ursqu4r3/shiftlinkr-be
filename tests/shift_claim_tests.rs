use be::database::location_repository::LocationRepository;
use be::database::models::{
    LocationInput, ShiftClaimInput, ShiftClaimStatus, ShiftInput, User, UserRole,
};
use be::database::shift_claim_repository::ShiftClaimRepository;
use be::database::shift_repository::ShiftRepository;
use be::database::user_repository::UserRepository;
use chrono::Utc;
use sqlx::SqlitePool;

#[sqlx::test]
async fn test_shift_claim_repository(pool: SqlitePool) -> Result<(), sqlx::Error> {
    // Disable foreign key constraints for testing
    sqlx::query("PRAGMA foreign_keys = OFF")
        .execute(&pool)
        .await?;

    // Create repositories
    let user_repo = UserRepository::new(pool.clone());
    let location_repo = LocationRepository::new(pool.clone());
    let shift_repo = ShiftRepository::new(pool.clone());
    let shift_claim_repo = ShiftClaimRepository::new(pool.clone());

    // Create test user
    let user = User {
        id: "test_user_123".to_string(),
        email: "test@example.com".to_string(),
        password_hash: "hash123".to_string(),
        name: "Test User".to_string(),
        role: UserRole::Employee,
        pto_balance_hours: 0,
        sick_balance_hours: 0,
        personal_balance_hours: 0,
        pto_accrual_rate: 0.0,
        hire_date: None,
        last_accrual_date: None,
        created_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
    };
    user_repo.create_user(&user).await.unwrap();

    // Create test location
    let location_input = LocationInput {
        name: "Test Location".to_string(),
        address: Some("123 Test St".to_string()),
        phone: Some("555-1234".to_string()),
        email: Some("test@location.com".to_string()),
    };
    let location = location_repo.create_location(location_input).await.unwrap();

    // Create test shift
    let shift_input = ShiftInput {
        title: "Test Shift".to_string(),
        description: Some("Test shift description".to_string()),
        location_id: location.id,
        team_id: None,
        assigned_user_id: None,
        start_time: Utc::now().naive_utc(),
        end_time: (Utc::now() + chrono::Duration::hours(8)).naive_utc(),
        hourly_rate: Some(15.0),
    };
    let shift = shift_repo.create_shift(shift_input).await.unwrap();

    // Test 1: Create a shift claim
    let claim_input = ShiftClaimInput {
        shift_id: shift.id,
        user_id: "test_user_123".to_string(),
    };
    let claim = shift_claim_repo.create_claim(&claim_input).await?;

    assert_eq!(claim.shift_id, shift.id);
    assert_eq!(claim.user_id, "test_user_123");
    assert!(matches!(claim.status, ShiftClaimStatus::Pending));
    assert!(claim.approved_by.is_none());
    assert!(claim.approval_notes.is_none());

    // Test 2: Get claim by ID
    let fetched_claim = shift_claim_repo.get_claim_by_id(claim.id).await?;
    assert!(fetched_claim.is_some());
    let fetched_claim = fetched_claim.unwrap();
    assert_eq!(fetched_claim.id, claim.id);
    assert_eq!(fetched_claim.shift_id, shift.id);

    // Test 3: Get claims by shift
    let shift_claims = shift_claim_repo.get_claims_by_shift(shift.id).await?;
    assert_eq!(shift_claims.len(), 1);
    assert_eq!(shift_claims[0].id, claim.id);

    // Test 4: Get claims by user
    let user_claims = shift_claim_repo.get_claims_by_user("test_user_123").await?;
    assert_eq!(user_claims.len(), 1);
    assert_eq!(user_claims[0].id, claim.id);

    // Test 5: Check if user has pending claim
    let has_pending = shift_claim_repo
        .has_pending_claim(shift.id, "test_user_123")
        .await?;
    assert!(has_pending);

    // Test 6: Approve the claim
    let approved_claim = shift_claim_repo
        .approve_claim(claim.id, "manager_123", Some("Approved!".to_string()))
        .await?;
    assert!(approved_claim.is_some());
    let approved_claim = approved_claim.unwrap();
    assert!(matches!(approved_claim.status, ShiftClaimStatus::Approved));
    assert_eq!(approved_claim.approved_by, Some("manager_123".to_string()));
    assert_eq!(approved_claim.approval_notes, Some("Approved!".to_string()));

    // Test 7: Check if shift has approved claim
    let has_approved = shift_claim_repo.has_approved_claim(shift.id).await?;
    assert!(has_approved);

    // Test 8: Get approved claim for shift
    let approved_claim_for_shift = shift_claim_repo
        .get_approved_claim_for_shift(shift.id)
        .await?;
    assert!(approved_claim_for_shift.is_some());
    let approved_claim_for_shift = approved_claim_for_shift.unwrap();
    assert!(matches!(
        approved_claim_for_shift.status,
        ShiftClaimStatus::Approved
    ));

    // Test 9: Get pending claims (should be empty now)
    let pending_claims = shift_claim_repo.get_pending_claims().await?;
    assert_eq!(pending_claims.len(), 0);

    println!("✅ All shift claim repository tests passed!");
    Ok(())
}

#[sqlx::test]
async fn test_shift_claim_cancel_and_reject(pool: SqlitePool) -> Result<(), sqlx::Error> {
    // Disable foreign key constraints for testing
    sqlx::query("PRAGMA foreign_keys = OFF")
        .execute(&pool)
        .await?;

    // Create repositories
    let user_repo = UserRepository::new(pool.clone());
    let location_repo = LocationRepository::new(pool.clone());
    let shift_repo = ShiftRepository::new(pool.clone());
    let shift_claim_repo = ShiftClaimRepository::new(pool.clone());

    // Create test user
    let user = User {
        id: "test_user_456".to_string(),
        email: "test2@example.com".to_string(),
        password_hash: "hash456".to_string(),
        name: "Test User2".to_string(),
        role: UserRole::Employee,
        pto_balance_hours: 0,
        sick_balance_hours: 0,
        personal_balance_hours: 0,
        pto_accrual_rate: 0.0,
        hire_date: None,
        last_accrual_date: None,
        created_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
    };
    user_repo.create_user(&user).await.unwrap();

    // Create test location
    let location_input = LocationInput {
        name: "Test Location 2".to_string(),
        address: Some("456 Test Ave".to_string()),
        phone: Some("555-5678".to_string()),
        email: Some("test2@location.com".to_string()),
    };
    let location = location_repo.create_location(location_input).await.unwrap();

    // Create test shift
    let shift_input = ShiftInput {
        title: "Test Shift 2".to_string(),
        description: Some("Test shift 2 description".to_string()),
        location_id: location.id,
        team_id: None,
        assigned_user_id: None,
        start_time: Utc::now().naive_utc(),
        end_time: (Utc::now() + chrono::Duration::hours(8)).naive_utc(),
        hourly_rate: Some(20.0),
    };
    let shift = shift_repo.create_shift(shift_input).await.unwrap();

    // Test 1: Create a shift claim
    let claim_input = ShiftClaimInput {
        shift_id: shift.id,
        user_id: "test_user_456".to_string(),
    };
    let claim = shift_claim_repo.create_claim(&claim_input).await?;

    // Test 2: Cancel the claim
    let cancelled_claim = shift_claim_repo
        .cancel_claim(claim.id, "test_user_456")
        .await?;
    assert!(cancelled_claim.is_some());
    let cancelled_claim = cancelled_claim.unwrap();
    assert!(matches!(
        cancelled_claim.status,
        ShiftClaimStatus::Cancelled
    ));

    // Test 3: Create another claim for rejection test
    let claim_input2 = ShiftClaimInput {
        shift_id: shift.id,
        user_id: "test_user_456".to_string(),
    };
    let claim2 = shift_claim_repo.create_claim(&claim_input2).await?;

    // Test 4: Reject the claim
    let rejected_claim = shift_claim_repo
        .reject_claim(claim2.id, "manager_456", Some("Not qualified".to_string()))
        .await?;
    assert!(rejected_claim.is_some());
    let rejected_claim = rejected_claim.unwrap();
    assert!(matches!(rejected_claim.status, ShiftClaimStatus::Rejected));
    assert_eq!(rejected_claim.approved_by, Some("manager_456".to_string()));
    assert_eq!(
        rejected_claim.approval_notes,
        Some("Not qualified".to_string())
    );

    println!("✅ All shift claim cancel/reject tests passed!");
    Ok(())
}
