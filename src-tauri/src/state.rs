use std::{
    collections::BTreeMap,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    sync::Arc,
};

use tokio::sync::Mutex;

use crate::{
    injection::{ORANGE_GLOW_MODULE_ID, TASKBOARD_MODULE_ID, THEME_MODULE_ID},
    model::{ModuleServiceView, ModuleSummary, ProductProfile, ProductStatus, ProductView},
    module_service::ModuleService,
    session::ProductSession,
};

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub enabled_modules: BTreeMap<String, Vec<String>>,
}

pub struct AppStateData {
    pub profiles: Vec<ProductProfile>,
    pub modules: Vec<ModuleSummary>,
    pub statuses: BTreeMap<String, ProductStatus>,
    pub settings: Settings,
}

pub struct AppState {
    pub data: Mutex<AppStateData>,
    pub sessions: Mutex<BTreeMap<String, Arc<Mutex<ProductSession>>>>,
    services: Mutex<BTreeMap<String, ModuleService>>,
    settings_path: PathBuf,
}

impl AppStateData {
    pub fn product_views(&self) -> Vec<ProductView> {
        self.profiles
            .iter()
            .map(|profile| {
                let enabled = self.settings.enabled_modules.get(&profile.id);
                let modules = self
                    .modules
                    .iter()
                    .cloned()
                    .map(|mut module| {
                        module.enabled_for = enabled
                            .filter(|ids| ids.contains(&module.id))
                            .map(|_| vec![profile.id.clone()])
                            .unwrap_or_default();
                        module
                    })
                    .collect();

                ProductView {
                    profile: profile.clone(),
                    modules,
                    services: vec![],
                    status: self
                        .statuses
                        .get(&profile.id)
                        .expect("built-in product has status")
                        .clone(),
                }
            })
            .collect()
    }

    pub fn set_module_enabled(
        &mut self,
        product_id: &str,
        module_id: &str,
        enabled: bool,
    ) -> Result<(), String> {
        if !self.profiles.iter().any(|profile| profile.id == product_id) {
            return Err(format!("未知产品：{product_id}"));
        }
        if !self.modules.iter().any(|module| module.id == module_id) {
            return Err(format!("未知模块：{module_id}"));
        }

        let modules = self
            .settings
            .enabled_modules
            .entry(product_id.to_string())
            .or_default();
        if enabled && !modules.iter().any(|id| id == module_id) {
            modules.push(module_id.to_string());
        } else if !enabled {
            modules.retain(|id| id != module_id);
        }
        Ok(())
    }

    pub fn set_product_phase(&mut self, product_id: &str, phase: &str) -> Result<(), String> {
        let status = self
            .statuses
            .get_mut(product_id)
            .ok_or_else(|| format!("未知产品：{product_id}"))?;
        status.phase = phase.to_string();
        Ok(())
    }

    pub fn reconcile_product_running(
        &mut self,
        product_id: &str,
        running: bool,
    ) -> Result<bool, String> {
        let status = self
            .statuses
            .get_mut(product_id)
            .ok_or_else(|| format!("未知产品：{product_id}"))?;
        let launching = matches!(
            status.phase.as_str(),
            "stopping" | "starting" | "connecting to CDP" | "injecting"
        );
        if !running && !launching {
            status.phase = "not running".into();
            status.launch_mode = None;
            status.cdp_status = "not used".into();
            return Ok(true);
        }
        if running && status.phase == "not running" {
            status.phase = "running normally".into();
        }
        Ok(false)
    }
}

impl AppState {
    pub fn load(config_dir: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let profile: ProductProfile =
            serde_json::from_str(include_str!("../resources/products/codex.json"))?;
        let settings_path = config_dir.join("settings.json");
        let settings = match fs::read_to_string(&settings_path) {
            Ok(json) => serde_json::from_str(&json)?,
            Err(error) if error.kind() == ErrorKind::NotFound => Settings::default(),
            Err(error) => return Err(error.into()),
        };
        let product_id = profile.id.clone();

        Ok(Self {
            data: Mutex::new(AppStateData {
                profiles: vec![profile],
                modules: vec![
                    ModuleSummary {
                        id: THEME_MODULE_ID.into(),
                        name: "Codex 主题".into(),
                        version: "0.1.0".into(),
                        enabled_for: vec![],
                        has_service: false,
                        browser_accessible: false,
                    },
                    ModuleSummary {
                        id: ORANGE_GLOW_MODULE_ID.into(),
                        name: "Codex 橙色光框".into(),
                        version: "0.1.0".into(),
                        enabled_for: vec![],
                        has_service: false,
                        browser_accessible: false,
                    },
                    ModuleSummary {
                        id: TASKBOARD_MODULE_ID.into(),
                        name: "任务看板".into(),
                        version: "0.1.0".into(),
                        enabled_for: vec![],
                        has_service: true,
                        browser_accessible: true,
                    },
                ],
                statuses: BTreeMap::from([(
                    product_id.clone(),
                    ProductStatus {
                        product_id,
                        phase: "not running".into(),
                        launch_mode: None,
                        cdp_status: "not used".into(),
                        module_errors: BTreeMap::new(),
                    },
                )]),
                settings,
            }),
            sessions: Mutex::new(BTreeMap::new()),
            services: Mutex::new(BTreeMap::new()),
            settings_path,
        })
    }

