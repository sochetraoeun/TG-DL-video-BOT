use std::path::{Path, PathBuf};
use tracing;

/// Guard that removes a temporary directory when dropped.
/// Ensures downloaded files don't leak on disk after upload or on error.
pub struct TempDirGuard {
    path: PathBuf,
}

impl TempDirGuard {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        if self.path.exists() {
            if let Err(e) = std::fs::remove_dir_all(&self.path) {
                tracing::warn!(path = %self.path.display(), error = %e, "failed to clean up temp dir");
            } else {
                tracing::debug!(path = %self.path.display(), "cleaned up temp dir");
            }
        }
    }
}
