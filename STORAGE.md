# Storage and Filesystem Documentation

This Laravel-inspired storage system provides a unified API for working with files across different storage drivers (local filesystem, S3, etc.).

## Configuration

Add storage configuration to your `.env` file:

```env
# Default storage disk
FILESYSTEM_DISK=local

# Local storage paths
STORAGE_LOCAL_ROOT=storage/app
STORAGE_PUBLIC_ROOT=storage/app/public

# For public disk URL generation
APP_URL=http://localhost:3000

# AWS S3 Configuration (optional)
AWS_ACCESS_KEY_ID=your_access_key
AWS_SECRET_ACCESS_KEY=your_secret_key
AWS_DEFAULT_REGION=us-east-1
AWS_BUCKET=your_bucket_name
AWS_ENDPOINT=https://s3.amazonaws.com

# MinIO Configuration (S3-compatible, included in docker-compose)
S3_ENDPOINT=http://localhost:9000
S3_ACCESS_KEY=rustaxum_access
S3_SECRET_KEY=rustaxum_secret_key_change_in_production
S3_BUCKET=rustaxum-storage
S3_REGION=us-east-1

# Cloudflare R2 Configuration (S3-compatible)
R2_ENDPOINT=https://account-id.r2.cloudflarestorage.com
R2_ACCESS_KEY=your_r2_access_key
R2_SECRET_KEY=your_r2_secret_key
R2_BUCKET=your_r2_bucket_name
R2_REGION=auto

# Google Cloud Storage Configuration
GCS_PROJECT_ID=your_project_id
GCS_BUCKET=your_gcs_bucket_name
GCS_SERVICE_ACCOUNT_KEY=path/to/service-account.json

# Azure Blob Storage Configuration
AZURE_STORAGE_ACCOUNT=your_storage_account
AZURE_STORAGE_KEY=your_access_key
AZURE_CONTAINER=your_container_name
```

## Basic Usage

### Simple File Operations

```rust
use rustaxum::storage;

// Store a file
let content = b"Hello, World!";
storage::put("hello.txt", content).await?;

// Read a file
let content = storage::get("hello.txt").await?;
let text = String::from_utf8(content)?;

// Check if file exists
if storage::exists("hello.txt").await? {
    println!("File exists!");
}

// Delete a file
storage::delete("hello.txt").await?;

// Copy a file
storage::copy("source.txt", "destination.txt").await?;

// Move a file
storage::move_file("old-name.txt", "new-name.txt").await?;

// Get file size
let size = storage::size("file.txt").await?;

// Get last modified time
let modified = storage::last_modified("file.txt").await?;
```

### Directory Operations

```rust
use rustaxum::storage;

// Create a directory
storage::make_directory("uploads").await?;

// List files in a directory
let files = storage::files("uploads").await?;

// List directories
let dirs = storage::directories("uploads").await?;

// Delete a directory and all its contents
storage::delete_directory("uploads").await?;
```

### Working with Specific Disks

```rust
use rustaxum::storage::{StorageManager, manager};

// Get a specific disk
let disk = manager::disk("public").await?;
disk.put("image.jpg", &image_data).await?;

// Get URL for public files
if let Some(url) = disk.url("image.jpg").await? {
    println!("File URL: {}", url);
}

// Use storage manager for multiple operations
let mut manager = StorageManager::new().await?;
let local_disk = manager.disk("local").await?;
let s3_disk = manager.disk("minio").await?; // or "s3", "r2"

// Copy from local to cloud storage
let content = local_disk.get("private-file.txt").await?;
s3_disk.put("backups/file.txt", &content).await?;

// Work with different cloud providers
let gcs_disk = manager.disk("gcs").await?;
let azure_disk = manager.disk("azure").await?;

// Upload to multiple providers
gcs_disk.put("data/important.json", &json_data).await?;
azure_disk.put("documents/report.pdf", &pdf_data).await?;
```

### Using Macros (Convenience)

