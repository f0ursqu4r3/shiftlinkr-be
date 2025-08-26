# ShiftLinkr Backend - Priority Issues TODO List

**Generated:** August 20, 2025  
**Status:** Post-Parameter Reordering & P0 Fixes

## **P0 - Critical Issues (Security/Data Integrity)**

### ✅ **COMPLETED**

1. **Missing Database Migration for `min_duration_minutes`**
   - **Status:** ✅ Fixed in migration `007_restructure_wage_and_shift_management.up.sql`
   - **Solution:** Added `min_duration_minutes`, `max_duration_minutes`, and `max_people` columns to shifts table

2. **SQL Injection Risk in Dynamic Queries**
   - **Status:** ✅ False positive - queries use server-controlled parameters
   - **Verification:** Dynamic queries use authenticated user context, not direct user input

3. **Missing Rate Limiting on Auth Endpoints**
   - **Status:** ✅ Fixed - Rate limiting implemented on all auth endpoints
   - **Solution:** Applied `AuthRateLimiter` to register, login, forgot-password, and reset-password

---

## **P1 - Data Consistency Issues**

1. **Transaction Rollback Gaps**
   - **Issue:** ✅ Multiple database operations without proper transaction boundaries
   - **Impact:** Partial state corruption if operations fail mid-process
   - **Files:** `handlers/shifts.rs`, `handlers/swaps.rs`, `handlers/time_off.rs`
   - **Solution:** Wrap multi-step operations in `DatabaseTransaction::run()`
   - **Example:**

     ```rust
     // Instead of separate operations
     let shift = create_shift(data).await?;
     let assignment = create_assignment(shift.id).await?;
     
     // Use single transaction
     DatabaseTransaction::run(|tx| {
         Box::pin(async move {
             let shift = create_shift(tx, data).await?;
             let assignment = create_assignment(tx, shift.id).await?;
             Ok((shift, assignment))
         })
     }).await?;
     ```

2. **Orphaned Data on Deletes**
   - **Status:** ✅ Fixed in migration `008_add_cascade_constraints.up.sql`
   - **Solution:** Added CASCADE and SET NULL constraints to all foreign key relationships
   - **Impact:** Prevents orphaned records when companies, users, shifts, or other parent entities are deleted
   - **Details:**
     - CASCADE: Child records deleted when parent deleted (e.g., shifts deleted when company deleted)
     - SET NULL: Optional references nullified when parent deleted (e.g., shift.user_id set to NULL when user deleted)

---

## **P2 - Performance Issues**

🔄 **TODO**

1. **N+1 Query Problems**
   - **Issue:** ✅ Handlers fetch related data in loops
   - **Impact:** Poor performance with large datasets
   - **Files:** `handlers/shifts.rs` (claims fetching), `handlers/stats.rs`
   - **Solution:** Use JOINs or batch queries instead of loops
   - **Example:**

     ```rust
     // Instead of N+1
     for shift in shifts {
         let claims = get_claims(shift.id).await?; // N queries
     }
     
     // Use batch query
     let shift_ids: Vec<Uuid> = shifts.iter().map(|s| s.id).collect();
     let claims_map = get_claims_batch(shift_ids).await?;
     ```

2. **Cache Invalidation Too Broad**
   - **Issue:** `cache.bump()` invalidates ALL cached entries
   - **Impact:** Unnecessary cache misses
   - **Files:** `middleware/cache.rs`, all handlers with `cache.bump()`
   - **Solution:** Implement granular invalidation by pattern/key
   - **Example:**

     ```rust
     // Instead of
     cache.bump(); // Invalidates everything
     
     // Use granular invalidation
     cache.invalidate_pattern("/shifts/*");
     cache.invalidate_key(format!("/shifts/{}", shift_id));
     ```

3. **Missing Database Indexes**
   - **Issue:** Key queries lack proper indexes
   - **Impact:** Slow query performance
   - **Solution:** Add indexes for common query patterns
   - **Migration needed:**

     ```sql
     CREATE INDEX idx_shifts_company_date ON shifts(company_id, start_time);
     CREATE INDEX idx_shifts_user_date ON shifts(user_id, start_time);
     CREATE INDEX idx_activity_logs_user ON activity_logs(user_id);
     CREATE INDEX idx_shift_claims_shift ON shift_claims(shift_id);
     CREATE INDEX idx_user_company_lookup ON user_company(user_id, company_id);
     ```

