use std::{
    collections::BTreeMap,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use tokio::sync::Mutex;

use crate::model::{ModuleSummary, ProductProfile, ProductStatus, ProductView};

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
                modules: vec![ModuleSummary {
                    id: "dev.cdp-injector.codex-theme".into(),
                    name: "Codex 主题".into(),
                    version: "0.1.0".into(),
                    enabled_for: vec![],
                }],
                statuses: BTreeMap::from([(
                    product_id.clone(),
                    ProductStatus {
                        product_id,
                        phase: "not running".into(),
                        module_errors: BTreeMap::new(),
                    },
                )]),
                settings,
            }),
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
        self.data.lock().await.product_views()
    }

    pub async fn set_module_enabled(
        &self,
        product_id: &str,
        module_id: &str,
        enabled: bool,
    ) -> Result<(), String> {
        let mut data = self.data.lock().await;
        data.set_module_enabled(product_id, module_id, enabled)?;
        self.persist_settings(&data.settings)
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
            state.persist_settings(&data.settings).unwrap();
        }

        let reloaded = AppState::load(config_dir.clone()).unwrap();
        let products = reloaded.data.blocking_lock().product_views();
        assert_eq!(
            products[0].modules[0].enabled_for,
            vec!["codex".to_string()]
        );

        std::fs::remove_dir_all(config_dir).unwrap();
    }
}
