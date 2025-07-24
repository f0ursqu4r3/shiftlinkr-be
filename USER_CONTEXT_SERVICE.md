# User Context Service

The `UserContextService` provides a convenient way to inject the current user and their company information into every request. This service extracts user data from JWT claims and provides easy access to user permissions and company information.

## Overview

The service consists of three main components:

1. **UserContext** - A struct containing user data, company info, and role information
2. **UserContextService** - A service that creates UserContext from requests
3. **Helper functions and macros** - Convenient ways to extract user context in handlers

## Key Features

- Automatic user and company data injection from JWT tokens
- Role-based permission checking
- Resource ownership validation  
- Company-scoped operations
- Easy-to-use helper functions and macros

## Usage in Handlers

### Method 1: Direct Service Usage

```rust
use actix_web::{web, HttpRequest, HttpResponse, Result};
use crate::services::UserContextService;

pub async fn my_handler(
    req: HttpRequest,
    user_context_service: web::Data<UserContextService>,
) -> Result<HttpResponse> {
    let user_context = user_context_service
        .extract_context(&req)
        .await
        .map_err(|e| actix_web::error::ErrorUnauthorized(format!("Auth error: {}", e)))?;

    // Use user_context...
    Ok(HttpResponse::Ok().json(user_context.user.name))
}
```

### Method 2: Using the Helper Macro (Recommended)

```rust
use crate::extract_user_context;

pub async fn my_handler(
    req: HttpRequest,
    user_context_service: web::Data<UserContextService>,
) -> Result<HttpResponse> {
    let user_context = extract_user_context!(user_context_service, &req);
    
    // Use user_context...
    Ok(HttpResponse::Ok().json(user_context.user.name))
}
```

### Method 3: Using the Helper Function

```rust
use crate::services::user_context::get_user_context;

pub async fn my_handler(
    req: HttpRequest,
    user_context_service: web::Data<UserContextService>,
) -> Result<HttpResponse> {
    let user_context = get_user_context(&user_context_service, &req).await?;
    
    // Use user_context...
    Ok(HttpResponse::Ok().json(user_context.user.name))
}
```

## UserContext Methods

### Basic Information
- `user_id()` - Get the user's UUID
- `user_email()` - Get the user's email
- `company_id()` - Get the company UUID (if any)
- `company_name()` - Get the company name (if any)
- `role()` - Get the user's role in the current company

### Permission Checking
- `is_admin()` - Check if user is admin
- `is_manager()` - Check if user is manager  
- `is_employee()` - Check if user is employee
- `is_manager_or_admin()` - Check if user is manager or admin
- `has_role(&role)` - Check if user has a specific role

### Resource Access Control
- `can_access_user_resource(user_id)` - Check if user can access another user's resources
- `can_manage_user(user_id)` - Check if user can perform admin operations on another user
- `belongs_to_company(company_id)` - Check if user belongs to a specific company

## Common Patterns

### 1. Admin-Only Operations

```rust
pub async fn admin_operation(
    req: HttpRequest,
    user_context_service: web::Data<UserContextService>,
) -> Result<HttpResponse> {
    let user_context = extract_user_context!(user_context_service, &req);
    
    if !user_context.is_admin() {
        return Ok(HttpResponse::Forbidden().json(json!({
            "error": "Admin access required"
        })));
    }
    
    // Perform admin operation...
    Ok(HttpResponse::Ok().json(json!({"message": "Success"})))
}
```

### 2. Resource Ownership Checks

```rust
pub async fn get_user_data(
    req: HttpRequest,
    user_context_service: web::Data<UserContextService>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let user_context = extract_user_context!(user_context_service, &req);
    let target_user_id = path.into_inner();
    
    if !user_context.can_access_user_resource(target_user_id) {
        return Ok(HttpResponse::Forbidden().json(json!({
            "error": "Cannot access this user's data"
        })));
    }
    
    // Fetch and return user data...
    Ok(HttpResponse::Ok().json(json!({"data": "user_data"})))
}
```

### 3. Company-Scoped Operations

```rust
pub async fn company_stats(
    req: HttpRequest,
    user_context_service: web::Data<UserContextService>,
) -> Result<HttpResponse> {
    let user_context = extract_user_context!(user_context_service, &req);
    
    let company_id = user_context.company_id()
        .ok_or_else(|| actix_web::error::ErrorBadRequest("User must belong to a company"))?;
    
    if !user_context.is_manager_or_admin() {
        return Ok(HttpResponse::Forbidden().json(json!({
            "error": "Manager or admin access required"
        })));
    }
    
    // Get company stats...
    Ok(HttpResponse::Ok().json(json!({
        "company_id": company_id,
        "stats": "company_stats_data"
    })))
}
```

### 4. Role-Based Data Filtering

```rust
pub async fn get_shifts(
    req: HttpRequest,
    user_context_service: web::Data<UserContextService>,
    shift_repo: web::Data<ShiftRepository>,
) -> Result<HttpResponse> {
    let user_context = extract_user_context!(user_context_service, &req);
    
    let shifts = if user_context.is_manager_or_admin() {
        // Managers and admins can see all company shifts
        shift_repo.get_shifts_by_company(user_context.company_id().unwrap()).await?
    } else {
        // Employees can only see their own shifts
        shift_repo.get_shifts_by_user(user_context.user_id()).await?
    };
    
    Ok(HttpResponse::Ok().json(shifts))
}
```

## Error Handling

The service will return errors in the following cases:

1. **Invalid or missing JWT token** - Returns 401 Unauthorized
2. **User not found** - Returns 401 Unauthorized  
3. **Company not found** - Returns 401 Unauthorized
4. **Database errors** - Returns 500 Internal Server Error

## Setup in main.rs

The service is automatically registered in `main.rs`:

```rust
// Service creation
let user_context_service = UserContextService::new(
    user_repository.clone(),
    company_repository.clone(),
);

// Registration with app
.app_data(web::Data::new(user_context_service))
```

## Performance Considerations

- The service makes database calls to fetch user and company data on each request
- Consider implementing caching if performance becomes an issue
- The service reuses existing repository instances to minimize overhead

## Migration from Direct Claims Usage

### Before (using Claims directly):
```rust
pub async fn old_handler(claims: Claims) -> Result<HttpResponse> {
    let user_id = claims.sub;
    // Manual permission checking...
}
```

### After (using UserContext):
```rust
pub async fn new_handler(
    req: HttpRequest,
    user_context_service: web::Data<UserContextService>,
) -> Result<HttpResponse> {
    let user_context = extract_user_context!(user_context_service, &req);
    let user_id = user_context.user_id();
    // Built-in permission checking...
}
```

## Testing

For testing purposes, you can create UserContext directly:

```rust
use crate::services::UserContextService;

let user_context = user_context_service.from_claims(&claims).await?;
// Or for admin operations:
let user_context = user_context_service.get_user_context(user_id, company_id).await?;
```