---

## **P3 - Error Handling & Observability**

### ✅ **PARTIALLY COMPLETED**

- **Handler Parameter Consistency:** ✅ All handlers now have consistent parameter ordering (UserContext before JSON)

🔄 **TODO**

1. **Inconsistent Error Responses**
   - **Issue:** Mix of error types and status codes
   - **Impact:** Poor API consistency
   - **Files:** All handlers
   - **Solution:** Standardize on `ApiError` enum and consistent status codes
   - **Example:**

     ```rust
     // Standardize error responses
     match result {
         Err(sqlx::Error::RowNotFound) => Err(AppError::NotFound("Resource not found".to_string())),
         Err(sqlx::Error::Database(e)) if e.constraint().is_some() => {
             Err(AppError::BadRequest("Constraint violation".to_string()))
         }
         Err(e) => Err(AppError::DatabaseError(e)),
     }
     ```

2. **Insufficient Logging**
    - **Issue:** No structured logging with correlation
    - **Impact:** Difficult debugging and monitoring
    - **Solution:** Implement `tracing` with spans and correlation IDs
    - **Implementation:**

      ```rust
      use tracing::{info_span, instrument};
      
      #[instrument(skip(ctx), fields(user_id = %ctx.user_id(), request_id))]
      pub async fn create_shift(...) -> Result<HttpResponse> {
          // Function body
      }
      ```

3. **Test Flakiness**
    - **Issue:** Random test failures due to shared state
    - **Impact:** Unreliable CI/CD pipeline
    - **Files:** All test files
    - **Solution:** Better test isolation and deterministic time handling
    - **Issues to fix:**
      - Duplicate key violations in admin tests
      - Time-based assertions without fixed clocks
      - Shared database state between tests
      - OnceCell/OnceLock conflicts in config tests

---

## **P4 - Code Quality & Maintainability**

🔄 **TODO**

1. **Repository Pattern Inconsistency**
    - **Issue:** Mix of repository functions and direct model methods
    - **Impact:** Inconsistent data access patterns
    - **Files:** `database/models/*`, `database/repositories/*`
    - **Solution:** Standardize on one pattern (prefer repository functions)
    - **Example:**

      ```rust
      // Instead of mixing patterns
      shift::create_shift(...)  // Sometimes this
      ShiftRepository::create(...) // Sometimes this
      
      // Use consistent pattern
      shift_repo::create_shift(tx, data).await?
      ```

2. **Duplicate Code in Permission Checks**
    - **Issue:** Permission checks repeated across handlers
    - **Impact:** Code duplication and maintenance burden
    - **Files:** All handlers with permission checks
    - **Solution:** Create permission middleware or helper macros
    - **Example:**

      ```rust
      // Instead of repeating this everywhere
      if !ctx.has_permission("admin") && !ctx.has_permission("manager") {
          return Err(ApiError::forbidden());
      }
      
      // Use helper
      ctx.require_any_role(&[Role::Admin, Role::Manager])?;
      ```

3. **Magic Numbers/Strings Throughout Codebase**
    - **Issue:** Hardcoded values without clear meaning
    - **Impact:** Poor maintainability
    - **Files:** `routes/*` (cache configs), handlers (limits)
    - **Solution:** Move to configuration structs or constants
    - **Example:**

      ```rust
      // Instead of
      let cache = CacheLayer::new(1000, 300);
      
      // Use configuration
      struct CacheConfig {
          max_entries: usize,
          ttl_seconds: u64,
      }
      let cache = CacheLayer::new(config.cache.max_entries, config.cache.ttl_seconds);
      ```

---

## **P5 - API Design Issues**

🔄 **TODO**

