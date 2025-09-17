pub mod local;
pub mod s3;
pub mod gcs;
pub mod azure;

pub use local::LocalFilesystem;
pub use s3::S3Filesystem;
pub use gcs::GcsFilesystem;
pub use azure::AzureFilesystem;