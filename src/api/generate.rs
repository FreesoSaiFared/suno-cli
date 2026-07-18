use super::SunoClient;
use super::types::{Clip, GenerateRequest, GenerateResponse};
use crate::errors::CliError;

/// A clip is terminal once it can no longer change: complete or error.
fn is_terminal(status: &str) -> bool {
    matches!(status, "complete" | "error")
}

/// Every requested id has a terminal clip in `clips`. An empty or partial
/// feed is NOT done — a missing id is still pending, so `.all()` over the
/// requested ids (not the returned clips) is what gates polling.
fn all_requested_terminal(ids: &[String], clips: &[Clip]) -> bool {
    ids.iter()
        .all(|id| clips.iter().any(|c| c.id == *id && is_terminal(&c.status)))
}

/// Requested ids that are absent or not yet terminal — reported on timeout.
fn pending_ids(ids: &[String], clips: &[Clip]) -> Vec<String> {
    ids.iter()
        .filter(|id| !clips.iter().any(|c| c.id == **id && is_terminal(&c.status)))
        .cloned()
        .collect()
}

/// Validate a create submission. Suno can answer HTTP 200 with an error
/// status or an empty clip list after quoting credits — either is a failure,
/// never silent success.
fn validate_submission(result: GenerateResponse) -> Result<Vec<Clip>, CliError> {
    if let Some(status) = result.status.as_deref()
        && status.eq_ignore_ascii_case("error")
    {
        return Err(CliError::GenerationFailed(format!(
            "Suno rejected the generation (status: {status})"
        )));
    }
    if result.clips.is_empty() {
        return Err(CliError::GenerationFailed(
            "Suno returned no clips — the generation was not created".into(),
        ));
    }
    Ok(result.clips)
}

impl SunoClient {
    /// Submit a music generation request (custom mode or inspiration mode).
    /// Posts to `/api/generate/v2-web/` — the legacy `/api/generate/v2/`
    /// returns `Token validation failed` since Suno migrated creates to
    /// `v2-web` server-side (verified 2026-04-07).
    /// Wrapped in `with_auth_retry` so a single stale-JWT failure recovers
    /// transparently via Clerk refresh.
    pub async fn generate(&self, req: &GenerateRequest) -> Result<Vec<Clip>, CliError> {
        self.with_auth_retry(|| async {
            let resp = self.post("/api/generate/v2-web/").json(req).send().await?;
            let resp = self.check_response(resp).await?;
            let result: GenerateResponse = resp.json().await?;
            validate_submission(result)
        })
        .await
    }

    /// Poll clip status by IDs until *every requested id* is complete or
    /// errored. A partial feed (only some ids returned, or still streaming)
    /// keeps waiting — returning early would let callers download nothing
    /// while reporting success. `interval_secs` seeds the backoff (config
    /// `poll_interval_secs`), doubling up to 15s or the seed, whichever is
    /// larger.
    pub async fn poll_clips(
        &self,
        ids: &[String],
        timeout_secs: u64,
        interval_secs: u64,
    ) -> Result<Vec<Clip>, CliError> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_secs);
        let mut delay = std::time::Duration::from_secs(interval_secs.max(1));
        let cap = std::time::Duration::from_secs(interval_secs.max(15));

        loop {
            let clips = self.get_clips(ids).await?;

            if all_requested_terminal(ids, &clips) {
                return Ok(clips);
            }
            if start.elapsed() >= timeout {
                let pending = pending_ids(ids, &clips);
                return Err(CliError::GenerationFailed(format!(
                    "generation timed out after {timeout_secs}s; {} of {} clip(s) still pending: {}",
                    pending.len(),
                    ids.len(),
                    pending.join(", ")
                )));
            }
            tokio::time::sleep(delay).await;
            delay = (delay * 2).min(cap);
        }
    }

    /// Fetch clips by IDs. Batches in pairs to avoid Suno's limit
    /// (SunoAI-API #49: 4+ IDs from different batches only returns first 2).
    /// Each chunk is wrapped in `with_auth_retry` so long polling waits
    /// survive Suno's JWT staleness window mid-generation.
    pub async fn get_clips(&self, ids: &[String]) -> Result<Vec<Clip>, CliError> {
        let mut all_clips = Vec::new();
        for chunk in ids.chunks(2) {
            let ids_param = chunk.join(",");
            let path = format!("/api/feed/?ids={ids_param}");
            let clips: Vec<Clip> = self
                .with_auth_retry(|| async {
                    let resp = self.get(&path).send().await?;
                    let resp = self.check_response(resp).await?;
                    let clips: Vec<Clip> = resp.json().await?;
                    Ok(clips)
                })
                .await?;
            all_clips.extend(clips);
        }
        Ok(all_clips)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clip(id: &str, status: &str) -> Clip {
        serde_json::from_value(serde_json::json!({
            "id": id,
            "title": "t",
            "status": status,
            "model_name": "chirp",
            "audio_url": null,
            "video_url": null,
            "image_url": null,
            "created_at": "2026-01-01T00:00:00Z",
        }))
        .unwrap()
    }

    #[test]
    fn submission_error_status_or_empty_clips_is_a_failure() {
        // HTTP 200 with an error status — credits quoted, nothing created.
        let r: GenerateResponse = serde_json::from_str(r#"{"status":"error","clips":[]}"#).unwrap();
        let err = validate_submission(r).unwrap_err();
        assert_eq!(err.error_code(), "generation_failed");

        // 200 with an empty clip list (no status) is equally a failure.
        let r: GenerateResponse = serde_json::from_str(r#"{"clips":[]}"#).unwrap();
        assert!(validate_submission(r).is_err());

        // A real submission returns clips and succeeds.
        let r = GenerateResponse {
            clips: vec![clip("a", "streaming")],
            status: None,
        };
        assert_eq!(validate_submission(r).unwrap().len(), 1);
    }

    #[test]
    fn polling_requires_every_requested_id_terminal() {
        let ids = vec!["a".to_string(), "b".to_string()];

        // Only one of two ids returned at all — the other is still pending.
        let partial = vec![clip("a", "complete")];
        assert!(!all_requested_terminal(&ids, &partial));
        assert_eq!(pending_ids(&ids, &partial), vec!["b".to_string()]);

        // Both present but one still streaming — not done.
        let streaming = vec![clip("a", "complete"), clip("b", "streaming")];
        assert!(!all_requested_terminal(&ids, &streaming));
        assert_eq!(pending_ids(&ids, &streaming), vec!["b".to_string()]);

        // Both terminal (one errored counts) — done, nothing pending.
        let done = vec![clip("a", "complete"), clip("b", "error")];
        assert!(all_requested_terminal(&ids, &done));
        assert!(pending_ids(&ids, &done).is_empty());
    }

    #[test]
    fn empty_feed_is_not_terminal() {
        // The old `.all()` over an empty clip list read as "done" and reported
        // success with nothing generated.
        let ids = vec!["a".to_string()];
        assert!(!all_requested_terminal(&ids, &[]));
        assert_eq!(pending_ids(&ids, &[]), ids);
    }
}
