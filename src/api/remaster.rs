use super::SunoClient;
use super::types::{Clip, GenerateRequest};
use crate::errors::CliError;

impl SunoClient {
    /// Remaster a clip with a different model version.
    /// Posts to `/api/generate/v2-web/` with the remaster model key and
    /// `cover_clip_id` pointing to the original. As with `cover()`, this is
    /// a best-guess port pending a real captured remaster request.
    pub async fn remaster(
        &self,
        clip_id: &str,
        remaster_model_key: &str,
        token: Option<String>,
        token_provider: Option<String>,
    ) -> Result<Vec<Clip>, CliError> {
        let mut req = GenerateRequest::new(remaster_model_key, "remaster");
        req.cover_clip_id = Some(clip_id.to_string());
        req.token = token;
        req.token_provider = token_provider;
        self.generate(&req).await
    }
}
