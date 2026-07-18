//! Hidden command for conformance/contract tests: deterministically triggers
//! each exit code (0-4) without touching network or auth state.

use crate::errors::CliError;
use crate::output::OutputFormat;

pub fn run(fmt: OutputFormat, code: i32) -> Result<(), CliError> {
    match code {
        0 => {
            if let OutputFormat::Json = fmt {
                crate::output::json::success(serde_json::json!({
                    "contract": true,
                    "exit_code": 0,
                }));
            } else {
                println!("contract: success");
            }
            Ok(())
        }
        1 => Err(CliError::Api {
            code: "transient",
            message: "contract: transient error".into(),
        }),
        2 => Err(CliError::Config("contract: config error".into())),
        3 => Err(CliError::InvalidInput("contract: bad input".into())),
        4 => Err(CliError::RateLimited),
        other => Err(CliError::InvalidInput(format!(
            "unknown contract code: {other} (valid: 0-4)"
        ))),
    }
}
