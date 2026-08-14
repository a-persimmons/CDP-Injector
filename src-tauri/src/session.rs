use std::{collections::BTreeMap, sync::Arc, time::Duration};

use serde_json::json;

use crate::{
    cdp::{list_targets, matches_target, CdpConnection},
    model::TargetContext,
};

pub struct ProductSession {
    pub port: u16,
    contexts: Vec<TargetContext>,
    connections: BTreeMap<String, Arc<CdpConnection>>,
    script_ids: BTreeMap<(String, String), String>,
    sources: BTreeMap<String, String>,
}

impl ProductSession {
    pub fn new(port: u16, contexts: Vec<TargetContext>) -> Self {
        Self {
            port,
            contexts,
            connections: BTreeMap::new(),
            script_ids: BTreeMap::new(),
            sources: BTreeMap::new(),
        }
    }

    pub async fn wait_for_target(&mut self) -> Result<(), String> {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
        // ponytail: polling is enough for the first Codex target; use CDP target
        // discovery events if measured replacement latency becomes a problem.
        while tokio::time::Instant::now() < deadline {
            if self.refresh_targets().await.unwrap_or(0) > 0 {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
        Err("等待 Codex CDP 目标超时".into())
    }

    pub async fn refresh_targets(&mut self) -> Result<usize, String> {
        let targets = list_targets(self.port)
            .await
            .map_err(|error| error.to_string())?;
        let matching: Vec<_> = targets
            .into_iter()
            .filter(|target| {
                self.contexts
                    .iter()
                    .any(|context| matches_target(target, context))
            })
            .collect();

        self.connections
            .retain(|target_id, _| matching.iter().any(|target| &target.id == target_id));
        self.script_ids
            .retain(|(_, target_id), _| self.connections.contains_key(target_id));
        for target in matching {
            if !self.connections.contains_key(&target.id) {
                let connection = CdpConnection::connect(&target.web_socket_debugger_url)
                    .await
                    .map_err(|error| error.to_string())?;
                self.connections.insert(target.id, Arc::new(connection));
            }
        }
        Ok(self.connections.len())
    }

    pub async fn probe(&self) -> Result<(), String> {
        let connection = self
            .connections
            .values()
            .next()
            .ok_or("没有匹配的 CDP 目标")?;
        let result = connection
            .send(
                "Runtime.evaluate",
                json!({ "expression": "1 + 1", "returnByValue": true }),
            )
            .await
            .map_err(|error| error.to_string())?;
        let value = result
            .pointer("/result/value")
            .and_then(serde_json::Value::as_i64);
        if value == Some(2) {
            Ok(())
        } else {
            Err("Codex CDP 探测返回异常".into())
        }
    }

    pub async fn install_source(
        &mut self,
        module_id: String,
        source: String,
    ) -> Result<(), String> {
        self.sources.insert(module_id, source);
        self.inject_missing_targets().await
    }

    pub async fn refresh_and_inject(&mut self) -> Result<(), String> {
        self.refresh_targets().await?;
        self.inject_missing_targets().await
    }

    async fn inject_missing_targets(&mut self) -> Result<(), String> {
        let mut pending = Vec::new();
        for (module_id, source) in &self.sources {
            for (target_id, connection) in &self.connections {
                if !self
                    .script_ids
                    .contains_key(&(module_id.clone(), target_id.clone()))
                {
                    pending.push((
                        module_id.clone(),
                        target_id.clone(),
                        source.clone(),
                        connection.clone(),
                    ));
                }
            }
        }

        for (module_id, target_id, source, connection) in pending {
            connection
                .send("Page.enable", json!({}))
                .await
                .map_err(|error| error.to_string())?;
            connection
                .send("Runtime.enable", json!({}))
                .await
                .map_err(|error| error.to_string())?;
            connection
                .send("Page.setBypassCSP", json!({ "enabled": true }))
                .await
                .map_err(|error| error.to_string())?;
            let result = connection
                .send(
                    "Page.addScriptToEvaluateOnNewDocument",
                    json!({ "source": source }),
                )
                .await
                .map_err(|error| error.to_string())?;
            let identifier = result
                .get("identifier")
                .and_then(serde_json::Value::as_str)
                .ok_or("CDP 未返回注入脚本标识")?
                .to_string();
            let current = connection
                .send(
                    "Runtime.evaluate",
                    json!({ "expression": source, "awaitPromise": true }),
                )
                .await
                .map_err(|error| error.to_string())?;
            if let Some(exception) = current.get("exceptionDetails") {
                return Err(format!("模块注入失败：{exception}"));
            }
            if current
                .pointer("/result/value")
                .and_then(serde_json::Value::as_bool)
                != Some(true)
            {
                return Err(format!("模块注入后未在目标页面生效：{module_id}"));
            }
            self.script_ids.insert((module_id, target_id), identifier);
        }
        Ok(())
    }

    pub async fn remove_source(
        &mut self,
        module_id: &str,
        cleanup: serde_json::Value,
    ) -> Result<(), String> {
        self.sources.remove(module_id);
        let installed: Vec<_> = self
            .script_ids
            .iter()
            .filter(|((installed_module, _), _)| installed_module == module_id)
            .map(|((module_id, target_id), identifier)| {
                ((module_id.clone(), target_id.clone()), identifier.clone())
            })
            .collect();
        for (key, identifier) in installed {
            let target_id = &key.1;
            self.script_ids.remove(&key);
            let Some(connection) = self.connections.get(target_id) else {
                continue;
            };
            connection
                .send(
                    "Page.removeScriptToEvaluateOnNewDocument",
                    json!({ "identifier": identifier }),
                )
                .await
                .map_err(|error| error.to_string())?;
            connection
                .send("Runtime.evaluate", cleanup.clone())
                .await
                .map_err(|error| error.to_string())?;
        }
        Ok(())
    }
}
