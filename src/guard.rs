//! Duplicate guard: prevent credit-burning or self-mutating operations
//! (generate, describe, extend, cover, remaster, update) from running twice
//! concurrently — agent retries and parallel agents are the failure mode.
//!
//! Acquisition is atomic (`OpenOptions::create_new`), not check-then-write:
//! two simultaneous acquires cannot both win the create, so they cannot both
//! spend credits. A lock is reclaimed only when its owner is provably gone
//! (dead PID), stale, or the caller forces it. Every lock carries a unique
//! per-guard nonce, and release removes the file only while it still holds
//! *our* nonce — so a guard whose lock was forcibly reclaimed by a successor
//! never deletes that successor's live lock.

use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::errors::CliError;

#[derive(Serialize, Deserialize)]
struct LockFile {
    pid: u32,
    started_at: String,
    operation: String,
    /// Unique per-acquiring-guard token. Defaults so pre-nonce lock files
    /// (or hand-written ones) still parse — an absent nonce reads as "" and
    /// never matches a live guard's nonce, so it can only ever be reclaimed.
    #[serde(default)]
    nonce: String,
}

// Time-based staleness backstop for a *live* PID whose owner is wedged.
// Longer than the default --wait (poll timeout 600s). A poll_timeout_secs
// configured above this could see a genuinely-live long wait reclaimed, so
// dead-PID detection — not the clock — is the primary reclaim signal.
const STALE_THRESHOLD_SECS: i64 = 3600;

/// A lock is stale when its timestamp is older than the threshold. An
/// unparseable timestamp counts as stale (corrupt lock → reclaimable).
fn is_stale(started_at: &str) -> bool {
    chrono::DateTime::parse_from_rfc3339(started_at)
        .map(|t| chrono::Utc::now().signed_duration_since(t).num_seconds() > STALE_THRESHOLD_SECS)
        .unwrap_or(true)
}

/// Is `pid` a currently-running process? Probes existence without signalling.
/// A recycled PID can read as alive (same caveat on both platforms), so this
/// is a liveness hint, not proof — the atomic lock file is the real guard.
#[cfg(unix)]
fn process_alive(pid: u32) -> bool {
    // kill(pid, 0) sends no signal; 0 means the process exists, ESRCH means it
    // is gone. A negative/zero pid would target a process group, so reject it.
    if pid == 0 || pid > i32::MAX as u32 {
        return false;
    }
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

#[cfg(windows)]
fn process_alive(pid: u32) -> bool {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
    // A dead PID yields a null handle; a live one opens (then we close it).
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            false
        } else {
            CloseHandle(handle);
            true
        }
    }
}

#[cfg(not(any(unix, windows)))]
fn process_alive(_pid: u32) -> bool {
    // No portable probe: assume alive so we never reclaim a peer's live lock.
    true
}

pub struct DuplicateGuard {
    lock_path: PathBuf,
    operation: String,
    /// Unique to this guard instance; written into the lock so release only
    /// removes a file still owned by us (see the module and release() docs).
    nonce: String,
    /// Only a guard that actually wrote the lock may delete it — otherwise a
    /// failed acquire would remove the *other* process's live lock on Drop,
    /// letting the very next retry through.
    acquired: bool,
}

impl DuplicateGuard {
    pub fn new(data_dir: &Path, operation: &str) -> Self {
        let lock_dir = data_dir.join("locks");
        let _ = std::fs::create_dir_all(&lock_dir);
        Self {
            lock_path: lock_dir.join(format!("{operation}.lock")),
            operation: operation.to_string(),
            nonce: uuid::Uuid::new_v4().to_string(),
            acquired: false,
        }
    }

