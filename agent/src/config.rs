use std::{env, error::Error, fs, path::PathBuf};
use serde::Deserialize;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use url::Url;

pub type AgentResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub server: ServerConfig,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerConfig {
    pub url: String,
    #[serde(default)]
    pub token: String,
    #[serde(default = "default_reconnect_seconds")]
    pub reconnect_seconds: u64,
}

fn default_reconnect_seconds() -> u64 { 5 }

impl Config {
    /// 실행 파일 옆의 설정을 읽고, 없으면 예제 설정을 생성합니다.
    pub fn load() -> AgentResult<Self> {
        let path: PathBuf = env::current_exe()?.with_file_name("Agent.toml");
        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let example = include_str!("../Agent.example.toml");
                fs::write(&path, example)?;
                example.to_string()
            }
            Err(error) => return Err(error.into()),
        };
        Self::parse(&content)
    }

    /// WSS 주소와 재연결 간격, 인증 헤더를 검증합니다.
    fn parse(content: &str) -> AgentResult<Self> {
        // TOML 파서 오류에 토큰이 포함된 원문이 노출되지 않도록 고정 메시지 사용
        let config: Self = toml::from_str(content).map_err(|_| "Agent.toml 형식이 잘못되었습니다.")?;
        let url = Url::parse(&config.server.url).map_err(|_| "서버 주소가 잘못되었습니다.")?;
        if url.scheme() != "wss" || url.host_str().is_none() {
            return Err("서버 주소는 wss:// 형식이어야 합니다.".into());
        }
        if !url.username().is_empty() || url.password().is_some() || url.query().is_some()
            || url.fragment().is_some() {
            return Err("서버 주소에 인증 정보, 쿼리 또는 fragment를 넣을 수 없습니다.".into());
        }
        if config.server.reconnect_seconds == 0 || config.server.reconnect_seconds > 3600 {
            return Err("reconnect_seconds는 1~3600이어야 합니다.".into());
        }
        if !config.server.token.is_empty() {
            HeaderValue::from_str(&format!("Bearer {}", config.server.token))
                .map_err(|_| "기기 토큰에 사용할 수 없는 문자가 있습니다.")?;
        }
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_and_defaults_are_valid() {
        assert!(Config::parse(include_str!("../Agent.example.toml")).is_ok());
        let config = Config::parse("[server]\nurl = 'wss://localhost/agent'").unwrap();
        assert_eq!(config.server.reconnect_seconds, 5);
        assert!(config.server.token.is_empty());
    }

    #[test]
    fn rejects_insecure_or_sensitive_urls() {
        for url in ["ws://localhost", "https://localhost", "wss://user:pass@localhost",
            "wss://localhost?token=secret", "wss://localhost#fragment"] {
            assert!(Config::parse(&format!("[server]\nurl = '{url}'")).is_err());
        }
    }

    #[test]
    fn rejects_invalid_interval_and_token_without_exposing_secrets() {
        for seconds in [0, 3601] {
            let content = format!("[server]\nurl = 'wss://localhost'\nreconnect_seconds = {seconds}");
            assert!(Config::parse(&content).is_err());
        }
        let content = "[server]\nurl = 'wss://localhost'\ntoken = \"secret\\nvalue\"";
        let error = Config::parse(content).err().unwrap().to_string();
        assert!(!error.contains("secret"));
        assert!(Config::parse(content).is_err());
    }
}
