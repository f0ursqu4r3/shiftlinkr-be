use be::database::models::{
    LocationInput, ShiftClaimInput, ShiftClaimStatus, ShiftInput, ShiftStatus, User,
};
use be::database::repositories::location::LocationRepository;
use be::database::repositories::shift::ShiftRepository;
use be::database::repositories::shift_claim::ShiftClaimRepository;
use be::database::repositories::user::UserRepository;
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
    let shift_claim_repo = ShiftClaimRepository::new(pool.clone()); // Create test user
    let user = User::new(
        "test@example.com".to_string(),
        "hash123".to_string(),
        "Test User".to_string(),
    );

    // Debug: Check if user creation succeeds
    match user_repo.create_user(&user).await {
        Ok(_) => println!("✅ User created successfully"),
        Err(e) => {
            println!("❌ User creation failed: {}", e);
            return Err(sqlx::Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("User creation failed: {}", e),
            )));
        }
    }

    // Create manager user for approval
    let manager = User::new(
        "manager@example.com".to_string(),
        "hash456".to_string(),
        "Manager User".to_string(),
    );

    // Debug: Check if manager creation succeeds
    match user_repo.create_user(&manager).await {
        Ok(_) => println!("✅ Manager created successfully"),
        Err(e) => {
            println!("❌ Manager creation failed: {}", e);
            return Err(sqlx::Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Manager creation failed: {}", e),
            )));
        }
    }

    // Create test location
    let location_input = LocationInput {
        name: "Test Location".to_string(),
        address: Some("123 Test St".to_string()),
        phone: Some("555-1234".to_string()),
        email: Some("test@location.com".to_string()),
        company_id: 1, // Default company ID
    };
    let location = match location_repo.create_location(location_input).await {
        Ok(loc) => {
            println!("✅ Location created successfully with ID: {}", loc.id);
            loc
        }
        Err(e) => {
            println!("❌ Location creation failed: {}", e);
            return Err(sqlx::Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Location creation failed: {}", e),
            )));
        }
    };

    // Create test shift
    let shift_input = ShiftInput {
        title: "Test Shift".to_string(),
        description: Some("Test shift description".to_string()),
        location_id: location.id,
        team_id: None,
        min_duration_minutes: Some(60),
        max_duration_minutes: Some(480),
        max_people: Some(5),
        status: ShiftStatus::Open,
        created_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
        start_time: Utc::now().naive_utc(),
        end_time: (Utc::now() + chrono::Duration::hours(8)).naive_utc(),
    };
    let shift = match shift_repo.create_shift(shift_input).await {
        Ok(shift) => {
            println!("✅ Shift created successfully with ID: {}", shift.id);
            shift
        }
        Err(e) => {
            println!("❌ Shift creation failed: {}", e);
            return Err(sqlx::Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Shift creation failed: {}", e),
            )));
        }
    };

    // Test 1: Create a shift claim
    let claim_input = ShiftClaimInput {
        shift_id: shift.id,
        user_id: user.id.clone(), // Use the actual user ID that was created
    };
    println!(
        "🔍 Attempting to create shift claim for shift_id: {}, user_id: {}",
        shift.id, user.id
    );
    let claim = match shift_claim_repo.create_claim(&claim_input).await {
        Ok(claim) => {
            println!("✅ Shift claim created successfully with ID: {}", claim.id);
            claim
        }
        Err(e) => {
            println!("❌ Shift claim creation failed: {}", e);
            return Err(e);
        }
    };

    assert_eq!(claim.shift_id, shift.id);
    assert_eq!(claim.user_id, user.id);
    assert!(matches!(claim.status, ShiftClaimStatus::Pending));
    assert!(claim.approved_by.is_none());
    assert!(claim.approval_notes.is_none());
    println!("✅ Shift claim assertions passed");

    // Test 2: Get claim by ID
    println!("🔍 Attempting to get claim by ID: {}", claim.id);
    let fetched_claim = match shift_claim_repo.get_claim_by_id(claim.id).await {
        Ok(fetched_claim) => {
            println!("✅ Fetched claim successfully");
            fetched_claim
        }
        Err(e) => {
            println!("❌ Get claim by ID failed: {}", e);
            return Err(e);
        }
    };
    assert!(fetched_claim.is_some());
    let fetched_claim = fetched_claim.unwrap();
    assert_eq!(fetched_claim.id, claim.id);
    assert_eq!(fetched_claim.shift_id, shift.id);
    println!("✅ Get claim by ID assertions passed");

    // Test 3: Get claims by shift
    println!("🔍 Attempting to get claims by shift: {}", shift.id);
    let shift_claims = match shift_claim_repo.get_claims_by_shift(shift.id).await {
        Ok(claims) => {
            println!("✅ Got {} claims for shift", claims.len());
            claims
        }
        Err(e) => {
            println!("❌ Get claims by shift failed: {}", e);
            return Err(e);
        }
    };
    assert_eq!(shift_claims.len(), 1);
    assert_eq!(shift_claims[0].id, claim.id);
    println!("✅ Get claims by shift assertions passed");

    // Test 4: Get claims by user
    println!("🔍 Attempting to get claims by user: {}", user.id);
    let user_claims = match shift_claim_repo.get_claims_by_user(&user.id).await {
        Ok(claims) => {
            println!("✅ Got {} claims for user", claims.len());
            claims
        }
        Err(e) => {
            println!("❌ Get claims by user failed: {}", e);
            return Err(e);
        }
    };
    assert_eq!(user_claims.len(), 1);
    assert_eq!(user_claims[0].id, claim.id);
    println!("✅ Get claims by user assertions passed");

    // Test 5: Check if user has pending claim
    println!("🔍 Checking if user has pending claim");
    let has_pending = match shift_claim_repo.has_pending_claim(shift.id, &user.id).await {
        Ok(has_pending) => {
            println!("✅ Has pending claim check completed: {}", has_pending);
            has_pending
        }
        Err(e) => {
            println!("❌ Has pending claim check failed: {}", e);
            return Err(e);
        }
    };
    assert!(has_pending);
    println!("✅ Has pending claim assertion passed");

    // Test 6: Approve the claim
    println!("🔍 Attempting to approve claim");
    let approved_claim = match shift_claim_repo
        .approve_claim(claim.id, "manager_123", Some("Approved!".to_string()))
        .await
    {
        Ok(approved_claim) => {
            println!("✅ Claim approved successfully");
            approved_claim
        }
        Err(e) => {
            println!("❌ Claim approval failed: {}", e);
            return Err(e);
        }
    };
    assert!(approved_claim.is_some());
    let approved_claim = approved_claim.unwrap();
    assert!(matches!(approved_claim.status, ShiftClaimStatus::Approved));
    assert_eq!(approved_claim.approved_by, Some("manager_123".to_string()));
    assert_eq!(approved_claim.approval_notes, Some("Approved!".to_string()));
    println!("✅ Approve claim assertions passed");

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
    let user = User::new(
        "test2@example.com".to_string(),
        "hash456".to_string(),
        "Test User2".to_string(),
    );
    user_repo.create_user(&user).await.unwrap();

    // Create manager user for approval
    let manager = User::new(
        "manager2@example.com".to_string(),
        "hash789".to_string(),
        "Manager User2".to_string(),
    );
    user_repo.create_user(&manager).await.unwrap();

    // Create test location
    let location_input = LocationInput {
        name: "Test Location 2".to_string(),
        address: Some("456 Test Ave".to_string()),
        phone: Some("555-5678".to_string()),
        email: Some("test2@location.com".to_string()),
        company_id: 1, // Default company ID
    };
    let location = location_repo.create_location(location_input).await.unwrap();

    // Create test shift
    let shift_input = ShiftInput {
        title: "Test Shift 2".to_string(),
        description: Some("Test shift 2 description".to_string()),
        location_id: location.id,
        team_id: None,
        min_duration_minutes: Some(60),
        max_duration_minutes: Some(480),
        max_people: Some(5),
        status: ShiftStatus::Open,
        created_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
        start_time: Utc::now().naive_utc(),
        end_time: (Utc::now() + chrono::Duration::hours(8)).naive_utc(),
    };
    let shift = shift_repo.create_shift(shift_input).await.unwrap();

    // Test 1: Create a shift claim
    let claim_input = ShiftClaimInput {
        shift_id: shift.id,
        user_id: user.id.clone(),
    };
    let claim = shift_claim_repo.create_claim(&claim_input).await?;

    // Test 2: Cancel the claim
    let cancelled_claim = shift_claim_repo.cancel_claim(claim.id, &user.id).await?;
    assert!(cancelled_claim.is_some());
    let cancelled_claim = cancelled_claim.unwrap();
    assert!(matches!(
        cancelled_claim.status,
        ShiftClaimStatus::Cancelled
    ));

    // Test 3: Create another claim for rejection test
    let claim_input2 = ShiftClaimInput {
        shift_id: shift.id,
        user_id: user.id.clone(),
    };
    let claim2 = shift_claim_repo.create_claim(&claim_input2).await?;

    // Test 4: Reject the claim
    let rejected_claim = shift_claim_repo
        .reject_claim(claim2.id, &manager.id, Some("Not qualified".to_string()))
        .await?;
    assert!(rejected_claim.is_some());
    let rejected_claim = rejected_claim.unwrap();
    assert!(matches!(rejected_claim.status, ShiftClaimStatus::Rejected));
    assert_eq!(rejected_claim.approved_by, Some(manager.id.clone()));
    assert_eq!(
        rejected_claim.approval_notes,
        Some("Not qualified".to_string())
    );

    println!("✅ All shift claim cancel/reject tests passed!");
    Ok(())
}
