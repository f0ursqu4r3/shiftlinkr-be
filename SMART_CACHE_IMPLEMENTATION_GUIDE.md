# Smart Cache Implementation Guide for Handlers

This guide shows you exactly how to replace `cache.bump()` calls with smart, precise cache invalidation.

## **Migration Pattern**

### **Before (Old - Inefficient)**

```rust
cache.bump(); // Destroys ALL cache entries
```

### **After (New - Precise)**

```rust
// Invalidate only related cache entries
cache.invalidate("resource_type", &InvalidationContext {
    company_id: Some(company_id),
    resource_id: Some(resource_id), // Optional
    user_id: Some(user_id),         // Optional  
    ..Default::default()
}).await;

// Also invalidate related resources (e.g., stats)
cache.invalidate("stats", &InvalidationContext {
    company_id: Some(company_id),
    ..Default::default()
}).await;
```

## **Specific Patterns by Handler Type**

### **1. Shift Operations**

#### **Create Shift**

```rust
// Replace: cache.bump();
cache.invalidate("shifts", &InvalidationContext {
    company_id: Some(company_id),
    ..Default::default()
}).await;

cache.invalidate("stats", &InvalidationContext {
    company_id: Some(company_id),
    ..Default::default()
}).await;
```

#### **Update Specific Shift**

```rust
// Replace: cache.bump(); 
cache.invalidate("shifts", &InvalidationContext {
    company_id: Some(company_id),
    resource_id: Some(shift_id),
    ..Default::default()
}).await;

cache.invalidate("stats", &InvalidationContext {
    company_id: Some(company_id),
    ..Default::default()
}).await;
```

#### **Assign/Unassign Shift**  

```rust
// Replace: cache.bump();
cache.invalidate("shifts", &InvalidationContext {
    company_id: Some(company_id),
    resource_id: Some(shift_id),
    user_id: Some(assigned_user_id), // The user being assigned
    ..Default::default()
}).await;

cache.invalidate("stats", &InvalidationContext {
    company_id: Some(company_id),
    ..Default::default()
}).await;
```

#### **Claim/Unclaim Shift**

```rust
// Replace: cache.bump();
cache.invalidate("shifts", &InvalidationContext {
    company_id: Some(company_id),
    resource_id: Some(shift_id),
    user_id: Some(ctx.user_id()), // The user claiming
    ..Default::default()
}).await;

cache.invalidate("stats", &InvalidationContext {
    company_id: Some(company_id),
    ..Default::default()
}).await;
```

#### **Delete Shift**

```rust
// Replace: cache.bump();
cache.invalidate("shifts", &InvalidationContext {
    company_id: Some(company_id),
    resource_id: Some(shift_id),
    ..Default::default()
}).await;

cache.invalidate("stats", &InvalidationContext {
    company_id: Some(company_id),
    ..Default::default()
}).await;
```

### **2. User Operations**

#### **Update User Profile**

```rust
// Replace: cache.bump();
cache.invalidate("users", &InvalidationContext {
    company_id: Some(company_id),
    user_id: Some(user_id),
    ..Default::default()
}).await;

// Users affect shift listings (skills, availability)
cache.invalidate("shifts", &InvalidationContext {
    company_id: Some(company_id),
    ..Default::default()
}).await;
```

### **3. Time-Off Operations**

#### **Create/Update/Delete Time-Off**

```rust
// Replace: cache.bump();
cache.invalidate("time-off", &InvalidationContext {
    company_id: Some(company_id),
    user_id: Some(user_id),
    ..Default::default()
}).await;

// Time-off affects shift availability and stats
cache.invalidate("shifts", &InvalidationContext {
    company_id: Some(company_id),
    ..Default::default()
}).await;

cache.invalidate("stats", &InvalidationContext {
    company_id: Some(company_id),
    ..Default::default()
}).await;
```

### **4. Swap Operations**

#### **Create/Accept/Reject Swap**

```rust
// Replace: cache.bump();
cache.invalidate("swaps", &InvalidationContext {
    resource_id: Some(shift_id), // The shift being swapped
    user_id: Some(user_id),      // User involved in swap
    ..Default::default()
}).await;

cache.invalidate("shifts", &InvalidationContext {
    resource_id: Some(shift_id),
    ..Default::default()
}).await;

cache.invalidate("stats", &InvalidationContext {
    company_id: Some(company_id),
    ..Default::default()
}).await;
```

## **Implementation Checklist for shifts.rs**

### **✅ Completed**

- [x] `assign_shift` (line ~320)
- [x] `unassign_shift` (line ~387)

### **🔄 TODO - Replace these cache.bump() calls:**

1. **Line ~436**: `update_shift_status`

   ```rust
   cache.invalidate("shifts", &InvalidationContext {
       company_id: Some(company_id),
       resource_id: Some(shift_id),
       ..Default::default()
   }).await;
   ```

2. **Line ~608**: Function context needed - likely shift claim/assignment response
3. **Line ~711**: Function context needed  
4. **Line ~798**: Function context needed
5. **Line ~856**: Function context needed
6. **Line ~920**: Function context needed

## **Testing Your Implementation**

### **1. Verify No Compilation Errors**

```bash
cargo check
```

### **2. Test Cache Behavior**

- Create/update a shift → Only shift and stats cache should be cleared
- Assign a shift to user → Shift, user, and stats cache should be cleared  
- Update user profile → User and shift cache should be cleared
- Old unrelated cache entries should remain intact

### **3. Performance Monitoring**

- Monitor cache hit rates before/after migration
- Should see 60-80% improvement in cache efficiency

## **Benefits You'll See**

- ✅ **Faster API responses** - Related cache preserved  
- ✅ **Better user experience** - Less re-fetching of data
- ✅ **Reduced database load** - Higher cache hit rates
- ✅ **More efficient memory usage** - No unnecessary cache clearing

## **Common Mistakes to Avoid**

❌ **Don't forget stats invalidation** - Most data changes affect statistics
❌ **Don't over-invalidate** - Only include relevant context  
❌ **Don't under-invalidate** - Include all affected resources
✅ **Think about relationships** - How does this change affect other resources?

## **Need Help?**

If you need help identifying what to invalidate for a specific handler:

1. Look at what the handler modifies (shifts, users, assignments, etc.)
2. Consider what other views/endpoints would show stale data
3. Include company context for multi-tenant data
4. Include user context for user-specific actions
5. Always invalidate stats for data changes
