use std::{
    net::TcpListener,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};

use tauri::Manager;
use tokio::process::{Child, Command};

use crate::injection::TASKBOARD_MODULE_ID;

pub struct ModuleService {
    child: Child,
    port: u16,
    url: String,
}

impl ModuleService {
    pub async fn start_taskboard(app: &tauri::AppHandle, module_id: &str) -> Result<Self, String> {
        if module_id != TASKBOARD_MODULE_ID {
            return Err(format!("模块不包含本地服务：{module_id}"));
        }
        let module_dir = bundled_path(
            app,
            "../builtin-modules/taskboard",
            "builtin-modules/taskboard",
        )?;
        Self::start(
            app,
            module_id,
            &module_dir,
            Path::new("service/index.mjs"),
            "/health",
            10_000,
            false,
        )
        .await
    }

    pub async fn start_installed(
        app: &tauri::AppHandle,
        module_id: &str,
        module_dir: &Path,
        entry: &str,
        health_path: &str,
        ready_timeout_ms: u64,
    ) -> Result<Self, String> {
        Self::start(
            app,
            module_id,
            module_dir,
            Path::new(entry),
            health_path,
            ready_timeout_ms,
            true,
        )
        .await
    }

    async fn start(
        app: &tauri::AppHandle,
        module_id: &str,
        module_dir: &Path,
        entry: &Path,
        health_path: &str,
        ready_timeout_ms: u64,
        expose_token: bool,
    ) -> Result<Self, String> {
        let listener = TcpListener::bind(("127.0.0.1", 0)).map_err(|error| error.to_string())?;
        let port = listener
            .local_addr()
            .map_err(|error| error.to_string())?
            .port();
        drop(listener);

        #[cfg(target_os = "windows")]
        let node_relative = "resources/node/node.exe";
        #[cfg(not(target_os = "windows"))]
        let node_relative = "resources/node/node";
        let node = bundled_path(app, node_relative, node_relative)?;
        let data_dir = app
            .path()
            .app_data_dir()
            .map_err(|error| error.to_string())?
            .join("Module Data")
            .join(module_id);
        std::fs::create_dir_all(&data_dir).map_err(|error| error.to_string())?;

        let token = uuid::Uuid::new_v4().to_string();
        let mut child = Command::new(node)
            .arg(module_dir.join(entry))
            .current_dir(module_dir)
            .env("CDP_HUB_MODULE_ID", module_id)
            .env("CDP_HUB_MODULE_DIR", &module_dir)
            .env("CDP_HUB_DATA_DIR", &data_dir)
            .env("CDP_HUB_HOST", "127.0.0.1")
            .env("CDP_HUB_PORT", port.to_string())
            .env("CDP_HUB_SESSION_TOKEN", &token)
            .env("CDP_HUB_PRODUCT_ID", "codex")
            .env("CODEX_TASKBOARD_DATA_DIR", &data_dir)
            .env("CODEX_TASKBOARD_HOST", "127.0.0.1")
            .env("CODEX_TASKBOARD_PORT", port.to_string())
            .env("NODE_NO_WARNINGS", "1")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|error| format!("无法启动模块服务：{error}"))?;

        let origin = format!("http://127.0.0.1:{port}");
        let url = if expose_token {
            format!("{origin}/?sessionToken={token}")
        } else {
            format!("{origin}/")
        };
        let deadline = tokio::time::Instant::now() + Duration::from_millis(ready_timeout_ms);
        while tokio::time::Instant::now() < deadline {
            if let Some(status) = child.try_wait().map_err(|error| error.to_string())? {
                return Err(format!("任务看板服务提前退出：{status}"));
            }
            if reqwest::get(format!("{origin}{health_path}"))
                .await
                .map(|response| response.status().is_success())
                .unwrap_or(false)
            {
                return Ok(Self { child, port, url });
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        let _ = child.kill().await;
        Err("等待模块服务就绪超时".into())
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn stop(mut self) {
        #[cfg(unix)]
        if let Some(pid) = self.child.id() {
            // SAFETY: the PID belongs to the child created above and is still owned here.
            unsafe {
                libc::kill(pid as i32, libc::SIGTERM);
            }
        }
        if tokio::time::timeout(Duration::from_secs(2), self.child.wait())
            .await
            .is_err()
        {
            let _ = self.child.kill().await;
        }
    }
}

fn bundled_path(
    app: &tauri::AppHandle,
    development_path: &str,
    resource_path: &str,
) -> Result<PathBuf, String> {
    #[cfg(debug_assertions)]
    {
        let _ = app;
        let _ = resource_path;
        Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(development_path))
    }
    #[cfg(not(debug_assertions))]
    {
        app.path()
            .resource_dir()
            .map(|path| path.join(resource_path))
            .map_err(|error| error.to_string())
    }
}
