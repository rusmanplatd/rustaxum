# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Naming Rules (Laravel Naming Conventions)
- **Follow Laravel naming exactly**: Use identical class names, method names, and conventions
- **NO marketing adjectives**: Never use "advanced", "enhanced", "complete", "improved", "better", "superior", "features", "optimized", "premium", or similar terms in code names
- **Laravel naming patterns**: Follow Laravel's exact naming patterns for consistency
- **Method signatures**: Match Laravel method signatures exactly, including parameter names and types

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

### Artisan CLI (Laravel-like Commands)
```bash
# Generate components using the artisan CLI

# Core Components
cargo run --bin artisan -- make controller PostController --resource
cargo run --bin artisan -- make model Post --migration
cargo run --bin artisan -- make service PostService
cargo run --bin artisan -- make middleware AuthMiddleware
cargo run --bin artisan -- make migration create_posts_table
cargo run --bin artisan -- make seeder UserSeeder
cargo run --bin artisan -- make request CreatePostRequest

# API Resources
cargo run --bin artisan -- make resource UserResource
cargo run --bin artisan -- make resource PostCollection --collection

# Communication & Events
cargo run --bin artisan -- make mail OrderShipped --markdown
cargo run --bin artisan -- make notification InvoicePaid --markdown
cargo run --bin artisan -- make event UserRegistered
cargo run --bin artisan -- make listener SendWelcomeEmail --event UserRegistered --queued

# Jobs & Background Processing
cargo run --bin artisan -- make job ProcessPayment
cargo run --bin artisan -- make job SendEmail --sync

# Authorization & Validation
cargo run --bin artisan -- make policy PostPolicy --model Post
cargo run --bin artisan -- make rule UppercaseRule

# Testing
cargo run --bin artisan -- make test UserServiceTest --unit
cargo run --bin artisan -- make test PostControllerTest

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

The Artisan CLI provides comprehensive code generation capabilities similar to Laravel's artisan command.

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
- **Controllers** (`src/app/http/controllers/`): Handle HTTP requests/responses, delegate to services
- **Services** (`src/app/services/`): Contain business logic, interact with models
- **Models** (`src/app/models/`): Data structures and database interactions
- **Middleware** (`src/app/http/middleware/`): Cross-cutting concerns (auth, CORS, logging)
- **Resources** (`src/app/resources/`): API response transformations and data presentation
- **Requests** (`src/app/http/requests/`): Input validation and form request handling
- **Mail** (`src/app/mail/`): Email composition and sending logic
- **Notifications** (`src/app/notifications/`): Multi-channel notification system
- **Jobs** (`src/app/jobs/`): Background task processing and queue management
- **Events** (`src/app/events/`): Event broadcasting and application event handling
- **Listeners** (`src/app/listeners/`): Event listeners and handlers
- **Policies** (`src/app/policies/`): Authorization logic and access control
- **Rules** (`src/app/rules/`): Custom validation rules and data validation
- **Tests** (`tests/`): Unit and feature tests for application components

**Configuration Management**: Environment-based configuration through `src/config/mod.rs` with the `Config` struct that loads from environment variables with sensible defaults.

**Modular Route Organization**:
- API routes in `src/routes/api.rs` (prefixed with `/api/`)
- Web routes in `src/routes/web.rs` (standard HTTP pages)
- Routes are merged in `main.rs` using `Router::merge()`

### Key Implementation Details

**Database Integration**: Uses Diesel ORM with PostgreSQL. Models use ULID for primary keys (stored as TEXT in database). Database migrations are in `src/database/migrations/`.

**Authentication Flow**: JWT-based authentication with bcrypt password hashing. Auth logic is split between `auth_controller.rs` and `auth_service.rs`.

**Middleware Stack**: Applied globally in `main.rs` using Tower's `ServiceBuilder`:

- Correlation middleware for request tracking
- Activity logging middleware
- Tracing for request logging
- Permissive CORS

**Error Handling**: Uses `anyhow::Result` for error propagation throughout the application.

### Artisan Make Commands

The framework provides comprehensive Laravel-style generators:

**Core Components**

- `make:controller` - HTTP request handlers with optional `--resource` flag
- `make:model` - Data models with optional `--migration` flag
- `make:service` - Business logic services
- `make:middleware` - HTTP middleware for cross-cutting concerns
- `make:request` - Form request validation classes

**API & Resources**

- `make:resource` - API response transformers with optional `--collection` flag
- `make:migration` - Database schema changes
- `make:seeder` - Database seeding classes

**Communication & Events**

- `make:mail` - Email classes with optional `--markdown` templates
- `make:notification` - Multi-channel notifications with optional `--markdown`
- `make:event` - Application events for broadcasting
- `make:listener` - Event handlers with `--event` and `--queued` options

**Background Processing**

- `make:job` - Background jobs with optional `--sync` flag for immediate execution

**Authorization & Validation**

- `make:policy` - Authorization policies with optional `--model` association
- `make:rule` - Custom validation rules

**Testing**

- `make:test` - Test classes with `--unit` flag for unit vs feature tests

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

**Query Builder Pattern**: Controllers for GET endpoints should use the `QueryBuilderService` pattern. Models must implement the `Queryable` trait with allowed filters, sorts, fields, and includes. GET endpoints accept `QueryParams` and use `<Model as QueryBuilderService<Model>>::index(Query(params), &pool)` for consistent filtering, sorting, pagination, and field selection.

## Service Access

When running with Docker Compose:

- Application: <http://localhost:3000>
- Database Admin (Adminer): <http://localhost:8080>
- Email Testing (Mailpit): <http://localhost:8025>
- PostgreSQL: localhost:5434
- Redis: localhost:6379

## Important Files

- `src/main.rs`: Application entry point, server setup, middleware configuration
- `src/lib.rs`: Library entry point with `create_app()` function
- `src/config/mod.rs`: Environment configuration management
- `src/routes/api.rs` and `src/routes/web.rs`: Route definitions
- `src/routes/oauth/mod.rs`: OAuth2 routes and endpoints
- `src/cli/main.rs`: Artisan CLI entry point
- `src/app/query_builder/`: Query builder system for advanced filtering, sorting, pagination
- `docker-compose.yaml`: Full development stack configuration
- `.env.example`: Environment variable template

## OAuth 2.1 Implementation

This framework includes a comprehensive OAuth 2.1 authorization server implementation with support for multiple RFCs and modern security standards.

### Supported RFC Standards

- **OAuth 2.1 Authorization Framework**: Core authorization server functionality
- **RFC 7636 (PKCE)**: Mandatory for all authorization code flows
- **RFC 8628 (Device Authorization Grant)**: For input-constrained devices (Smart TVs, IoT)
- **RFC 9068 (JWT Profile for Access Tokens)**: Structured JWT tokens with enhanced claims
- **RFC 9449 (DPoP)**: Demonstrating Proof of Possession for enhanced token security
- **RFC 7662 (Token Introspection)**: Token metadata and validation

### OAuth Services and Components

**Core Services** (`src/app/services/oauth/`):
- `TokenService`: JWT token generation, validation, and RFC 9068 compliance
- `ClientService`: OAuth client management and organization-scoped access
- `ScopeService`: Permission scope validation and management
- `DeviceService`: RFC 8628 device authorization grant implementation
- `DPoPService`: RFC 9449 proof of possession validation

**Models** (`src/app/models/oauth/`):
- `AccessToken`: JWT-backed tokens with DPoP binding support
- `RefreshToken`: Rotating refresh tokens
- `AuthCode`: PKCE-enabled authorization codes
- `Client`: OAuth client applications with multi-tenant support
- `DeviceCode`: Device authorization codes for RFC 8628
- `Scope`: Permission scopes

**Controllers** (`src/app/http/controllers/oauth/`):
- `OAuthController`: Core authorization and token endpoints
- `DeviceController`: RFC 8628 device flow endpoints with HTML interface
- `ClientController`: Client management API

**Middleware** (`src/app/http/middleware/`):
- `dpop_middleware`: DPoP token validation for protected resources
- `oauth_middleware`: General OAuth token validation

### Key Features

- **Multi-tenant Organization Support**: Clients scoped to organizations
- **DPoP Token Binding**: Cryptographic binding of tokens to client keys
- **Device Flow**: User-friendly device authorization for smart devices
- **JWT Profile Compliance**: RFC 9068 structured tokens with enhanced claims
- **PKCE Mandatory**: All authorization code flows require PKCE
- **Token Introspection**: RFC 7662 compliant token metadata endpoint

### OAuth Endpoints

**Core OAuth 2.1**:
- `GET /oauth/authorize` - Authorization endpoint (PKCE required)
- `POST /oauth/token` - Token endpoint (supports DPoP)
- `POST /oauth/introspect` - Token introspection
- `POST /oauth/revoke` - Token revocation

**Device Authorization Grant (RFC 8628)**:
- `POST /oauth/device/code` - Device authorization request
- `GET /oauth/device` - User verification interface
- `POST /oauth/device/authorize` - User authorization

**Client Management**:
- `GET /oauth/clients` - List clients
- `POST /oauth/clients` - Create client
- `PUT /oauth/clients/{id}` - Update client
- `DELETE /oauth/clients/{id}` - Delete client

### Security Features

- **OAuth 2.1 Compliance**: Removed insecure flows (implicit, password)
- **Mandatory PKCE**: All flows require proof key for code exchange
- **DPoP Support**: Token binding prevents theft and replay attacks
- **JWT Tokens**: Self-contained tokens with cryptographic validation
- **Organization Scoping**: Multi-tenant client access control
- **Scope-based Authorization**: Fine-grained permission management

# important-instruction-reminders
Do what has been asked; nothing more, nothing less.
NEVER create files unless they're absolutely necessary for achieving your goal.
ALWAYS prefer editing an existing file to creating a new one.
NEVER proactively create documentation files (*.md) or README files. Only create documentation files if explicitly requested by the User.
