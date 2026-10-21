use std::{sync::mpsc::Sender, time::Duration};
use futures_util::{SinkExt, StreamExt};
use tokio::time::{self, Instant, MissedTickBehavior};
use tokio_tungstenite::{connect_async, tungstenite::{client::IntoClientRequest,
    http::{header::AUTHORIZATION, HeaderValue}, Message}};
use crate::config::{AgentResult, ServerConfig};

pub enum TrayEvent {
    Status(String),
    Quit,
}

/// 연결 실패 또는 종료 후 지정된 간격으로 다시 연결합니다.
pub async fn run(config: ServerConfig, events: Sender<TrayEvent>) {
    loop {
        if events.send(TrayEvent::Status("연결 중".into())).is_err() { return; }
        let reason = match connect_and_listen(&config, &events).await {
            Ok(()) => "서버 연결 종료",
            Err(_) => "접속 실패 또는 연결 끊김",
        };
        let status = format!("{reason} ({}초 후 재시도)", config.reconnect_seconds);
        if events.send(TrayEvent::Status(status)).is_err() { return; }
        time::sleep(Duration::from_secs(config.reconnect_seconds)).await;
    }
}

/// 인증 헤더로 접속하고 Ping/Pong으로 연결 생존 여부를 확인합니다.
async fn connect_and_listen(config: &ServerConfig, events: &Sender<TrayEvent>) -> AgentResult<()> {
    let mut request = config.url.as_str().into_client_request()?;
    if !config.token.is_empty() {
        let mut token = HeaderValue::from_str(&format!("Bearer {}", config.token))?;
        token.set_sensitive(true);
        request.headers_mut().insert(AUTHORIZATION, token);
    }
    let (mut socket, _) = time::timeout(Duration::from_secs(10), connect_async(request)).await??;
    events.send(TrayEvent::Status("연결됨".into()))?;
    let mut heartbeat = time::interval(Duration::from_secs(30));
    heartbeat.set_missed_tick_behavior(MissedTickBehavior::Delay);
    let mut last_pong = Instant::now();
    let mut shutdown_requested = false;

    loop {
        tokio::select! {
            _ = heartbeat.tick() => {
                if last_pong.elapsed() >= Duration::from_secs(90) {
                    return Err("서버 응답 시간 초과".into());
                }
                time::timeout(Duration::from_secs(10), socket.send(Message::Ping(Vec::new().into())))
                    .await??;
            }
            message = socket.next() => {
                match message {
                    Some(Ok(Message::Text(text))) => {
                        let result = tokio::task::spawn_blocking(move || {
                            crate::command::handle_message(&text, || {
                                if shutdown_requested { return Err("이미 종료를 요청했습니다.".into()); }
                                crate::command::shutdown_computer()
                            })
                        }).await?;
                        if let Some(result) = result {
                            shutdown_requested |= result.success;
                            let response = serde_json::to_string(&result)?;
                            time::timeout(Duration::from_secs(3), socket.send(Message::Text(response.into())))
                                .await??;
                        }
                    }
                    Some(Ok(Message::Pong(_))) => last_pong = Instant::now(),
                    Some(Ok(Message::Ping(_))) => {
                        // tungstenite가 준비한 Pong 응답 전송
                        time::timeout(Duration::from_secs(10), socket.flush()).await??;
                    }
                    Some(Ok(Message::Close(_))) => {
                        let _ = time::timeout(Duration::from_secs(2), socket.flush()).await;
                        return Ok(());
                    }
                    None => return Ok(()),
                    Some(Err(error)) => return Err(error.into()),
                    Some(Ok(_)) => {},
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use tokio::net::TcpListener;
    use tokio_tungstenite::tungstenite;

    #[tokio::test]
    async fn handshake_sends_token_and_handles_ping_and_close() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let config = ServerConfig {
            url: format!("ws://{}/agent", listener.local_addr().unwrap()),
            token: "test-device-token".into(), reconnect_seconds: 1,
        };
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut socket = tokio_tungstenite::accept_hdr_async(stream,
                |request: &tungstenite::handshake::server::Request, response| {
                    assert_eq!(request.headers()[AUTHORIZATION], "Bearer test-device-token");
                    Ok(response)
                }).await.unwrap();
            socket.send(Message::Ping(b"health".to_vec().into())).await.unwrap();
            loop {
                match socket.next().await.unwrap().unwrap() {
                    Message::Pong(data) if data.as_ref() == b"health" => break,
                    Message::Ping(_) => socket.flush().await.unwrap(),
                    _ => {},
                }
            }
            socket.close(None).await.unwrap();
        });
        let (events, receiver) = mpsc::channel();
        time::timeout(Duration::from_secs(5), connect_and_listen(&config, &events))
            .await.unwrap().unwrap();
        server.await.unwrap();
        assert!(matches!(receiver.try_recv().unwrap(), TrayEvent::Status(status) if status == "연결됨"));
    }

    #[tokio::test]
    async fn reconnects_after_server_disconnects() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let config = ServerConfig {
            url: format!("ws://{}/agent", listener.local_addr().unwrap()),
            token: String::new(), reconnect_seconds: 1,
        };
        let server = tokio::spawn(async move {
            for _ in 0..2 {
                let (stream, _) = listener.accept().await.unwrap();
                let mut socket = tokio_tungstenite::accept_async(stream).await.unwrap();
                socket.send(Message::Ping(b"ready".to_vec().into())).await.unwrap();
                loop {
                    match socket.next().await.unwrap().unwrap() {
                        Message::Pong(data) if data.as_ref() == b"ready" => break,
                        Message::Ping(_) => socket.flush().await.unwrap(),
                        _ => {},
                    }
                }
                socket.close(None).await.unwrap();
            }
        });
        let (events, receiver) = mpsc::channel();
        let client = tokio::spawn(run(config, events));
        let result = time::timeout(Duration::from_secs(5), server).await;
        client.abort();
        result.unwrap().unwrap();
        let connected = receiver.try_iter().filter(|event| {
            matches!(event, TrayEvent::Status(status) if status == "연결됨")
        }).count();
        assert_eq!(connected, 2);
    }
}
