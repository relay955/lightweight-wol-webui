use std::{collections::{HashMap, HashSet}, sync::{Arc, Mutex}, time::Duration};
use rocket::tokio::{sync::{mpsc, oneshot}, time::timeout};
use serde::Serialize;
use crate::error::SystemError;

#[derive(Serialize)]
pub struct ShutdownCommand {
    pub r#type: &'static str,
    pub id: String,
    pub expires_at: i64,
}

pub struct ShutdownRequest {
    pub command: ShutdownCommand,
    pub result: oneshot::Sender<Result<(), String>>,
}

#[derive(Clone, Default)]
pub struct AgentConnections(Arc<Mutex<HashMap<i64, mpsc::Sender<ShutdownRequest>>>>);

pub struct AgentRegistration {
    connections: AgentConnections,
    id: i64,
    sender: mpsc::Sender<ShutdownRequest>,
}

impl AgentConnections {
    /// 현재 연결된 장치 ID를 한 번에 조회합니다.
    pub fn connected_device_ids(&self) -> HashSet<i64> {
        self.0.lock().unwrap().iter()
            .filter(|(_, sender)| !sender.is_closed()).map(|(&id, _)| id).collect()
    }

    /// 장치의 최신 연결을 등록합니다.
    pub fn register(&self, id: i64) -> (AgentRegistration, mpsc::Receiver<ShutdownRequest>) {
        let (sender, receiver) = mpsc::channel(1);
        self.0.lock().unwrap().insert(id, sender.clone());
        (AgentRegistration { connections: self.clone(), id, sender }, receiver)
    }

    /// 현재 연결에만 종료 요청을 전달하고 에이전트의 실행 결과를 기다립니다.
    pub async fn shutdown(&self, id: i64) -> Result<(), SystemError> {
        let sender = self.0.lock().unwrap().get(&id).cloned()
            .ok_or_else(|| agent_error("에이전트가 연결되어 있지 않습니다."))?;
        let (result, receiver) = oneshot::channel();
        let command = ShutdownCommand {
            r#type: "shutdown", id: uuid::Uuid::new_v4().to_string(),
            expires_at: chrono::Utc::now().timestamp() + 5,
        };
        sender.try_send(ShutdownRequest { command, result })
            .map_err(|_| agent_error("에이전트 연결이 끊겼거나 다른 요청을 처리 중입니다."))?;
        match timeout(Duration::from_secs(8), receiver).await {
            Ok(Ok(Ok(()))) => Ok(()),
            Ok(Ok(Err(message))) => Err(agent_error(&message)),
            Ok(Err(_)) => Err(agent_error("명령 처리 중 에이전트 연결이 끊겼습니다.")),
            Err(_) => Err(agent_error("에이전트 응답 시간이 초과되었습니다. 종료 여부를 확인하세요.")),
        }
    }
}

fn agent_error(message: &str) -> SystemError { SystemError::APIError(503, 0, message.into()) }

impl Drop for AgentRegistration {
    fn drop(&mut self) {
        let mut connections = self.connections.0.lock().unwrap();
        if connections.get(&self.id).is_some_and(|sender| sender.same_channel(&self.sender)) {
            connections.remove(&self.id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rocket::async_test]
    async fn offline_device_returns_error_and_old_disconnect_preserves_new_connection() {
        let connections = AgentConnections::default();
        assert!(connections.shutdown(1).await.is_err());
        let (old, _) = connections.register(1);
        let (_new, mut receiver) = connections.register(1);
        drop(old);
        let response = rocket::tokio::spawn(async move {
            let request = receiver.recv().await.unwrap();
            assert_eq!(request.command.r#type, "shutdown");
            assert!(request.command.expires_at > chrono::Utc::now().timestamp());
            request.result.send(Ok(())).unwrap();
        });
        connections.shutdown(1).await.unwrap();
        response.await.unwrap();
    }
}
