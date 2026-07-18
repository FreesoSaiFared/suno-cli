use serde::Serialize;

#[derive(Serialize)]
pub struct Envelope<T: Serialize> {
    pub version: &'static str,
    pub status: &'static str,
    pub data: T,
}

pub fn success<T: Serialize>(data: T) {
    with_status("success", data);
}

/// Success-family envelope with a non-default status: "no_results" for empty
/// list/search hits, "partial_success" when some items of a batch failed.
pub fn with_status<T: Serialize>(status: &'static str, data: T) {
    let envelope = Envelope {
        version: "1",
        status,
        data,
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&envelope).unwrap_or_default()
    );
}

/// Wrap clap's --help / --version text in a success envelope so piped help
/// stays machine-parseable (framework: help is data, not an error).
pub fn help(usage: &str) {
    success(serde_json::json!({ "usage": usage }));
}

pub fn error(code: &str, message: &str, suggestion: &str) {
    let envelope = serde_json::json!({
        "version": "1",
        "status": "error",
        "error": {
            "code": code,
            "message": message,
            "suggestion": suggestion,
        }
    });
    eprintln!(
        "{}",
        serde_json::to_string_pretty(&envelope).unwrap_or_default()
    );
}
