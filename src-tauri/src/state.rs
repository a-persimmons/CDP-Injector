use std::{
    collections::BTreeMap,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use tokio::sync::Mutex;

use crate::model::{ModuleSummary, ProductProfile, ProductStatus};

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
}

fn temporary_path(path: &Path) -> PathBuf {
    path.with_extension("json.tmp")
}
