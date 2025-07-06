# ShiftLinkr Backend Tests

This document describes the comprehensive test suite for the ShiftLinkr backend.

## Test Organization

The backend tests are organized into several test files:

### 1. Authentication Tests (`auth_tests.rs`)
- **Purpose**: Tests core authentication functionality
- **Coverage**: 
  - User registration
  - User login
  - JWT token generation and validation
  - Password hashing and verification
  - Role-based access control
  - Edge cases (duplicate emails, invalid credentials)

### 2. Password Reset Tests (`password_reset_tests.rs`)
- **Purpose**: Tests the password reset flow
- **Coverage**:
  - Token generation and validation
  - Token expiration handling
  - Token usage tracking (one-time use)
  - Password update functionality
  - Security edge cases (invalid tokens, expired tokens)

### 3. Integration Tests (`integration_tests.rs`)
- **Purpose**: Tests HTTP endpoints end-to-end
- **Coverage**:
  - All auth endpoints (`/register`, `/login`, `/me`, `/forgot-password`, `/reset-password`)
  - Request/response validation
  - Error handling
  - Complete password reset flow
  - Authentication middleware

### 4. Configuration Tests (`config_tests.rs`)
- **Purpose**: Tests configuration management
- **Coverage**:
  - Environment variable handling
  - Default value fallbacks
  - Configuration validation
  - Server address formatting

## Running Tests

### Individual Test Suites

```bash
# Run authentication tests
cargo test --test auth_tests

# Run password reset tests
cargo test --test password_reset_tests

# Run integration tests
cargo test --test integration_tests

# Run config tests (single-threaded to avoid env var conflicts)
cargo test --test config_tests -- --test-threads=1
```

### All Tests

Use the provided test runner script:

```bash
./run_tests.sh
```

This script runs all tests with proper isolation and provides a comprehensive summary.

## Test Coverage

The test suite covers:

### Core Functionality
- ✅ User registration and authentication
- ✅ JWT token management
- ✅ Password hashing and verification
- ✅ Password reset flow
- ✅ Role-based access control

### Security Features
- ✅ Password reset token security
- ✅ Token expiration handling
- ✅ One-time token usage
- ✅ Invalid token handling
- ✅ Authentication middleware

### Error Handling
- ✅ Duplicate email registration
- ✅ Invalid login credentials
- ✅ Missing/invalid JWT tokens
- ✅ Expired/invalid reset tokens
- ✅ Malformed requests

### Configuration
- ✅ Environment variable handling
- ✅ Default configuration values
- ✅ Configuration validation

## Test Database

Tests use isolated SQLite databases created in temporary directories. Each test gets its own database instance to prevent interference between tests.

## Test Fixtures

The `common/mod.rs` module provides shared test utilities:
- `TestContext`: Creates isolated test environments
- `setup_test_env()`: Configures test logging

## Continuous Integration

All tests must pass before code can be merged. The test suite is designed to:
- Run quickly and reliably
- Provide clear failure messages
- Test both happy path and error conditions
- Maintain test isolation

## Adding New Tests

When adding new functionality:

1. **Unit Tests**: Add to appropriate test file (e.g., `auth_tests.rs`)
2. **Integration Tests**: Add HTTP endpoint tests to `integration_tests.rs`
3. **Security Tests**: Ensure security edge cases are covered
4. **Documentation**: Update this file with new test coverage

## Test Results

Current test status:
- **Authentication Tests**: ✅ 11 tests passing
- **Password Reset Tests**: ✅ 12 tests passing
- **Integration Tests**: ✅ 10 tests passing
- **Configuration Tests**: ✅ 6 tests passing

**Total**: 39 tests passing

The ShiftLinkr backend is thoroughly tested and ready for production use.
