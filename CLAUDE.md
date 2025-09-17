# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Building and Running
```bash
# Build the project
cargo build

# Run the application locally
cargo run

# Run with release optimizations
cargo build --release
cargo run --release
```

### Docker Development (Recommended)
```bash
# Start all services (app, PostgreSQL, Redis, Adminer)
docker-compose up -d

# View application logs
docker-compose logs -f app

# Rebuild and restart after code changes
docker-compose up --build -d

# Stop all services
docker-compose down

# Remove everything including volumes
docker-compose down -v
```

### Testing
Currently no test framework is configured. When adding tests, follow Rust conventions with `cargo test`.

## Architecture Overview

This is a Laravel-inspired Rust web framework built with Axum, following familiar MVC patterns adapted for Rust.

### Core Architecture Principles

**Layered Architecture**: The application follows a clear separation of concerns with distinct layers:
- **Routes** (`src/routes/`): Define URL patterns and map to controllers
- **Controllers** (`src/app/controllers/`): Handle HTTP requests/responses, delegate to services
- **Services** (`src/app/services/`): Contain business logic, interact with models
- **Models** (`src/app/models/`): Data structures and database interactions
- **Middleware** (`src/app/middleware/`): Cross-cutting concerns (auth, CORS, logging)

**Configuration Management**: Environment-based configuration through `src/config/mod.rs` with the `Config` struct that loads from environment variables with sensible defaults.

**Modular Route Organization**:
- API routes in `src/routes/api.rs` (prefixed with `/api/`)
- Web routes in `src/routes/web.rs` (standard HTTP pages)
- Routes are merged in `main.rs` using `Router::merge()`

### Key Implementation Details

**Database Integration**: Uses SQLx with PostgreSQL. Models use ULID for primary keys (stored as TEXT in database). Database migrations are in `src/database/migrations/`.

**Authentication Flow**: JWT-based authentication with bcrypt password hashing. Auth logic is split between `auth_controller.rs` and `auth_service.rs`.

**Middleware Stack**: Applied globally in `main.rs` using Tower's `ServiceBuilder`:
- Tracing for request logging
- Permissive CORS (can be customized in `src/app/middleware/cors.rs`)

**Error Handling**: Uses `anyhow::Result` for error propagation throughout the application.

### Development Patterns

**Adding New Features**:
1. Create model in `src/app/models/`
2. Create service for business logic in `src/app/services/`
3. Create controller in `src/app/controllers/`
4. Add routes in appropriate route file
5. Register controller module in `src/app/controllers/mod.rs`

**Environment Configuration**: Copy `.env.example` to `.env` and modify. All config is centralized in the `Config` struct with environment variable fallbacks.

**Database Changes**: Add SQL migration files to `src/database/migrations/` following the naming convention `XXX_description.sql`.

## Service Access

When running with Docker Compose:
- Application: http://localhost:3000
- Database Admin (Adminer): http://localhost:8080
- PostgreSQL: localhost:5432
- Redis: localhost:6379

## Important Files

- `src/main.rs`: Application entry point, server setup, middleware configuration
- `src/config/mod.rs`: Environment configuration management
- `src/routes/api.rs` and `src/routes/web.rs`: Route definitions
- `docker-compose.yaml`: Full development stack configuration
- `.env.example`: Environment variable template