    pub fn persist_settings(&self, settings: &Settings) -> Result<(), String> {
        let parent = self.settings_path.parent().ok_or("设置文件路径无效")?;
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        let temporary = temporary_path(&self.settings_path);
        let json = serde_json::to_vec_pretty(settings).map_err(|error| error.to_string())?;
        fs::write(&temporary, json).map_err(|error| error.to_string())?;
        fs::rename(temporary, &self.settings_path).map_err(|error| error.to_string())
    }

    pub async fn product_views(&self) -> Vec<ProductView> {
        let mut views = self.data.lock().await.product_views();
        let services: Vec<_> = self
            .services
            .lock()
            .await
            .iter()
            .map(|(module_id, service)| ModuleServiceView {
                module_id: module_id.clone(),
                host: "127.0.0.1".into(),
                port: service.port(),
            })
            .collect();
        for view in &mut views {
            view.services.clone_from(&services);
        }
        views
    }

    pub async fn set_module_enabled(
        &self,
        product_id: &str,
        module_id: &str,
        enabled: bool,
    ) -> Result<Vec<String>, String> {
        let mut data = self.data.lock().await;
        data.set_module_enabled(product_id, module_id, enabled)?;
        self.persist_settings(&data.settings)?;
        Ok(data
            .settings
            .enabled_modules
            .get(product_id)
            .cloned()
            .unwrap_or_default())
    }

    pub async fn launch_data(
        &self,
        product_id: &str,
    ) -> Result<(ProductProfile, Vec<String>), String> {
        let data = self.data.lock().await;
        let profile = data
            .profiles
            .iter()
            .find(|profile| profile.id == product_id)
            .cloned()
            .ok_or_else(|| format!("未知产品：{product_id}"))?;
        let enabled_modules = data
            .settings
            .enabled_modules
            .get(product_id)
            .cloned()
            .unwrap_or_default();
        Ok((profile, enabled_modules))
    }

    pub async fn set_product_phase(&self, product_id: &str, phase: &str) -> Result<(), String> {
        self.data.lock().await.set_product_phase(product_id, phase)
    }

    pub async fn set_product_launch_mode(
        &self,
        product_id: &str,
        launch_mode: crate::model::LaunchMode,
    ) -> Result<(), String> {
        let mut data = self.data.lock().await;
        let status = data
            .statuses
            .get_mut(product_id)
            .ok_or_else(|| format!("未知产品：{product_id}"))?;
        status.launch_mode = Some(launch_mode);
        Ok(())
    }

    pub async fn product_launch_mode(&self, product_id: &str) -> Option<crate::model::LaunchMode> {
        self.data
            .lock()
            .await
            .statuses
            .get(product_id)
            .and_then(|status| status.launch_mode)
    }

    pub async fn set_product_cdp_status(
        &self,
        product_id: &str,
        cdp_status: &str,
    ) -> Result<(), String> {
        let mut data = self.data.lock().await;
        let status = data
            .statuses
            .get_mut(product_id)
            .ok_or_else(|| format!("未知产品：{product_id}"))?;
        status.cdp_status = cdp_status.into();
        Ok(())
    }

    pub async fn reconcile_product_running(
        &self,
        product_id: &str,
        running: bool,
    ) -> Result<(), String> {
        let stopped = self
            .data
            .lock()
            .await
            .reconcile_product_running(product_id, running)?;
        if stopped {
            self.sessions.lock().await.remove(product_id);
            self.stop_all_services().await;
        }
        Ok(())
    }

