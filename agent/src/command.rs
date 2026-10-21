use std::{path::PathBuf, process::Command, time::{SystemTime, UNIX_EPOCH}};
use std::os::windows::process::CommandExt;
use serde::{Deserialize, Serialize};
use crate::config::AgentResult;

const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
enum ServerCommand {
    #[serde(rename = "shutdown")]
    Shutdown { id: String, expires_at: u64 },
}

#[derive(Serialize)]
pub struct CommandResult {
    pub r#type: &'static str,
    pub id: String,
    pub success: bool,
    pub error: Option<String>,
}

/// 만료되지 않은 종료 명령만 실행하고 OS의 요청 처리 결과를 반환합니다.
pub fn handle_message(text: &str, shutdown: impl FnOnce() -> AgentResult<()>) -> Option<CommandResult> {
    let ServerCommand::Shutdown { id, expires_at } = serde_json::from_str(text).ok()?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
    let result = if expires_at <= now || expires_at > now + 10 {
        Err("종료 명령이 만료되었거나 유효하지 않습니다.".into())
    } else {
        shutdown()
    };
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
    fn accepts_only_unexpired_shutdown_and_reports_execution_result() {
        let expires_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 5;
        let text = serde_json::json!({"type": "shutdown", "id": "test", "expires_at": expires_at});
        let response = handle_message(&text.to_string(), || Ok(())).unwrap();
        assert!(response.success);
        assert_eq!(response.id, "test");
        let response = handle_message(&text.to_string(), || Err("mock failure".into())).unwrap();
        assert!(!response.success);
        assert_eq!(response.error.as_deref(), Some("mock failure"));
        let expired = r#"{"type":"shutdown","id":"old","expires_at":1}"#;
        assert!(!handle_message(expired, || panic!("Expired command executed")).unwrap().success);
        for text in [r#"{"type":"reboot"}"#, "shutdown", r#"{"type":"shell","cmd":"anything"}"#] {
            assert!(handle_message(text, || panic!("Unknown command executed")).is_none());
        }
    }
}
