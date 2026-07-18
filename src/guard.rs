//! Duplicate guard: prevent credit-burning or self-mutating operations
//! (generate, describe, cover, remaster, update) from running twice
//! concurrently — agent retries and parallel agents are the failure mode.
//! Lock file with PID + timestamp under the data dir; locks from dead
//! processes or older than one hour are stale and overwritten.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::errors::CliError;

#[derive(Serialize, Deserialize)]
struct LockFile {
    pid: u32,
    started_at: String,
    operation: String,
}

// Longer than any --wait (poll timeout defaults to 600s) so a live lock
// always means a live operation.
const STALE_THRESHOLD_SECS: i64 = 3600;

pub struct DuplicateGuard {
    lock_path: PathBuf,
    /// Only a guard that actually wrote the lock may delete it — otherwise a
    /// failed acquire would remove the *other* process's live lock on Drop,
    /// letting the very next retry through.
    acquired: bool,
}

impl DuplicateGuard {
    pub fn new(data_dir: &std::path::Path, operation: &str) -> Self {
        let lock_dir = data_dir.join("locks");
        let _ = std::fs::create_dir_all(&lock_dir);
        Self {
            lock_path: lock_dir.join(format!("{operation}.lock")),
            acquired: false,
        }
    }

    /// Returns Ok(()) when it is safe to proceed and writes a fresh lock.
    /// A live, fresh lock from another process is exit 3 unless --force.
    pub fn acquire(&mut self, force: bool) -> Result<(), CliError> {
        if let Ok(contents) = std::fs::read_to_string(&self.lock_path)
            && let Ok(lock) = serde_json::from_str::<LockFile>(&contents)
        {
            let pid_alive = unsafe { libc::kill(lock.pid as i32, 0) == 0 };
            // Unparseable timestamps count as stale.
            let is_stale = chrono::DateTime::parse_from_rfc3339(&lock.started_at)
                .map(|t| {
                    chrono::Utc::now().signed_duration_since(t).num_seconds() > STALE_THRESHOLD_SECS
                })
                .unwrap_or(true);

            if pid_alive && !is_stale && !force {
                return Err(CliError::InvalidInput(format!(
                    "Operation '{}' already running (pid {}). Use --force to override.",
                    lock.operation, lock.pid
                )));
            }
        }

        let lock = LockFile {
            pid: std::process::id(),
            started_at: chrono::Utc::now().to_rfc3339(),
            operation: self
                .lock_path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into(),
        };
        std::fs::write(&self.lock_path, serde_json::to_string(&lock)?)?;
        self.acquired = true;
        Ok(())
    }

    /// Also called on Drop, so early returns and panics still clean up.
    pub fn release(&mut self) {
        if self.acquired {
            let _ = std::fs::remove_file(&self.lock_path);
            self.acquired = false;
        }
    }
}

impl Drop for DuplicateGuard {
    fn drop(&mut self) {
        self.release();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn second_acquire_conflicts_and_force_overrides() {
        let tmp = tempfile::tempdir().unwrap();
        let mut first = DuplicateGuard::new(tmp.path(), "op");
        first.acquire(false).unwrap();

        let mut second = DuplicateGuard::new(tmp.path(), "op");
        let err = second.acquire(false).unwrap_err();
        assert_eq!(err.exit_code(), 3);

        second.acquire(true).unwrap();
    }

    #[test]
    fn lock_released_on_drop() {
        let tmp = tempfile::tempdir().unwrap();
        let lock_path = {
            let mut guard = DuplicateGuard::new(tmp.path(), "op");
            guard.acquire(false).unwrap();
            guard.lock_path.clone()
        };
        assert!(!lock_path.exists());
    }

    #[test]
    fn failed_acquire_does_not_release_holders_lock() {
        // The agent-retry scenario: retry #1 must conflict AND leave the
        // holder's lock intact, so retry #2 still conflicts.
        let tmp = tempfile::tempdir().unwrap();
        let mut holder = DuplicateGuard::new(tmp.path(), "op");
        holder.acquire(false).unwrap();

        {
            let mut retry = DuplicateGuard::new(tmp.path(), "op");
            retry.acquire(false).unwrap_err();
        }
        assert!(holder.lock_path.exists());

        let mut retry2 = DuplicateGuard::new(tmp.path(), "op");
        retry2.acquire(false).unwrap_err();
    }

    #[test]
    fn stale_lock_is_overwritten() {
        let tmp = tempfile::tempdir().unwrap();
        let lock_dir = tmp.path().join("locks");
        std::fs::create_dir_all(&lock_dir).unwrap();
        let two_hours_ago = chrono::Utc::now() - chrono::Duration::hours(2);
        let stale = serde_json::json!({
            "pid": std::process::id(),
            "started_at": two_hours_ago.to_rfc3339(),
            "operation": "op",
        });
        std::fs::write(lock_dir.join("op.lock"), stale.to_string()).unwrap();

        let mut guard = DuplicateGuard::new(tmp.path(), "op");
        guard.acquire(false).unwrap();
    }
}
