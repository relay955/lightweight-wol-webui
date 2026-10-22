use std::{path::PathBuf, process::Command};
use std::os::windows::process::CommandExt;
use serde::{Deserialize, Serialize};
use crate::config::AgentResult;

const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
enum ServerCommand {
    #[serde(rename = "shutdown")]
    Shutdown { id: String },
}

#[derive(Serialize)]
pub struct CommandResult {
    pub r#type: &'static str,
    pub id: String,
    pub success: bool,
    pub error: Option<String>,
}

/// 종료 명령을 실행하고 OS의 요청 처리 결과를 반환합니다.
pub fn handle_message(text: &str, shutdown: impl FnOnce() -> AgentResult<()>) -> Option<CommandResult> {
    let ServerCommand::Shutdown { id } = serde_json::from_str(text).ok()?;
    let result = shutdown();
    Some(CommandResult {
        r#type: "command_result", id, success: result.is_ok(), error: result.err().map(|e| e.to_string()),
    })
}

/// Windows 기본 종료 프로그램으로 앱을 강제 종료하고 시스템 종료를 요청합니다.
pub fn shutdown_computer() -> AgentResult<()> {
    let root = std::env::var_os("SystemRoot").ok_or("Windows 시스템 경로를 찾을 수 없습니다.")?;
    let status = Command::new(PathBuf::from(root).join("System32").join("shutdown.exe"))
        .args(["/s", "/f", "/t", "0"]).creation_flags(CREATE_NO_WINDOW).status()
        .map_err(|_| "Windows 종료 프로그램을 실행할 수 없습니다.")?;
    if !status.success() { return Err("Windows가 종료 요청을 거부했습니다.".into()); }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_shutdown_and_reports_execution_result() {
        let text = serde_json::json!({"type": "shutdown", "id": "test"});
        let response = handle_message(&text.to_string(), || Ok(())).unwrap();
        assert!(response.success);
        assert_eq!(response.id, "test");
        let response = handle_message(&text.to_string(), || Err("mock failure".into())).unwrap();
        assert!(!response.success);
        assert_eq!(response.error.as_deref(), Some("mock failure"));
        for text in [r#"{"type":"reboot"}"#, "shutdown", r#"{"type":"shell","cmd":"anything"}"#] {
            assert!(handle_message(text, || panic!("Unknown command executed")).is_none());
        }
    }
}
