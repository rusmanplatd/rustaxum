#[macro_export]
macro_rules! storage {
    () => {
        $crate::storage::storage().await
    };
    ($disk:expr_2021) => {
        $crate::storage::disk($disk).await
    };
}

#[macro_export]
macro_rules! storage_put {
    ($path:expr_2021, $contents:expr_2021) => {
        $crate::storage::put($path, $contents).await
    };
    ($disk:expr_2021, $path:expr_2021, $contents:expr_2021) => {
        $crate::storage::disk($disk).await?.put($path, $contents).await
    };
}

#[macro_export]
macro_rules! storage_get {
    ($path:expr_2021) => {
        $crate::storage::get($path).await
    };
    ($disk:expr_2021, $path:expr_2021) => {
        $crate::storage::disk($disk).await?.get($path).await
    };
}

#[macro_export]
macro_rules! storage_exists {
    ($path:expr_2021) => {
        $crate::storage::exists($path).await
    };
    ($disk:expr_2021, $path:expr_2021) => {
        $crate::storage::disk($disk).await?.exists($path).await
    };
}

#[macro_export]
macro_rules! storage_delete {
    ($path:expr_2021) => {
        $crate::storage::delete($path).await
    };
    ($disk:expr_2021, $path:expr_2021) => {
        $crate::storage::disk($disk).await?.delete($path).await
    };
}

#[macro_export]
macro_rules! storage_url {
    ($path:expr_2021) => {
        $crate::storage::url($path).await
    };
    ($disk:expr_2021, $path:expr_2021) => {
        $crate::storage::disk($disk).await?.url($path).await
    };
}

#[macro_export]
macro_rules! storage_files {
    ($directory:expr_2021) => {
        $crate::storage::files($directory).await
    };
    ($disk:expr_2021, $directory:expr_2021) => {
        $crate::storage::disk($disk).await?.files($directory).await
    };
}