    pub async fn set_module_error(
        &self,
        product_id: &str,
        module_id: &str,
        error: Option<String>,
    ) -> Result<(), String> {
        let mut data = self.data.lock().await;
        let status = data
            .statuses
            .get_mut(product_id)
            .ok_or_else(|| format!("未知产品：{product_id}"))?;
        if let Some(error) = error {
            status.module_errors.insert(module_id.into(), error);
        } else {
            status.module_errors.remove(module_id);
        }
        Ok(())
    }

    pub async fn ensure_module_service(
        &self,
        app: &tauri::AppHandle,
        module_id: &str,
    ) -> Result<Option<String>, String> {
        if module_id != TASKBOARD_MODULE_ID {
            return Ok(None);
        }
        if let Some(service) = self.services.lock().await.get(module_id) {
            return Ok(Some(service.url().into()));
        }

        let service = ModuleService::start(app, module_id).await?;
        let url = service.url().to_string();
        self.services.lock().await.insert(module_id.into(), service);
        Ok(Some(url))
    }

    pub async fn stop_module_service(&self, module_id: &str) {
        if let Some(service) = self.services.lock().await.remove(module_id) {
            service.stop().await;
        }
    }

    pub async fn browser_service_url(&self, module_id: &str) -> Result<String, String> {
        let browser_accessible = self
            .data
            .lock()
            .await
            .modules
            .iter()
            .find(|module| module.id == module_id)
            .map(|module| module.browser_accessible)
            .ok_or_else(|| format!("未知模块：{module_id}"))?;
        if !browser_accessible {
            return Err("该模块服务不支持浏览器访问".into());
        }
        self.services
            .lock()
            .await
            .get(module_id)
            .map(|service| service.url().to_string())
            .ok_or_else(|| "模块服务尚未运行".into())
    }

    async fn stop_all_services(&self) {
        let services = std::mem::take(&mut *self.services.lock().await);
        for service in services.into_values() {
            service.stop().await;
        }
    }

    pub async fn replace_session(
        &self,
        product_id: String,
        session: ProductSession,
    ) -> Arc<Mutex<ProductSession>> {
        let session = Arc::new(Mutex::new(session));
        self.sessions
            .lock()
            .await
            .insert(product_id, session.clone());
        session
    }

    pub async fn session(&self, product_id: &str) -> Option<Arc<Mutex<ProductSession>>> {
        self.sessions.lock().await.get(product_id).cloned()
    }
}

fn temporary_path(path: &Path) -> PathBuf {
    path.with_extension("json.tmp")
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::AppState;

    #[test]
    fn module_selection_persists() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let config_dir = std::env::temp_dir().join(format!("cdp-injector-{unique}"));
        let state = AppState::load(config_dir.clone()).unwrap();

        {
            let mut data = state.data.blocking_lock();
            data.set_module_enabled("codex", "dev.cdp-injector.codex-theme", true)
                .unwrap();
            data.set_module_enabled("codex", "dev.cdp-injector.codex-orange-glow", true)
                .unwrap();
            data.set_module_enabled("codex", "dev.dashi.taskboard", true)
                .unwrap();
            state.persist_settings(&data.settings).unwrap();
        }

        let reloaded = AppState::load(config_dir.clone()).unwrap();
        let products = reloaded.data.blocking_lock().product_views();
        assert!(products[0]
            .modules
            .iter()
            .all(|module| module.enabled_for == ["codex"]));

        std::fs::remove_dir_all(config_dir).unwrap();
    }

    #[test]
    fn product_phase_updates() {
        let state = AppState::load(std::env::temp_dir()).unwrap();
        let mut data = state.data.blocking_lock();

        data.set_product_phase("codex", "starting").unwrap();

        assert_eq!(data.product_views()[0].status.phase, "starting");
    }

    #[test]
    fn product_running_state_reconciles() {
        let state = AppState::load(std::env::temp_dir()).unwrap();
        let mut data = state.data.blocking_lock();

        data.set_product_phase("codex", "injected").unwrap();
        data.statuses.get_mut("codex").unwrap().launch_mode =
            Some(crate::model::LaunchMode::Injected);
        assert!(data.reconcile_product_running("codex", false).unwrap());
        assert_eq!(data.product_views()[0].status.phase, "not running");
        assert_eq!(data.product_views()[0].status.launch_mode, None);
        assert_eq!(data.product_views()[0].status.cdp_status, "not used");

        assert!(!data.reconcile_product_running("codex", true).unwrap());
        assert_eq!(data.product_views()[0].status.phase, "running normally");
    }
}
