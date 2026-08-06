use std::sync::atomic::{AtomicU64, Ordering};

use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::sync::Mutex;
use tokio_tungstenite::{
    connect_async,
    tungstenite::Message,
    MaybeTlsStream, WebSocketStream,
};

use crate::model::TargetContext;

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetInfo {
    pub id: String,
    #[serde(rename = "type")]
    pub target_type: String,
    pub url: String,
    pub web_socket_debugger_url: String,
}

pub fn matches_target(target: &TargetInfo, context: &TargetContext) -> bool {
    target.target_type == context.target_type
        && context
            .url_prefixes
            .iter()
            .any(|prefix| target.url.starts_with(prefix))
        && !context
            .exclude_url_contains
            .iter()
            .any(|part| target.url.contains(part))
}

#[derive(Debug, thiserror::Error)]
pub enum CdpError {
    #[error("CDP HTTP 请求失败：{0}")]
    Http(#[from] reqwest::Error),
    #[error("CDP WebSocket 失败：{0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("CDP 响应无效：{0}")]
    Json(#[from] serde_json::Error),
    #[error("CDP 连接已关闭")]
    Closed,
    #[error("CDP 命令失败：{0}")]
    Command(String),
}

pub async fn list_targets(port: u16) -> Result<Vec<TargetInfo>, CdpError> {
    Ok(reqwest::get(format!("http://127.0.0.1:{port}/json/list"))
        .await?
        .error_for_status()?
        .json()
        .await?)
}

type Socket = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

pub struct CdpConnection {
    socket: Mutex<Socket>,
    next_id: AtomicU64,
}

impl CdpConnection {
    pub async fn connect(url: &str) -> Result<Self, CdpError> {
        let (socket, _) = connect_async(url).await?;
        Ok(Self {
            socket: Mutex::new(socket),
            next_id: AtomicU64::new(1),
        })
    }

    pub async fn send(&self, method: &str, params: Value) -> Result<Value, CdpError> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let mut socket = self.socket.lock().await;
        socket
            .send(Message::Text(
                json!({ "id": id, "method": method, "params": params })
                    .to_string()
                    .into(),
            ))
            .await?;

        while let Some(message) = socket.next().await {
            let message = message?;
            if !message.is_text() {
                continue;
            }
            let response: Value = serde_json::from_str(message.to_text()?)?;
            if response.get("id").and_then(Value::as_u64) != Some(id) {
                continue;
            }
            if let Some(error) = response.get("error") {
                return Err(CdpError::Command(error.to_string()));
            }
            return Ok(response.get("result").cloned().unwrap_or(Value::Null));
        }
        Err(CdpError::Closed)
    }
}

#[cfg(test)]
mod tests {
    use crate::model::TargetContext;

    use super::{matches_target, TargetInfo};

    #[test]
    fn matches_codex_target() {
        let context = TargetContext {
            id: "main".into(),
            target_type: "page".into(),
            url_prefixes: vec!["app://".into()],
            exclude_url_contains: vec!["global-dictation".into()],
        };
        let target = |target_type: &str, url: &str| TargetInfo {
            id: "target".into(),
            target_type: target_type.into(),
            url: url.into(),
            web_socket_debugger_url: "ws://127.0.0.1/target".into(),
        };

        assert!(matches_target(&target("page", "app://codex"), &context));
        assert!(!matches_target(&target("worker", "app://codex"), &context));
        assert!(!matches_target(&target("page", "https://example.com"), &context));
        assert!(!matches_target(
            &target("page", "app://global-dictation"),
            &context
        ));
    }
}
