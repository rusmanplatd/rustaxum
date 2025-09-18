# Artisan CLI - Laravel-like Commands for Rust Axum

This project includes a Laravel-inspired CLI tool called `artisan` that helps you generate boilerplate code and manage your Rust Axum application.

## Installation

The `artisan` binary is automatically built when you compile the project:

```bash
cargo build
```

## Usage

```bash
cargo run --bin artisan -- <COMMAND>
```

Or after building:

```bash
./target/debug/artisan <COMMAND>
```

## Available Commands

### Make Commands

Generate various application components:

#### Controllers

```bash
# Generate a basic controller
cargo run --bin artisan -- make controller UserController

# Generate a resource controller with CRUD methods
cargo run --bin artisan -- make controller PostController --resource
```

**Resource controllers** include the following methods:

- `index()` - List all resources
- `store()` - Create a new resource
- `show(id)` - Show a specific resource
- `update(id)` - Update a specific resource
- `destroy(id)` - Delete a specific resource

#### Models

```bash
# Generate a basic model
cargo run --bin artisan -- make model User

# Generate a model with a migration
cargo run --bin artisan -- make model Post --migration
```

Generated models include:

- Main struct with ULID primary key
- Create/Update request structs
- Response struct for API responses
- Database FromRow implementation
- Helper methods

#### Services

```bash
# Generate a service class
cargo run --bin artisan -- make service UserService
```

Services include basic CRUD operations:

- `create()`
- `find_by_id()`
- `update()`
- `delete()`
- `list()`

#### Middleware

```bash
# Generate middleware
cargo run --bin artisan -- make middleware AuthMiddleware
```

#### Migrations

```bash
# Generate a migration
cargo run --bin artisan -- make migration create_users_table

# Generate a migration to add columns
cargo run --bin artisan -- make migration add_email_to_users

# Generate a migration to drop a table
cargo run --bin artisan -- make migration drop_old_table
```

The generator intelligently creates different migration templates based on the name:

- `create_*_table` - Creates a new table with common columns
- `add_*_to_*` - Adds columns to existing table
- `drop_*_table` - Drops a table
- Generic migrations for other patterns

### Database Commands

#### Migrate

```bash
# Run pending migrations
cargo run --bin artisan -- migrate

# Fresh migration (drops all tables and reruns all migrations)
cargo run --bin artisan -- migrate --fresh
```

**Note:** Fresh migrations are currently not implemented for safety reasons.

### Development Commands

#### Serve

```bash
# Start development server on default port (3000)
cargo run --bin artisan -- serve

# Start on custom port
cargo run --bin artisan -- serve --port 8080

# Start on custom host and port
cargo run --bin artisan -- serve --host 0.0.0.0 --port 8080
```

## Examples

### Creating a Blog Post Feature

1. **Generate the model with migration:**

   ```bash
   cargo run --bin artisan -- make model Post --migration
   ```

2. **Generate the service:**

   ```bash
   cargo run --bin artisan -- make service PostService
   ```

3. **Generate the resource controller:**

   ```bash
   cargo run --bin artisan -- make controller PostController --resource
   ```

4. **Run the migration:**

   ```bash
   cargo run --bin artisan -- migrate
   ```

5. **Start the development server:**

   ```bash
   cargo run --bin artisan -- serve
   ```

### Generated File Structure

After running the above commands, you'll have:

```txt
src/
├── app/
│   ├── controllers/
│   │   └── post_controller.rs     # Resource controller with CRUD
│   ├── models/
│   │   └── post.rs                # Post model with ULID
│   └── services/
│       └── post_service.rs        # Service with business logic
├── database/
│   └── migrations/
│       └── 20231201120000_create_posts_table.sql
```

## File Naming Conventions

- **Controllers**: `PascalCase` → `snake_case.rs` (e.g., `UserController` → `user_controller.rs`)
- **Models**: `PascalCase` → `snake_case.rs` (e.g., `User` → `user.rs`)
- **Services**: `PascalCase` → `snake_case.rs` (e.g., `UserService` → `user_service.rs`)
- **Middleware**: `PascalCase` → `snake_case.rs` (e.g., `AuthMiddleware` → `auth_middleware.rs`)
- **Migrations**: `timestamp_snake_case.sql` (e.g., `20231201120000_create_users_table.sql`)

## Module Updates

The CLI automatically updates the appropriate `mod.rs` files when generating new components:

- Controllers are added to `src/app/controllers/mod.rs`
- Models are added to `src/app/models/mod.rs`
- Services are added to `src/app/services/mod.rs`
- Middleware is added to `src/app/middleware/mod.rs`

## Prerequisites

Make sure you have the following dependencies in your `Cargo.toml`:

```toml
[dependencies]
clap = { version = "4.0", features = ["derive"] }
# ... other dependencies
```

## Environment Variables

For database operations, ensure your `.env` file is properly configured:

```env
DATABASE_URL=postgresql://username:password@localhost/database_name
```

## Tips

1. **Use descriptive names**: The generators work best with clear, descriptive names
2. **Follow conventions**: Stick to Laravel/Rails naming conventions for best results
3. **Resource controllers**: Use the `--resource` flag for full CRUD controllers
4. **Migrations with models**: Use `--migration` flag when creating models to generate the database schema
5. **Development workflow**: Use `artisan serve` for development instead of `cargo run`

## Contributing

To add new generators or commands:

1. Add the command to `src/cli/mod.rs`
2. Implement the handler in `src/cli/commands/`
3. Create the generator in `src/cli/generators/` if needed
4. Update this documentation
