# RustAxum

A Laravel-inspired web framework built with Rust and Axum.

## Project Structure

```
src/
├── main.rs                 # Application entry point
├── app/                    # Application logic (Laravel's app/)
│   ├── controllers/        # HTTP controllers
│   ├── middleware/         # Custom middleware
│   ├── models/            # Data models
│   └── services/          # Business logic services
├── routes/                 # Route definitions
│   ├── api.rs             # API routes
│   └── web.rs             # Web routes
├── config/                 # Configuration
└── database/              # Database related files
    ├── migrations/        # Database migrations
    └── seeders/          # Database seeders

storage/                   # Storage directory
├── logs/                  # Application logs
└── uploads/              # File uploads

tests/                     # Test files
├── unit/                  # Unit tests
└── integration/          # Integration tests
```

## Features

- **Laravel-like Structure**: Familiar directory organization for Laravel developers
- **Controllers**: Organized HTTP request handlers
- **Models**: Data models with SQLx integration
- **Middleware**: Custom middleware for authentication, CORS, etc.
- **Services**: Business logic separation
- **Configuration**: Environment-based configuration
- **Database**: PostgreSQL with SQLx and migrations

## Getting Started

### Option 1: Docker (Recommended)

1. **Start the application with Docker Compose**:

   ```bash
   docker-compose up -d
   ```

2. **The application will be available at**:
   - API: `http://localhost:3000`
   - Database Admin (Adminer): `http://localhost:8080`
   - Email Testing (Mailpit): `http://localhost:8025`

3. **Stop the application**:

   ```bash
   docker-compose down
   ```

### Option 2: Local Development

1. **Install dependencies**:

   ```bash
   cargo build
   ```

2. **Set up environment**:

   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

3. **Run the application**:

   ```bash
   cargo run
   ```

The server will start on `http://localhost:3000`.

## API Endpoints

### Authentication

- `POST /api/auth/login` - User login
- `POST /api/auth/register` - User registration

### Users

- `GET /api/users` - List all users
- `GET /api/users/:id` - Get user by ID

### Web Routes

- `GET /` - Welcome page
- `GET /health` - Health check

## Environment Variables

See `.env.example` for all available configuration options.

## Database Setup

### With Docker

The PostgreSQL database is automatically set up when using `docker-compose up`.

### Local Development

1. Create a PostgreSQL database
2. Update `DATABASE_URL` in your `.env` file
3. Run migrations (when implemented)

## Docker Services

The `docker-compose.yaml` includes:

- **app**: The main Rust application
- **db**: PostgreSQL 16 database
- **redis**: Redis for caching/sessions (optional)
- **adminer**: Web-based database administration tool
- **mailpit**: Email testing tool for development

### Docker Commands

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f app

# Stop all services
docker-compose down

# Rebuild and start
docker-compose up --build -d

# Remove everything including volumes
docker-compose down -v
```

## Development

The project follows Laravel conventions adapted for Rust:

- **Controllers**: Handle HTTP requests and responses
- **Models**: Represent data structures and database interactions
- **Services**: Contain business logic
- **Middleware**: Handle cross-cutting concerns
- **Routes**: Define URL patterns and handlers

## Dependencies

- **axum**: Web framework
- **tokio**: Async runtime
- **diesel**: Database ORM
- **serde**: Serialization
- **tower**: Middleware
- **tracing**: Logging
- **bcrypt**: Password hashing
- **jsonwebtoken**: JWT authentication
- **ulid**: Sortable unique identifiers

```bash
# login
curl -X 'POST' \
  'http://localhost:3000/api/auth/login' \
  -H 'accept: */*' \
  -H 'Content-Type: application/json' \
  -d '{
  "email": "admin@example.com",
  "password": "password"
}' | jq
```

```bash
# Get current user info
curl -X 'GET' \
  'http://localhost:3000/api/me' \
  -H 'accept: */*' \
  -H 'Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIwMUs2NVdSRkVFRTZDMFhXNlk3OUdLMjk4MyIsImV4cCI6MTc1OTA3MzU3MSwiaWF0IjoxNzU4OTg3MTcxLCJqdGkiOiIwMUs2NVdXRFJBMEM3MktBN01KM0NNM0NXWCJ9.7i8ck35bZPuu6VhqnCFSK8q5VyyIxaQWxA8shFg9tJA' | jq
```