```rust
use rustaxum::{storage_put, storage_get, storage_exists};

// Store file using macro
storage_put!("hello.txt", b"Hello, World!").await?;

// Store to specific disk
storage_put!("public", "image.jpg", &image_data).await?;
storage_put!("minio", "backups/data.json", &json_data).await?;
storage_put!("gcs", "analytics/logs.txt", &log_data).await?;

// Read file using macro
let content = storage_get!("hello.txt").await?;

// Read from specific disk
let content = storage_get!("public", "image.jpg").await?;
let backup = storage_get!("minio", "backups/data.json").await?;

// Check existence
if storage_exists!("hello.txt").await? {
    println!("File exists!");
}
```

## File Information

```rust
use rustaxum::storage::manager;

let disk = manager::default_disk().await?;
let info = disk.get_info("document.pdf").await?;

println!("Path: {}", info.path);
println!("Size: {} bytes", info.size);
println!("Modified: {}", info.last_modified);
println!("MIME type: {:?}", info.mime_type);
println!("Is file: {}", info.is_file);
println!("Is directory: {}", info.is_directory);
```

## Available Disk Drivers

### Local Driver

Stores files on the local filesystem.

**Configuration:**
- `driver`: "local"
- `root`: Base directory path
- `visibility`: "private" or "public"
- `url`: URL prefix for public files (optional)

### S3-Compatible Drivers

Works with Amazon S3, MinIO, Cloudflare R2, and other S3-compatible services.

**S3 Configuration:**
- `driver`: "s3"
- `bucket`: S3 bucket name
- `region`: AWS region
- `key`: AWS access key ID
- `secret`: AWS secret access key
- `endpoint`: S3 endpoint (optional, defaults to AWS)
- `url`: Custom URL prefix (optional)

**MinIO Configuration (included in docker-compose):**
- Uses the same S3 driver with MinIO-specific endpoint
- Automatically configured when using `docker-compose up`
- Access via `minio` disk name

**Cloudflare R2 Configuration:**
- Uses the same S3 driver with R2-specific endpoint
- Region should be set to "auto" for R2
- Access via `r2` disk name

### Google Cloud Storage Driver

Stores files on Google Cloud Storage.

**Configuration:**
- `driver`: "gcs"
- `bucket`: GCS bucket name
- `key`: GCP project ID
- `secret`: Service account key JSON (optional, uses ADC if not provided)
- `url`: Custom URL prefix (optional)

### Azure Blob Storage Driver

Stores files on Azure Blob Storage.

**Configuration:**
- `driver`: "azure"
- `bucket`: Azure container name
- `key`: Storage account name
- `secret`: Access key
- `url`: Custom URL prefix (optional)

## Docker Services

When using `docker-compose up`, the following storage services are available:

- **MinIO**: S3-compatible object storage at `http://localhost:9000` (API) and `http://localhost:9001` (Console)
- **PostgreSQL**: Database at `localhost:5432`
- **Redis**: Cache/sessions at `localhost:6379`
- **Mailpit**: Email testing at `http://localhost:8025`

## Example: Multi-Cloud File Upload Handler

```rust
use axum::{extract::Multipart, response::Json};
use rustaxum::storage::{StorageManager, manager};
use serde_json::{json, Value};

pub async fn upload_file(mut multipart: Multipart) -> Result<Json<Value>, String> {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap_or("file");
        let filename = field.file_name().map(|f| f.to_string());
        let data = field.bytes().await.unwrap();

        if let Some(filename) = filename {
            let path = format!("uploads/{}", filename);

            // Store in multiple locations for redundancy
            let local_disk = manager::disk("local").await.map_err(|e| e.to_string())?;
            let cloud_disk = manager::disk("minio").await.map_err(|e| e.to_string())?; // or "s3", "gcs", "azure"

            // Store locally first
            local_disk.put(&path, &data).await.map_err(|e| e.to_string())?;

            // Then backup to cloud
            cloud_disk.put(&format!("backups/{}", path), &data).await
                .map_err(|e| e.to_string())?;

            // Generate URL for public access
            let url = local_disk.url(&path).await.unwrap_or(None);

            return Ok(Json(json!({
                "success": true,
                "path": path,
                "url": url,
                "size": data.len(),
                "backup": "cloud"
            })));
        }
    }

    Err("No file provided".to_string())
}
```

