use super::SunoClient;
use crate::errors::CliError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct UploadAudioRequest {
    extension: String,
}

/// Response from POST /api/uploads/audio/ — S3 presigned POST credentials.
#[derive(Debug, Deserialize, Serialize)]
pub struct UploadAudioResponse {
    /// The upload ID assigned by Suno
    pub id: String,
    /// S3 endpoint URL (e.g. https://suno-uploads.s3.amazonaws.com/)
    pub url: String,
    /// S3 presigned POST fields — must be sent as multipart form-data
    pub fields: S3Fields,
    /// Whether the file has been uploaded yet
    pub is_file_uploaded: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct S3Fields {
    #[serde(rename = "Content-Type")]
    pub content_type: String,
    pub key: String,
    #[serde(rename = "AWSAccessKeyId")]
    pub aws_access_key_id: String,
    pub policy: String,
    pub signature: String,
}

/// Response from GET /api/uploads/audio/{id}/ — upload processing status.
#[derive(Debug, Deserialize, Serialize)]
pub struct UploadStatusResponse {
    pub id: String,
    pub status: String,
    #[serde(default)]
    pub title: String,
}

impl SunoClient {
    /// Step 1: Request an upload slot for an audio file.
    /// POST /api/uploads/audio/ with {"extension": "mp3"}
    /// Returns S3 presigned POST credentials.
    pub async fn upload_audio_init(
        &self,
        extension: &str,
    ) -> Result<UploadAudioResponse, CliError> {
        let body = UploadAudioRequest {
            extension: extension.to_string(),
        };
        let resp = self.post("/api/uploads/audio/").json(&body).send().await?;
        let resp = self.check_response(resp).await?;
        let upload_resp: UploadAudioResponse = resp.json().await.map_err(|e| CliError::Api {
            code: "parse_error",
            message: format!("Failed to parse upload response: {e}"),
        })?;
        Ok(upload_resp)
    }

    /// Step 2: Upload the actual audio bytes to S3 via presigned POST (multipart form-data).
    pub async fn upload_audio_to_s3(
        &self,
        init: &UploadAudioResponse,
        data: Vec<u8>,
    ) -> Result<(), CliError> {
        let file_name = init.fields.key.rsplit('/').next().unwrap_or("audio.mp3");

        let file_part = reqwest::multipart::Part::bytes(data)
            .file_name(file_name.to_string())
            .mime_str(&init.fields.content_type)
            .map_err(|e| CliError::Api {
                code: "upload_failed",
                message: format!("invalid upload MIME type: {e}"),
            })?;

        let form = reqwest::multipart::Form::new()
            .text("Content-Type", init.fields.content_type.clone())
            .text("key", init.fields.key.clone())
            .text("AWSAccessKeyId", init.fields.aws_access_key_id.clone())
            .text("policy", init.fields.policy.clone())
            .text("signature", init.fields.signature.clone())
            .part("file", file_part);

        // S3 uploads can be slow; use a generous per-request timeout.
        let resp = self
            .client
            .post(&init.url)
            .timeout(std::time::Duration::from_secs(300))
            .multipart(form)
            .send()
            .await
            .map_err(|e| CliError::Api {
                code: "upload_failed",
                message: format!("S3 upload failed: {e}"),
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(CliError::Api {
                code: "upload_failed",
                message: format!("S3 upload failed (HTTP {status}): {body}"),
            });
        }

        Ok(())
    }

    /// Step 3: Notify Suno that the upload is complete.
    pub async fn upload_audio_finish(
        &self,
        upload_id: &str,
        filename: &str,
    ) -> Result<(), CliError> {
        let body = serde_json::json!({
            "upload_type": "audio",
            "upload_filename": filename,
        });
        let resp = self
            .post(&format!("/api/uploads/audio/{upload_id}/upload-finish/"))
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(CliError::Api {
                code: "upload_finish_failed",
                message: format!("upload-finish failed (HTTP {status}): {body}"),
            });
        }

        Ok(())
    }

    /// Query upload processing status. A 404 or an explicit `deleted` state
    /// means the upload disappeared after processing/moderation, so callers
    /// can surface the rejection immediately rather than waiting to timeout.
    pub async fn upload_audio_status(
        &self,
        upload_id: &str,
    ) -> Result<Option<UploadStatusResponse>, CliError> {
        let resp = self
            .get(&format!("/api/uploads/audio/{upload_id}/"))
            .send()
            .await?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(CliError::Api {
                code: "upload_status_failed",
                message: format!("upload status check failed (HTTP {status}): {body}"),
            });
        }

        let status_resp: UploadStatusResponse = resp.json().await.map_err(|e| CliError::Api {
            code: "parse_error",
            message: format!("Failed to parse upload status: {e}"),
        })?;

        if status_resp.status == "deleted" {
            return Ok(None);
        }

        Ok(Some(status_resp))
    }

    /// Convert an upload_id to a clip_id by calling the initialize-clip endpoint.
    pub async fn initialize_clip(&self, upload_id: &str) -> Result<String, CliError> {
        let resp = self
            .post(&format!("/api/uploads/audio/{upload_id}/initialize-clip/"))
            .json(&serde_json::json!({ "user_reviewed_tags": true }))
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(CliError::Api {
                code: "initialize_clip_failed",
                message: format!("initialize-clip failed (HTTP {status}): {body}"),
            });
        }

        let value: serde_json::Value = resp.json().await.map_err(|e| CliError::Api {
            code: "parse_error",
            message: format!("Failed to parse initialize-clip response: {e}"),
        })?;
        let clip_id = value
            .get("clip_id")
            .or_else(|| value.get("id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| CliError::Api {
                code: "parse_error",
                message: format!("initialize-clip response did not contain a clip id: {value}"),
            })?;
        Ok(clip_id.to_string())
    }
}
