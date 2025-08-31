use crate::middleware::cache::{CacheLayer, InvalidationContext};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_smart_tag_inference() {
        let cache = CacheLayer::new(1000, 300);

        // Test JWT extraction and tag inference
        let test_cases = vec![
            (
                "/api/v1/shifts/abc-123?user_id=456",
                Some(
                    "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiI0NTYiLCJjb21wYW55X2lkIjoiNzg5Iiwicm9sZSI6Im1hbmFnZXIiLCJleHAiOjE2MjM4MjQwMDB9.fake_signature",
                ),
                vec![
                    "user:456",
                    "company:789",
                    "shifts",
                    "shifts:company:789",
                    "shifts:abc-123",
                    "shifts:abc-123:user:456",
                    "user:456", // from query param
                    "stats",    // relationship
                ],
            ),
            (
                "/api/v1/users/def-456?company_id=789",
                None,
                vec![
                    "users",
                    "users:def-456",
                    "company:789", // from query param
                    "shifts",      // relationship
                ],
            ),
            (
                "/api/v1/stats?company_id=789&period=month",
                Some("Bearer eyJzdWIiOiI0NTYiLCJjb21wYW55X2lkIjoiNzg5In0.fake"),
                vec![
                    "user:456",
                    "company:789",
                    "stats",
                    "stats:company:789",
                    "shifts",   // stats depend on shifts
                    "users",    // stats depend on users
                    "time-off", // stats depend on time-off
                ],
            ),
        ];

        for (uri, auth, expected_tags) in test_cases {
            let tags = cache.infer_tags(uri, auth);
            println!("URI: {} → Tags: {:?}", uri, tags);

            // Verify some expected tags are present
            for expected in expected_tags {
                assert!(
                    tags.contains(&expected.to_string()),
                    "Expected tag '{}' not found in {:?}",
                    expected,
                    tags
                );
            }
        }
    }

    #[tokio::test]
    async fn test_cache_invalidation_precision() {
        let cache = CacheLayer::new(1000, 300);

        // Simulate caching some responses
        cache
            .store_with_tags(
                "key1".to_string(),
                crate::middleware::cache::CachedHttp {
                    status: 200,
                    headers: vec![],
                    body: b"shift data".to_vec(),
                    tags: vec!["shifts".to_string(), "company:123".to_string()],
                },
            )
            .await;

        cache
            .store_with_tags(
                "key2".to_string(),
                crate::middleware::cache::CachedHttp {
                    status: 200,
                    headers: vec![],
                    body: b"user data".to_vec(),
                    tags: vec!["users".to_string(), "company:456".to_string()],
                },
            )
            .await;

        // Invalidate only company:123 related entries
        cache
            .invalidate(
                "shifts",
                &InvalidationContext {
                    company_id: Some(
                        uuid::Uuid::parse_str("12345678-1234-1234-1234-123456789012").unwrap(),
                    ),
                    ..Default::default()
                },
            )
            .await;

        // key1 should be invalidated, key2 should remain
        assert!(
            cache.cache.get("key1").await.is_none(),
            "key1 should be invalidated"
        );
        assert!(
            cache.cache.get("key2").await.is_some(),
            "key2 should remain cached"
        );
    }
}

/// Example usage in handlers showing the improved API
pub fn example_handler_usage() {
    // Before: Nuclear option
    // cache.bump(); // Destroys ALL cache entries!

    // After: Surgical precision
    /*
    cache.invalidate("shifts", &InvalidationContext {
        company_id: Some(company_id),      // From user context
        resource_id: Some(shift_id),       // Specific shift
        user_id: Some(user_id),           // User-specific context
        ..Default::default()
    }).await;

    // This will intelligently invalidate only:
    // - shifts (all shift endpoints)
    // - company:123 (company-scoped endpoints)
    // - shifts:company:123 (company-specific shifts)
    // - shifts:abc-456 (this specific shift)
    // - shifts:abc-456:user:789 (user's access to this shift)
    // - stats (related resource that depends on shifts)
    */
}
