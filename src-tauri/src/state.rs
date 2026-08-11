use std::{
    collections::BTreeMap,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use tauri::Manager;
use tokio::sync::Mutex;

use crate::{
    agent_integration::{self, AgentArtifacts},
    injection::{ORANGE_GLOW_MODULE_ID, TASKBOARD_MODULE_ID, THEME_MODULE_ID},
    model::{
        ApplicationType, ModuleServiceView, ModuleSummary, ProductProfile, ProductStatus,
        ProductView,
    },
    module_package::{self, InstalledModule, ModuleManifest, ModulePackagePreview},
    module_service::{self, ModuleService},
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
    installed_modules: Mutex<BTreeMap<String, InstalledModule>>,
    services: Mutex<BTreeMap<String, ModuleService>>,
    agent_artifacts: Mutex<BTreeMap<String, AgentArtifacts>>,
    settings_path: PathBuf,
    modules_dir: PathBuf,
    runtime_bin_dir: PathBuf,
    quitting: AtomicBool,
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
    pub fn load(
        config_dir: PathBuf,
        runtime_bin_dir: PathBuf,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let profile: ProductProfile =
            serde_json::from_str(include_str!("../resources/products/codex.json"))?;
        let settings_path = config_dir.join("settings.json");
        let settings = match fs::read_to_string(&settings_path) {
            Ok(json) => serde_json::from_str(&json)?,
            Err(error) if error.kind() == ErrorKind::NotFound => Settings::default(),
            Err(error) => return Err(error.into()),
        };
        let product_id = profile.id.clone();
        let modules_dir = config_dir.join("Modules");
        let installed_modules = module_package::scan_installed(&modules_dir)?;
        let taskboard_manifest: ModuleManifest = serde_json::from_str(include_str!(
            "../../builtin-modules/taskboard/manifest.json"
        ))?;
        let mut modules = vec![
            ModuleSummary {
                id: THEME_MODULE_ID.into(),
                name: "Codex 主题".into(),
                version: "0.1.0".into(),
                description: "为 Codex 提供主题与配色".into(),
                capabilities: vec!["renderer-injection".into(), "csp-bypass".into()],
                enabled_for: vec![],
                has_service: false,
                browser_accessible: false,
                agent_skills: vec![],
                agent_commands: vec![],
            },
            ModuleSummary {
                id: ORANGE_GLOW_MODULE_ID.into(),
                name: "Codex 橙色光框".into(),
                version: "0.1.0".into(),
                description: "为 Codex 窗口添加橙色发光边框".into(),
                capabilities: vec!["renderer-injection".into(), "csp-bypass".into()],
                enabled_for: vec![],
                has_service: false,
                browser_accessible: false,
                agent_skills: vec![],
                agent_commands: vec![],
            },
            module_summary(&taskboard_manifest),
        ];
        modules.extend(
            installed_modules
                .values()
                .map(|module| module_summary(&module.manifest)),
        );

        Ok(Self {
            data: Mutex::new(AppStateData {
                profiles: vec![profile],
                modules,
                statuses: BTreeMap::from([(
                    product_id.clone(),
                    ProductStatus {
                        product_id,
                        phase: "not running".into(),
                        launch_mode: None,
                        cdp_status: "not used".into(),
                        module_errors: BTreeMap::new(),
                        agent_integrations: BTreeMap::new(),
                    },
                )]),
                settings,
            }),
            sessions: Mutex::new(BTreeMap::new()),
            installed_modules: Mutex::new(installed_modules),
            services: Mutex::new(BTreeMap::new()),
            agent_artifacts: Mutex::new(BTreeMap::new()),
            settings_path,
            modules_dir,
            runtime_bin_dir,
            quitting: AtomicBool::new(false),
        })
    }

    pub fn begin_quit(&self) -> bool {
        !self.quitting.swap(true, Ordering::SeqCst)
    }

    pub fn is_quitting(&self) -> bool {
        self.quitting.load(Ordering::SeqCst)
    }

    pub fn inspect_module_package(path: &Path) -> Result<ModulePackagePreview, String> {
        module_package::inspect_package(path)
    }

    pub fn modules_dir(&self) -> PathBuf {
        self.modules_dir.clone()
    }

    pub async fn register_installed_module(&self, module: InstalledModule) -> Result<(), String> {
        if matches!(
            module.manifest.id.as_str(),
            THEME_MODULE_ID | ORANGE_GLOW_MODULE_ID | TASKBOARD_MODULE_ID
        ) {
            return Err("模块 ID 与内置模块冲突".into());
        }
        let summary = module_summary(&module.manifest);
        self.stop_module_service(&summary.id).await;
        self.stop_agent_integration(&summary.id).await;
        self.installed_modules
            .lock()
            .await
            .insert(summary.id.clone(), module);
        let mut data = self.data.lock().await;
        if let Some(current) = data.modules.iter_mut().find(|item| item.id == summary.id) {
            let enabled_for = current.enabled_for.clone();
            *current = ModuleSummary {
                enabled_for,
                ..summary
            };
        } else {
            data.modules.push(summary);
        }
        Ok(())
    }

    pub async fn module_source(
        &self,
        module_id: &str,
        service_url: Option<&str>,
    ) -> Result<String, String> {
        if let Some(module) = self.installed_modules.lock().await.get(module_id).cloned() {
            return crate::injection::build_installed_module_source(&module, service_url);
        }
        crate::injection::build_module_source(module_id, service_url)
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

    pub async fn has_module_errors(&self, product_id: &str) -> bool {
        self.data
            .lock()
            .await
            .statuses
            .get(product_id)
            .is_some_and(|status| !status.module_errors.is_empty())
    }

    pub fn runtime_bin_dir(&self) -> Result<PathBuf, String> {
        fs::create_dir_all(&self.runtime_bin_dir).map_err(|error| error.to_string())?;
        Ok(self.runtime_bin_dir.clone())
    }

    pub async fn activate_agent_integration(
        &self,
        app: &tauri::AppHandle,
        product_id: &str,
        module_id: &str,
        service_url: Option<&str>,
    ) -> Result<Option<String>, String> {
        self.stop_agent_integration(module_id).await;
        let (module_root, spec) = if module_id == TASKBOARD_MODULE_ID {
            let manifest: ModuleManifest = serde_json::from_str(include_str!(
                "../../builtin-modules/taskboard/manifest.json"
            ))
            .map_err(|error| error.to_string())?;
            (
                module_service::taskboard_module_dir(app)?,
                manifest.agent_integration,
            )
        } else {
            let installed = self.installed_modules.lock().await.get(module_id).cloned();
            let Some(installed) = installed else {
                return Ok(None);
            };
            (installed.path, installed.manifest.agent_integration)
        };
        let Some(spec) = spec else { return Ok(None) };
        let application_type = self
            .data
            .lock()
            .await
            .profiles
            .iter()
            .find(|profile| profile.id == product_id)
            .map(|profile| profile.application_type)
            .ok_or_else(|| format!("未知产品：{product_id}"))?;
        if application_type != ApplicationType::CodexAgent {
            return Err("该应用类型不支持 Agent Skill 和 CLI".into());
        }

        let skills_root = app
            .path()
            .home_dir()
            .map_err(|error| error.to_string())?
            .join(".codex/skills");
        #[cfg(unix)]
        let command_bin = Some(
            app.path()
                .home_dir()
                .map_err(|error| error.to_string())?
                .join(".local/bin"),
        );
        #[cfg(not(unix))]
        let command_bin: Option<PathBuf> = None;
        let runtime_bin = self.runtime_bin_dir()?;
        let node = module_service::bundled_node(app)?;
        let (artifacts, status) = agent_integration::activate(
            &module_root,
            &spec,
            &skills_root,
            &runtime_bin,
            command_bin.as_deref(),
            &node,
            service_url,
        );
        let error = status.error.clone();
        self.agent_artifacts
            .lock()
            .await
            .insert(module_id.into(), artifacts);
        self.data
            .lock()
            .await
            .statuses
            .get_mut(product_id)
            .ok_or_else(|| format!("未知产品：{product_id}"))?
            .agent_integrations
            .insert(module_id.into(), status);
        Ok(error)
    }

    pub async fn stop_agent_integration(&self, module_id: &str) {
        if let Some(artifacts) = self.agent_artifacts.lock().await.remove(module_id) {
            agent_integration::deactivate(artifacts);
        }
        for status in self.data.lock().await.statuses.values_mut() {
            status.agent_integrations.remove(module_id);
        }
    }

    pub async fn ensure_module_service(
        &self,
        app: &tauri::AppHandle,
        module_id: &str,
    ) -> Result<Option<String>, String> {
        if let Some(service) = self.services.lock().await.get(module_id) {
            return Ok(Some(service.url().into()));
        }

        let service = if module_id == TASKBOARD_MODULE_ID {
            ModuleService::start_taskboard(app, module_id).await?
        } else {
            let installed = self.installed_modules.lock().await.get(module_id).cloned();
            let Some(installed) = installed else {
                return Ok(None);
            };
            let Some(spec) = &installed.manifest.service else {
                return Ok(None);
            };
            ModuleService::start_installed(
                app,
                module_id,
                &installed.path,
                &spec.entry,
                &spec.health_path,
                spec.ready_timeout_ms,
            )
            .await?
        };
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

    pub async fn stop_all_services(&self) {
        let services = std::mem::take(&mut *self.services.lock().await);
        for service in services.into_values() {
            service.stop().await;
        }
        let artifacts = std::mem::take(&mut *self.agent_artifacts.lock().await);
        for artifact in artifacts.into_values() {
            agent_integration::deactivate(artifact);
        }
        for status in self.data.lock().await.statuses.values_mut() {
            status.agent_integrations.clear();
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

fn module_summary(manifest: &ModuleManifest) -> ModuleSummary {
    let integration = manifest.agent_integration.as_ref();
    ModuleSummary {
        id: manifest.id.clone(),
        name: manifest.name.clone(),
        version: manifest.version.clone(),
        description: manifest.description.clone(),
        capabilities: manifest.capabilities.clone(),
        enabled_for: vec![],
        has_service: manifest.service.is_some(),
        browser_accessible: manifest.service.is_some(),
        agent_skills: integration
            .map(|value| {
                value
                    .skills
                    .iter()
                    .map(|skill| skill.name.clone())
                    .collect()
            })
            .unwrap_or_default(),
        agent_commands: integration
            .map(|value| {
                value
                    .commands
                    .iter()
                    .map(|command| command.name.clone())
                    .collect()
            })
            .unwrap_or_default(),
    }
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
        let state = AppState::load(config_dir.clone(), config_dir.join("runtime-bin")).unwrap();

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

        let reloaded = AppState::load(config_dir.clone(), config_dir.join("runtime-bin")).unwrap();
        let products = reloaded.data.blocking_lock().product_views();
        assert!(products[0]
            .modules
            .iter()
            .all(|module| module.enabled_for == ["codex"]));

        std::fs::remove_dir_all(config_dir).unwrap();
    }

    #[test]
    fn product_phase_updates() {
        let temp = std::env::temp_dir();
        let state = AppState::load(temp.clone(), temp.join("cdp-injector-runtime")).unwrap();
        let mut data = state.data.blocking_lock();

        data.set_product_phase("codex", "starting").unwrap();

        assert_eq!(data.product_views()[0].status.phase, "starting");
    }

    #[test]
    fn product_running_state_reconciles() {
        let temp = std::env::temp_dir();
        let state = AppState::load(temp.clone(), temp.join("cdp-injector-runtime")).unwrap();
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

    #[test]
    fn runtime_bin_can_live_outside_a_config_path_with_spaces() {
        let config_dir = std::env::temp_dir().join("Application Support/cdp-injector");
        let runtime_bin = std::env::temp_dir().join("cdp-injector-runtime/bin");
        let state = AppState::load(config_dir, runtime_bin.clone()).unwrap();

        assert_eq!(state.runtime_bin_dir().unwrap(), runtime_bin);
    }
}
