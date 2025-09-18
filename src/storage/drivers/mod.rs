pub mod local;
pub mod s3;

pub use local::LocalFilesystem;
pub use s3::S3Filesystem;