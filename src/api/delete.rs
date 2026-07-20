use serde_json::json;

use super::SunoClient;
use crate::errors::CliError;

impl SunoClient {
    /// Move clips into (`trash: true`) or out of (`trash: false`) the trash.
    ///
    /// Route + body verified live on 2026-07-20: `POST /api/gen/trash` with a
    /// top-level `{"clip_ids": [...], "trash": bool}` body, answering
    /// `{"ids": [...], "is_trashed": bool}`. The previous `/api/feed/trash` +
    /// `{"ids": [...]}` now 404s, and despite the 422's `body.spec.*` field
    /// locations a `spec`-wrapped body is rejected — those locs are server
    /// parameter names, not a required wrapper.
    pub async fn set_trashed(&self, ids: &[String], trash: bool) -> Result<(), CliError> {
        self.with_auth_retry(|| async {
            let resp = self
                .post("/api/gen/trash")
                .json(&json!({ "clip_ids": ids, "trash": trash }))
                .send()
                .await?;
            self.check_response(resp).await?;
            Ok(())
        })
        .await
    }
}
