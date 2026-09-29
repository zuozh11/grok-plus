use std::path::{Path, PathBuf};

/// Whether `path` resolves to the user's home directory.
pub fn is_home_dir(path: &Path) -> bool {
    let Some(home) = xai_dirs::home_dir() else {
        return false;
    };
    canonicalize_or_owned(path) == canonicalize_or_owned(&home)
}

pub fn canonicalize_or_owned(path: &Path) -> PathBuf {
    dunce::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}
