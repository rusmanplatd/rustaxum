#[macro_export]
macro_rules! storage {
    () => {
        $crate::storage::storage().await
    };
    ($disk:expr) => {
        $crate::storage::disk($disk).await
    };
}

#[macro_export]
macro_rules! storage_put {
    ($path:expr, $contents:expr) => {
        $crate::storage::put($path, $contents).await
    };
    ($disk:expr, $path:expr, $contents:expr) => {
        $crate::storage::disk($disk).await?.put($path, $contents).await
    };
}

#[macro_export]
macro_rules! storage_get {
    ($path:expr) => {
        $crate::storage::get($path).await
    };
    ($disk:expr, $path:expr) => {
        $crate::storage::disk($disk).await?.get($path).await
    };
}

#[macro_export]
macro_rules! storage_exists {
    ($path:expr) => {
        $crate::storage::exists($path).await
    };
    ($disk:expr, $path:expr) => {
        $crate::storage::disk($disk).await?.exists($path).await
    };
}

#[macro_export]
macro_rules! storage_delete {
    ($path:expr) => {
        $crate::storage::delete($path).await
    };
    ($disk:expr, $path:expr) => {
        $crate::storage::disk($disk).await?.delete($path).await
    };
}

#[macro_export]
macro_rules! storage_url {
    ($path:expr) => {
        $crate::storage::url($path).await
    };
    ($disk:expr, $path:expr) => {
        $crate::storage::disk($disk).await?.url($path).await
    };
}

#[macro_export]
macro_rules! storage_files {
    ($directory:expr) => {
        $crate::storage::files($directory).await
    };
    ($disk:expr, $directory:expr) => {
        $crate::storage::disk($disk).await?.files($directory).await
    };
}