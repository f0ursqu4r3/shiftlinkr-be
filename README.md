# ShiftLinkr Backend API

A Rust-based REST API for the ShiftLinkr employee scheduling system, built with Actix Web and SQLx.

## Features

✅ **Employee Authentication**

- User registration with email/password
- JWT-based authentication
- Role-based access control (Admin, Manager, Employee)
- Secure password hashing with bcrypt

## Tech Stack

- **Framework**: Actix Web 4.11
- **Database**: SQLite with SQLx
- **Authentication**: JWT tokens
- **Password Hashing**: bcrypt
- **Serialization**: Serde JSON

## API Endpoints

### Authentication

#### Register a new user

```bash
POST /api/v1/auth/register
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "password123",
  "name": "John Doe",
  "role": "employee"  // optional: "admin", "manager", "employee"
}
```

#### Login

```bash
POST /api/v1/auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "password123"
}
```

#### Get current user info

```bash
GET /api/v1/auth/me
Authorization: Bearer <jwt_token>
```

### Health Check

#### Health status

```bash
GET /health
```

## Quick Start

1. **Clone and setup environment:**

   ```bash
   cd be
   cp .env.example .env
   # Edit .env with your configuration
   ```

2. **Build the application:**

   ```bash
   cargo build
   ```

3. **Run the server:**

   ```bash
   cargo run
   ```

4. **Test the API:**

   ```bash
   ./test_api.sh
   ```

The server will start on the configured host and port (default: `http://127.0.0.1:8080`) and automatically create a SQLite database with the required tables.

## Database Schema

### Users Table

- `id` (TEXT PRIMARY KEY) - UUID
- `email` (TEXT UNIQUE) - User's email
- `password_hash` (TEXT) - Bcrypt hashed password
- `name` (TEXT) - User's full name
- `role` (TEXT) - User role: "admin", "manager", or "employee"
- `created_at` (DATETIME) - Account creation timestamp
- `updated_at` (DATETIME) - Last update timestamp

## Architecture

The codebase follows a modular architecture:

- **`database/`** - Database models, repositories, and migrations
- **`auth/`** - Authentication service and JWT handling
- **`handlers/`** - HTTP request handlers
- **`main.rs`** - Application entry point and server configuration

## Security Features

- JWT tokens with 30-day expiration
- bcrypt password hashing with secure defaults
- Role-based access control ready for implementation
- Input validation and error handling

## Development

### Environment Variables

The application uses environment variables for configuration. Copy `.env.example` to `.env` and customize as needed:

- `DATABASE_URL` - SQLite database path (default: `sqlite:./shiftlinkr.db`)
- `JWT_SECRET` - JWT signing secret (change this in production!)
- `JWT_EXPIRATION_DAYS` - JWT token expiration time (default: 30 days)
- `HOST` - Server host address (default: `127.0.0.1`)
- `PORT` - Server port number (default: `8080`)
- `RUST_LOG` - Logging configuration (default: `info,be=debug,sqlx=warn`)
- `ENVIRONMENT` - Application environment (default: `development`)

### Testing

Run the test script to verify all endpoints:

```bash
./test_api.sh
```

## Next Steps

Based on the [roadmap](../ROADMAP.md), the next features to implement are:

1. Business Admin Dashboard endpoints
2. Shift management and calendar views
3. Time-off request system
4. Shift swapping functionality

---

**Status**: ✅ MVP Authentication Complete
**Version**: 1.0.0