    /// Returns Ok(()) when it is safe to proceed, having atomically written a
    /// lock stamped with our nonce. A live, fresh lock owned by another
    /// process is exit 3 unless `force`.
    pub fn acquire(&mut self, force: bool) -> Result<(), CliError> {
        // Bounded retry: reclaiming a dead/stale/forced lock is remove-then-
        // create_new, which races other acquirers. A competitor that wins the
        // create turns the next pass into a clean conflict, so this converges;
        // the cap only guards a pathological create/remove ping-pong.
        for _ in 0..16 {
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&self.lock_path)
            {
                Ok(mut file) => {
                    let lock = LockFile {
                        pid: std::process::id(),
                        started_at: chrono::Utc::now().to_rfc3339(),
                        operation: self.operation.clone(),
                        nonce: self.nonce.clone(),
                    };
                    file.write_all(serde_json::to_string(&lock)?.as_bytes())?;
                    self.acquired = true;
                    return Ok(());
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    let held = std::fs::read_to_string(&self.lock_path)
                        .ok()
                        .and_then(|c| serde_json::from_str::<LockFile>(&c).ok());
                    // A dead PID is reclaimable regardless of age; missing or
                    // corrupt lock content is reclaimable too.
                    let reclaimable = match &held {
                        Some(lock) => {
                            force || !process_alive(lock.pid) || is_stale(&lock.started_at)
                        }
                        None => true,
                    };
                    if !reclaimable {
                        let lock = held.expect("Some when not reclaimable");
                        return Err(CliError::InvalidInput(format!(
                            "Operation '{}' already running (pid {}). Use --force to override.",
                            lock.operation, lock.pid
                        )));
                    }
                    // Remove the reclaimable lock and retry the create. A peer
                    // that reclaimed first leaves NotFound — just loop and
                    // re-evaluate whatever now occupies the path.
                    match std::fs::remove_file(&self.lock_path) {
                        Ok(()) => {}
                        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                        Err(e) => return Err(e.into()),
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }
        Err(CliError::InvalidInput(format!(
            "could not acquire '{}' lock — contended lock file at {}",
            self.operation,
            self.lock_path.display()
        )))
    }

    /// Also called on Drop, so early returns and panics still clean up.
    /// Removes the lock only while it still carries our nonce: a successor may
    /// have forcibly reclaimed the path, and we must not delete its live lock.
    pub fn release(&mut self) {
        if !self.acquired {
            return;
        }
        self.acquired = false;
        let still_ours = std::fs::read_to_string(&self.lock_path)
            .ok()
            .and_then(|c| serde_json::from_str::<LockFile>(&c).ok())
            .is_some_and(|lock| lock.nonce == self.nonce);
        if still_ours {
            let _ = std::fs::remove_file(&self.lock_path);
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

    #[test]
    fn concurrent_acquire_only_one_wins() {
        // Atomic create_new: the second acquirer of a live lock must fail, not
        // silently overwrite — otherwise both agents spend credits at once.
        let tmp = tempfile::tempdir().unwrap();
        let mut a = DuplicateGuard::new(tmp.path(), "gen");
        a.acquire(false).unwrap();

        let mut b = DuplicateGuard::new(tmp.path(), "gen");
        let err = b.acquire(false).unwrap_err();
        assert_eq!(
            err.exit_code(),
            3,
            "second concurrent acquire must be exit 3"
        );
        assert!(!b.acquired, "the loser must not believe it holds the lock");
    }

    #[test]
    fn release_does_not_delete_a_reclaimed_lock() {
        // B holds; A forces its way in (stamping A's nonce). When B later
        // releases it must NOT delete A's live lock — the ownership nonce is
        // what stops a stale releaser from freeing a successor's guard.
        let tmp = tempfile::tempdir().unwrap();
        let mut b = DuplicateGuard::new(tmp.path(), "op");
        b.acquire(false).unwrap();

        let mut a = DuplicateGuard::new(tmp.path(), "op");
        a.acquire(true).unwrap();

        b.release();
        assert!(
            a.lock_path.exists(),
            "stale releaser deleted the successor's lock"
        );

        a.release();
        assert!(!a.lock_path.exists());
    }

    #[cfg(unix)]
    #[test]
    fn dead_pid_lock_reclaimed_regardless_of_age() {
        // A fresh lock owned by a dead PID must be reclaimable immediately —
        // dead-process detection, not the clock, is the primary signal.
        let tmp = tempfile::tempdir().unwrap();
        let lock_dir = tmp.path().join("locks");
        std::fs::create_dir_all(&lock_dir).unwrap();

        let mut child = std::process::Command::new("true").spawn().unwrap();
        let dead_pid = child.id();
        child.wait().unwrap();

        let fresh = serde_json::json!({
            "pid": dead_pid,
            "started_at": chrono::Utc::now().to_rfc3339(),
            "operation": "op",
            "nonce": "someone-else",
        });
        std::fs::write(lock_dir.join("op.lock"), fresh.to_string()).unwrap();

        let mut guard = DuplicateGuard::new(tmp.path(), "op");
        guard.acquire(false).unwrap();
    }
}