## Example: Multi-Cloud Image Processing

```rust
use rustaxum::storage::{StorageManager, manager};

pub async fn process_and_store_image(image_data: Vec<u8>, filename: &str) -> anyhow::Result<String> {
    // Store original image locally
    let original_path = format!("images/original/{}", filename);
    let local_disk = manager::disk("local").await?;
    local_disk.put(&original_path, &image_data).await?;

    // Process image (resize, optimize, etc.)
    // let processed_data = process_image(&image_data)?;

    // Store processed image to public disk
    let public_path = format!("images/processed/{}", filename);
    let public_disk = manager::disk("public").await?;
    public_disk.put(&public_path, &image_data).await?;

    // Backup to cloud storage (MinIO/S3)
    let cloud_disk = manager::disk("minio").await?;
    cloud_disk.put(&format!("images/backup/{}", filename), &image_data).await?;

    // Store thumbnails in different locations
    // let thumbnail_data = create_thumbnail(&image_data)?;
    // gcs_disk.put(&format!("thumbnails/{}", filename), &thumbnail_data).await?;

    // Return public URL
    let url = public_disk.url(&public_path).await?
        .unwrap_or_else(|| format!("/storage/{}", public_path));

    Ok(url)
}
```

## Error Handling

The storage system provides detailed error types:

```rust
use rustaxum::storage::FilesystemError;

match storage::get("nonexistent.txt").await {
    Ok(content) => println!("File content: {:?}", content),
    Err(e) => {
        if let Some(fs_error) = e.downcast_ref::<FilesystemError>() {
            match fs_error {
                FilesystemError::FileNotFound { path } => {
                    println!("File not found: {}", path);
                }
                FilesystemError::PermissionDenied { path } => {
                    println!("Permission denied: {}", path);
                }
                FilesystemError::DiskFull => {
                    println!("Disk is full!");
                }
                _ => println!("Other filesystem error: {}", fs_error),
            }
        }
    }
}
```

## Advanced Configuration

You can create custom disk configurations:

```rust
use rustaxum::storage::{StorageManager, filesystem::DiskConfig};
use std::collections::HashMap;

let mut disks = HashMap::new();
disks.insert("custom".to_string(), DiskConfig {
    driver: "local".to_string(),
    root: Some("/custom/path".to_string()),
    visibility: Some("private".to_string()),
    throw: Some(false),
    url: None,
    endpoint: None,
    bucket: None,
    region: None,
    key: None,
    secret: None,
});

let config = StorageConfig {
    default: "custom".to_string(),
    disks,
};

let manager = StorageManager::from_config(config).await;
```

## Quick Start with Docker

1. **Start the services:**
   ```bash
   docker-compose up -d
   ```

2. **Configure your environment:**
   ```bash
   # Use MinIO (included in docker-compose)
   FILESYSTEM_DISK=minio

   # Or use local storage
   FILESYSTEM_DISK=local
   ```

3. **Access MinIO Console:**
   - Visit `http://localhost:9001`
   - Login with `rustaxum_access` / `rustaxum_secret_key_change_in_production`
   - View your `rustaxum-storage` bucket

4. **Use in your application:**
   ```rust
   use rustaxum::storage;

   // Store file (automatically uses configured disk)
   storage::put("test.txt", b"Hello MinIO!").await?;

   // Or specify disk explicitly
   let minio_disk = rustaxum::storage::manager::disk("minio").await?;
   minio_disk.put("backup/test.txt", b"Hello MinIO backup!").await?;
   ```

## Production Considerations

- **Security**: Change default MinIO credentials in production
- **Backup Strategy**: Use multiple storage providers for redundancy
- **Performance**: Choose storage providers closest to your users
- **Cost**: Monitor storage costs across different providers
- **Compliance**: Ensure data residency requirements are met

This multi-cloud storage system provides a Laravel-like interface for file operations while supporting multiple cloud providers and taking advantage of Rust's type safety and async capabilities.