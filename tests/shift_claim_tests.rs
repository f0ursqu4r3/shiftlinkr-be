use be::database::models::{
    CreateUpdateShiftInput, LocationInput, ShiftClaimInput, ShiftClaimStatus, ShiftStatus, User,
};
use be::database::repositories::{
    location as location_repo, shift as shift_repo, shift_claim as shift_claim_repo,
    user as user_repo,
};
use be::database::transaction::DatabaseTransaction;
use chrono::Utc;

mod common;

#[tokio::test]
async fn test_shift_claim_repository() -> Result<(), be::error::AppError> {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();

    // Create users
    let user = User::new(
        "test@example.com".to_string(),
        "hash123".to_string(),
        "Test User".to_string(),
    );
    let manager = User::new(
        "manager@example.com".to_string(),
        "hash456".to_string(),
        "Manager User".to_string(),
    );
    DatabaseTransaction::run(|tx| {
        let u = user.clone();
        let m = manager.clone();
        Box::pin(async move {
            user_repo::create_user(tx, &u).await?;
            user_repo::create_user(tx, &m).await?;
            Ok::<_, be::error::AppError>(())
        })
    })
    .await?;

    // Create a company for location/shift
    let company_id = common::create_default_test_company().await.unwrap();

    // Create location
    let location = DatabaseTransaction::run(|tx| {
        let input = LocationInput {
            name: "Test Location".to_string(),
            address: Some("123 Test St".to_string()),
            phone: Some("555-1234".to_string()),
            email: Some("test@location.com".to_string()),
            company_id,
        };
        Box::pin(async move {
            Ok::<_, be::error::AppError>(location_repo::create_location(tx, input).await?)
        })
    })
    .await?;

    // Create shift
    let shift = DatabaseTransaction::run(|tx| {
        let input = CreateUpdateShiftInput {
            company_id,
            title: "Test Shift".to_string(),
            description: Some("Test shift description".to_string()),
            location_id: location.id,
            team_id: None,
            start_time: Utc::now(),
            end_time: Utc::now() + chrono::Duration::hours(8),
            min_duration_minutes: Some(60),
            max_duration_minutes: Some(480),
            max_people: Some(5),
            status: ShiftStatus::Open,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        Box::pin(
            async move { Ok::<_, be::error::AppError>(shift_repo::create_shift(tx, input).await?) },
        )
    })
    .await?;

    // Create shift claim
    let claim = DatabaseTransaction::run(|tx| {
        let input = ShiftClaimInput {
            shift_id: shift.id,
            user_id: user.id,
        };
        Box::pin(async move {
            Ok::<_, be::error::AppError>(shift_claim_repo::create_claim(tx, &input).await?)
        })
    })
    .await?;

    assert_eq!(claim.shift_id, shift.id);
    assert_eq!(claim.user_id, user.id);
    assert!(matches!(claim.status, ShiftClaimStatus::Pending));
    assert!(claim.actioned_by.is_none());
    assert!(claim.action_notes.is_none());

    // Get claim by ID
    let fetched_claim = shift_claim_repo::get_claim_by_id(claim.id).await?;
    assert!(fetched_claim.is_some());
    let fetched_claim = fetched_claim.unwrap();
    assert_eq!(fetched_claim.id, claim.id);
    assert_eq!(fetched_claim.shift_id, shift.id);

    // Get claims by shift
    let shift_claims = shift_claim_repo::get_claims_by_shift(shift.id).await?;
    assert_eq!(shift_claims.len(), 1);
    assert_eq!(shift_claims[0].id, claim.id);

    // Get claims by user
    let user_claims = shift_claim_repo::get_claims_by_user(user.id).await?;
    assert_eq!(user_claims.len(), 1);
    assert_eq!(user_claims[0].id, claim.id);

    // Check pending
    let has_pending = shift_claim_repo::has_pending_claim(shift.id, user.id).await?;
    assert!(has_pending.is_some());

    // Approve claim
    let approved = DatabaseTransaction::run(|tx| {
        let claim_id = claim.id;
        let manager_id = manager.id;
        Box::pin(async move {
            Ok::<_, be::error::AppError>(
                shift_claim_repo::approve_claim(tx, claim_id, manager_id, Some("Approved!".into()))
                    .await?,
            )
        })
    })
    .await?;
    assert!(approved.is_some());
    let approved = approved.unwrap();
    assert!(matches!(approved.status, ShiftClaimStatus::Approved));
    assert_eq!(approved.actioned_by, Some(manager.id));
    assert_eq!(approved.action_notes, Some("Approved!".to_string()));

    // Shift has approved claim
    let has_approved = shift_claim_repo::has_approved_claim(shift.id).await?;
    assert!(has_approved.is_some());

    // Approved claim for shift
    let approved_for_shift = shift_claim_repo::get_approved_claim_for_shift(shift.id).await?;
    assert!(approved_for_shift.is_some());
    assert!(matches!(
        approved_for_shift.unwrap().status,
        ShiftClaimStatus::Approved
    ));

    Ok(())
}

#[tokio::test]
async fn test_shift_claim_cancel_and_reject() -> Result<(), be::error::AppError> {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();

    // Users
    let user = User::new(
        "test2@example.com".to_string(),
        "hash456".to_string(),
        "Test User2".to_string(),
    );
    let manager = User::new(
        "manager2@example.com".to_string(),
        "hash789".to_string(),
        "Manager User2".to_string(),
    );
    DatabaseTransaction::run(|tx| {
        let u = user.clone();
        let m = manager.clone();
        Box::pin(async move {
            user_repo::create_user(tx, &u).await?;
            user_repo::create_user(tx, &m).await?;
            Ok::<_, be::error::AppError>(())
        })
    })
    .await?;

    // Company and location
    let company_id = common::create_default_test_company().await.unwrap();
    let location = DatabaseTransaction::run(|tx| {
        let input = LocationInput {
            name: "Test Location 2".to_string(),
            address: Some("456 Test Ave".to_string()),
            phone: Some("555-5678".to_string()),
            email: Some("test2@location.com".to_string()),
            company_id,
        };
        Box::pin(async move {
            Ok::<_, be::error::AppError>(location_repo::create_location(tx, input).await?)
        })
    })
    .await?;

    // Shift
    let shift = DatabaseTransaction::run(|tx| {
        let input = CreateUpdateShiftInput {
            company_id,
            title: "Test Shift 2".to_string(),
            description: Some("Test shift 2 description".to_string()),
            location_id: location.id,
            team_id: None,
            start_time: Utc::now(),
            end_time: Utc::now() + chrono::Duration::hours(8),
            min_duration_minutes: Some(60),
            max_duration_minutes: Some(480),
            max_people: Some(5),
            status: ShiftStatus::Open,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        Box::pin(
            async move { Ok::<_, be::error::AppError>(shift_repo::create_shift(tx, input).await?) },
        )
    })
    .await?;

    // Claim
    let claim = DatabaseTransaction::run(|tx| {
        let input = ShiftClaimInput {
            shift_id: shift.id,
            user_id: user.id,
        };
        Box::pin(async move {
            Ok::<_, be::error::AppError>(shift_claim_repo::create_claim(tx, &input).await?)
        })
    })
    .await?;

    // Cancel claim
    let cancelled = DatabaseTransaction::run(|tx| {
        let cid = claim.id;
        let uid = user.id;
        Box::pin(async move {
            Ok::<_, be::error::AppError>(shift_claim_repo::cancel_claim(tx, cid, uid).await?)
        })
    })
    .await?;
    assert!(cancelled.is_some());
    assert!(matches!(
        cancelled.unwrap().status,
        ShiftClaimStatus::Cancelled
    ));

    // New claim for rejection
    let claim2 = DatabaseTransaction::run(|tx| {
        let input = ShiftClaimInput {
            shift_id: shift.id,
            user_id: user.id,
        };
        Box::pin(async move {
            Ok::<_, be::error::AppError>(shift_claim_repo::create_claim(tx, &input).await?)
        })
    })
    .await?;

    // Reject claim
    let rejected = DatabaseTransaction::run(|tx| {
        let cid = claim2.id;
        let mid = manager.id;
        Box::pin(async move {
            Ok::<_, be::error::AppError>(
                shift_claim_repo::reject_claim(tx, cid, mid, Some("Not qualified".into())).await?,
            )
        })
    })
    .await?;
    assert!(rejected.is_some());
    let rejected = rejected.unwrap();
    assert!(matches!(rejected.status, ShiftClaimStatus::Rejected));
    assert_eq!(rejected.actioned_by, Some(manager.id));
    assert_eq!(rejected.action_notes, Some("Not qualified".to_string()));

    Ok(())
}
