# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

# Naming Rules (Laravel Naming Conventions)
- **Follow Laravel naming exactly**: Use identical class names, method names, and conventions
- **NO marketing adjectives**: Never use "advanced", "enhanced", "complete", "improved", "better", "superior", "features", "optimized", "premium", or similar terms in code names
- **Laravel naming patterns**: Follow Laravel's exact naming patterns for consistency
- **Method signatures**: Match Laravel method signatures exactly, including parameter names and types

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

### Artisan CLI (Laravel-like Commands)
```bash
# Generate components using the artisan CLI
cargo run --bin artisan -- make controller PostController --resource
cargo run --bin artisan -- make model Post --migration
cargo run --bin artisan -- make service PostService
cargo run --bin artisan -- make middleware AuthMiddleware
cargo run --bin artisan -- make migration create_posts_table
cargo run --bin artisan -- make seeder UserSeeder

# Run database migrations
cargo run --bin artisan -- migrate

# Run migrations with seeding
cargo run --bin artisan -- migrate --seed

# Fresh migrations with seeding
cargo run --bin artisan -- migrate --fresh --seed

# Refresh migrations with seeding
cargo run --bin artisan -- migrate:refresh --seed

# Database seeding commands
cargo run --bin artisan -- db:seed                    # Run all seeders
cargo run --bin artisan -- db:seed --class DatabaseSeeder  # Run specific seeder
cargo run --bin artisan -- db:seed --fresh            # Reset DB and seed
cargo run --bin artisan -- db:seed:list               # List available seeders

# Start development server
cargo run --bin artisan -- serve --port 3000
```

See `ARTISAN.md` for comprehensive CLI documentation.

### Testing
```bash
# Run tests
cargo test

# Run tests with coverage
cargo test --verbose
```

Test dependencies are configured in Cargo.toml: `axum-test`, `tokio-test`, `mockall`, and `serial_test`.

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

**Adding New Features** (using Artisan CLI):
1. `cargo run --bin artisan -- make model ModelName --migration`
2. `cargo run --bin artisan -- make service ModelService`
3. `cargo run --bin artisan -- make controller ModelController --resource`
4. Add routes in appropriate route file
5. `cargo run --bin artisan -- migrate`

**Manual approach** (if not using Artisan):
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
- Email Testing (Mailpit): http://localhost:8025
- PostgreSQL: localhost:5432
- Redis: localhost:6379

## Important Files

- `src/main.rs`: Application entry point, server setup, middleware configuration
- `src/lib.rs`: Library entry point with `create_app()` function
- `src/config/mod.rs`: Environment configuration management
- `src/routes/api.rs` and `src/routes/web.rs`: Route definitions
- `src/cli/main.rs`: Artisan CLI entry point
- `docker-compose.yaml`: Full development stack configuration
- `.env.example`: Environment variable template
- `ARTISAN.md`: Comprehensive CLI documentation