1. **Inconsistent API Responses**
    - **Issue:** Mix of `ApiResponse::success()` and direct `HttpResponse`
    - **Impact:** Inconsistent client integration
    - **Files:** All handlers
    - **Solution:** Standardize on `ApiResponse` wrapper
    - **Example:**

      ```rust
      // Instead of mixing
      HttpResponse::Ok().json(data)
      ApiResponse::success(data)
      
      // Always use
      Ok(ApiResponse::success(data))
      ```

2. **Missing Pagination**
    - **Issue:** Large queries can return unlimited results
    - **Impact:** Memory issues and poor UX
    - **Files:** `handlers/shifts.rs`, `handlers/stats.rs`
    - **Solution:** Add pagination parameters and limits
    - **Example:**

      ```rust
      #[derive(Deserialize)]
      struct PaginationQuery {
          page: Option<u32>,
          limit: Option<u32>,
      }
      
      pub async fn get_shifts(
          query: web::Query<PaginationQuery>,
          ctx: UserContext,
      ) -> Result<HttpResponse> {
          let page = query.page.unwrap_or(1);
          let limit = query.limit.unwrap_or(50).min(100); // Cap at 100
          // Implementation
      }
      ```

3. **No API Versioning Strategy**
    - **Issue:** Routes at `/api/v1` but no versioning implementation
    - **Impact:** Future breaking changes difficult to manage
    - **Solution:** Implement proper API versioning strategy

---

## **P6 - Development Experience**

🔄 **TODO**

1. **Missing API Documentation**
    - **Issue:** No OpenAPI/Swagger documentation
    - **Impact:** Poor developer experience
    - **Solution:** Generate docs from code using `utoipa`
    - **Implementation:**

      ```rust
      use utoipa::{OpenApi, ToSchema};
      
      #[derive(OpenApi)]
      #[openapi(
          paths(get_shifts, create_shift),
          components(schemas(Shift, ShiftInput))
      )]
      struct ApiDoc;
      ```

2. **Development Scripts Need Updates**
    - **Issue:** Scripts reference outdated endpoints/payloads
    - **Impact:** Broken development workflow
    - **Files:** `test_api.sh`, `test_invite_flow.sh`, `test_password_reset.sh`, etc.
    - **Solution:** Update scripts to match current API

3. **Environment Configuration Security**
    - **Issue:** `.env` file in git (contains sensitive data)
    - **Impact:** Security risk
    - **Solution:** Remove `.env` from git, keep only `.env.example`
    - **Action:**

      ```bash
      git rm .env
      echo ".env" >> .gitignore
      ```

---

## **Quick Wins (High Impact, Low Effort)**

### **Immediate Actions**

1. **Add missing database indexes** (P2.8) - Single migration file
2. **Remove .env from git** (P6.20) - One command
3. **Update development scripts** (P6.19) - Text replacements

### **Short-term (1-2 days)**

1. **Fix transaction boundaries** (P1.4) - Wrap existing code
2. **Standardize error responses** (P3.9) - Pattern replacement
3. **Add pagination limits** (P5.16) - Add query parameters

---

## **Summary Statistics**

- **Total Issues:** 20
- **✅ Completed:** 3 (P0 critical issues)
- **🔄 Remaining:** 17
- **High Priority (P1-P2):** 5 issues
- **Medium Priority (P3-P4):** 7 issues  
- **Low Priority (P5-P6):** 5 issues

## **Recommended Implementation Order**

### **Week 1: Data Integrity & Performance**

- [ ] P1.4: Fix transaction rollback gaps
- [ ] P2.8: Add database indexes
- [ ] P1.5: Add CASCADE constraints

### **Week 2: Error Handling & Observability**

- [ ] P3.9: Standardize error responses
- [ ] P3.10: Implement structured logging
- [ ] P3.11: Fix test flakiness

### **Week 3: Code Quality**

- [ ] P4.12: Standardize repository pattern
- [ ] P4.13: Create permission helpers
- [ ] P4.14: Extract configuration constants

### **Week 4: API & Documentation**

- [ ] P5.15: Standardize API responses
- [ ] P5.16: Add pagination
- [ ] P6.18: Generate API documentation

---

*This document should be updated as issues are resolved or new issues are identified.*
*Last updated: August 20, 2025*
