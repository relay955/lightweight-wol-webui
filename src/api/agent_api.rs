use rocket::{http::Status, request::{FromRequest, Outcome}, Request};
use rocket::futures::{SinkExt, StreamExt};
use rocket_db_pools::Database;
use crate::db::Db;
use crate::module::agent::AgentConnections;
use rocket::{State, tokio::time::{timeout, Duration, Instant}};

#[derive(serde::Deserialize)]
struct CommandResult {
    r#type: String,
    id: String,
    success: bool,
    error: Option<String>,
}

pub struct AuthDevice(pub i64);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthDevice {
    type Error = ();

    /// 업그레이드 전에 장치 키를 검증합니다.
    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let headers: Vec<_> = request.headers().get("Authorization").collect();
        let key = match headers.as_slice() {
            [header] => header.strip_prefix("Bearer ").unwrap_or(""),
            _ => "",
        };
        if key.len() != 64 || !key.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Outcome::Error((Status::Unauthorized, ()));
        }
        let Some(db) = Db::fetch(request.rocket()) else {
            return Outcome::Error((Status::InternalServerError, ()));
        };
        let result = sqlx::query_scalar::<_, i64>("SELECT id FROM device WHERE authkey = ?")
            .bind(key).fetch_optional(&db.0).await;
        match result {
            Ok(Some(id)) => Outcome::Success(AuthDevice(id)),
            Ok(None) => Outcome::Error((Status::Unauthorized, ())),
            Err(_) => Outcome::Error((Status::InternalServerError, ())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::{fairing::AdHoc, http::Header, local::asynchronous::Client};
    use crate::db::{migration::migrate, device::generate_authkey};

    #[rocket::async_test]
    async fn requires_device_key_before_websocket_upgrade() {
        let figment = rocket::Config::figment()
            .merge(("databases.sqlite.url", "sqlite::memory:"))
            .merge(("databases.sqlite.max_connections", 1));
        let app = rocket::custom(figment).manage(AgentConnections::default()).attach(Db::init())
            .attach(AdHoc::try_on_ignite("Migrate test DB", |rocket| async {
                migrate(&Db::fetch(&rocket).unwrap().0).await.unwrap();
                Ok(rocket)
            })).mount("/api", routes![connect]);
        let client = Client::tracked(app).await.unwrap();
        let key = generate_authkey();
        let db = Db::fetch(client.rocket()).unwrap();
        sqlx::query("INSERT INTO device (name, mac, ip, order_num, authkey) VALUES ('test', '', '', 1, ?)")
            .bind(&key).execute(&db.0).await.unwrap();
        for auth in [None, Some("Bearer invalid".to_string()), Some(format!("Bearer {}", "0".repeat(64)))] {
            let mut request = client.get("/api/agent/connect");
            if let Some(auth) = auth { request = request.header(Header::new("Authorization", auth)); }
            assert_eq!(request.dispatch().await.status(), Status::Unauthorized);
        }
        let response = client.get("/api/agent/connect")
            .header(Header::new("Authorization", format!("Bearer {key}")))
            .header(Header::new("Connection", "Upgrade"))
            .header(Header::new("Upgrade", "websocket"))
            .header(Header::new("Sec-WebSocket-Version", "13"))
            .header(Header::new("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ=="))
            .dispatch().await;
        // 로컬 클라이언트는 네트워크 업그레이드 전의 응답만 제공
        assert_eq!(response.status(), Status::Ok);
        sqlx::query("DELETE FROM device").execute(&db.0).await.unwrap();
        assert_eq!(client.get("/api/agent/connect")
            .header(Header::new("Authorization", format!("Bearer {key}")))
            .dispatch().await.status(), Status::Unauthorized);
    }

    #[rocket::async_test]
    async fn upgrades_live_connection_and_answers_ping() {
        use tokio_tungstenite::{connect_async, tungstenite::{client::IntoClientRequest, Message}};
        use rocket::tokio::{sync::oneshot, time::{timeout, Duration}};

        let key = generate_authkey();
        let seed_key = key.clone();
        let (ready, port) = oneshot::channel();
        let figment = rocket::Config::figment().merge(("port", 0))
            .merge(("address", "127.0.0.1"))
            .merge(("databases.sqlite.url", "sqlite::memory:"))
            .merge(("databases.sqlite.max_connections", 1));
        let agents = AgentConnections::default();
        let requests = agents.clone();
        let app = rocket::custom(figment).manage(agents).attach(Db::init())
            .attach(AdHoc::try_on_ignite("Seed test device", move |rocket| async move {
                let db = Db::fetch(&rocket).unwrap();
                migrate(&db.0).await.unwrap();
                sqlx::query("INSERT INTO device (name, mac, ip, order_num, authkey)
                    VALUES ('test', '', '', 1, ?)").bind(seed_key).execute(&db.0).await.unwrap();
                Ok(rocket)
            }))
            .attach(AdHoc::on_liftoff("Ready", move |rocket| Box::pin(async move {
                let _ = ready.send(rocket.config().port);
            }))).mount("/api", routes![connect]).ignite().await.unwrap();
        let shutdown = app.shutdown();
        let server = rocket::tokio::spawn(app.launch());
        let result = timeout(Duration::from_secs(10), async {
            let port = port.await.unwrap();
            let mut request = format!("ws://127.0.0.1:{port}/api/agent/connect")
                .into_client_request().unwrap();
            request.headers_mut().insert("Authorization", format!("Bearer {key}").parse().unwrap());
            let (mut socket, response) = connect_async(request).await.unwrap();
            assert_eq!(response.status().as_u16(), 101);
            socket.send(Message::Ping(b"health".to_vec())).await.unwrap();
            assert_eq!(socket.next().await.unwrap().unwrap(), Message::Pong(b"health".to_vec()));
            let command_request = rocket::tokio::spawn(async move { requests.shutdown(1).await });
            let Message::Text(command) = socket.next().await.unwrap().unwrap() else {
                panic!("Expected shutdown command");
            };
            let command: serde_json::Value = serde_json::from_str(&command).unwrap();
            assert_eq!(command["type"], "shutdown");
            socket.send(Message::Text(serde_json::json!({
                "type": "command_result", "id": command["id"], "success": true
            }).to_string())).await.unwrap();
            command_request.await.unwrap().unwrap();
            socket.close(None).await.unwrap();
            assert!(matches!(socket.next().await.unwrap().unwrap(), Message::Close(_)));
        }).await;
        shutdown.notify();
        server.await.unwrap().unwrap();
        result.unwrap();
    }
}

/// 인증된 에이전트의 연결을 유지하고 Ping과 Close에 응답합니다.
#[get("/agent/connect")]
pub fn connect(auth: AuthDevice, ws: rocket_ws::WebSocket, agents: &State<AgentConnections>)
    -> rocket_ws::Channel<'static> {
    let agents = agents.inner().clone();
    ws.config(rocket_ws::Config {
        max_message_size: Some(64 * 1024), max_frame_size: Some(64 * 1024), ..Default::default()
    }).channel(move |mut stream| Box::pin(async move {
        let (_registration, mut commands) = agents.register(auth.0);
        let mut pending: Option<crate::module::agent::ShutdownRequest> = None;
        let mut last_message = Instant::now();
        let mut heartbeat = rocket::tokio::time::interval(Duration::from_secs(1));
        loop {
            rocket::tokio::select! {
                command = commands.recv() => {
                    let Some(command) = command else { break; };
                    if pending.is_some() {
                        let _ = command.result.send(Err("다른 종료 요청을 처리 중입니다.".into()));
                        continue;
                    }
                    let text = serde_json::to_string(&command.command).expect("Serializable command");
                    let sent = timeout(Duration::from_secs(3), stream.send(rocket_ws::Message::Text(text)));
                    match sent.await {
                        Ok(result) => result?,
                        Err(_) => break,
                    }
                    pending = Some(command);
                }
                message = stream.next() => {
                    let Some(message) = message else { break; };
                    last_message = Instant::now();
                    match message? {
                        rocket_ws::Message::Ping(_) => stream.flush().await?,
                        rocket_ws::Message::Close(_) => { stream.flush().await?; break; }
                        rocket_ws::Message::Text(text) => {
                            let Ok(result) = serde_json::from_str::<CommandResult>(&text) else { continue; };
                            if result.r#type != "command_result"
                                || !pending.as_ref().is_some_and(|p| p.command.id == result.id) { continue; }
                            let command = pending.take().unwrap();
                            let result = if result.success { Ok(()) } else {
                                Err(result.error.unwrap_or_else(|| "컴퓨터 종료 요청에 실패했습니다.".into()))
                            };
                            let _ = command.result.send(result);
                        }
                        _ => {},
                    }
                }
                _ = heartbeat.tick() => {
                    if last_message.elapsed() >= Duration::from_secs(90) { break; }
                    if pending.as_ref().is_some_and(|p| p.result.is_closed()) { pending = None; }
                }
            }
        }
        Ok(())
    }))
}
