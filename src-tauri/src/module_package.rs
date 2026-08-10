use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use semver::Version;
use zip::ZipArchive;

const MAX_ARCHIVE_BYTES: u64 = 100 * 1024 * 1024;
const MAX_ARCHIVE_FILES: usize = 10_000;
const SUPPORTED_CAPABILITIES: [&str; 5] = [
    "renderer-injection",
    "local-service",
    "module-data",
    "csp-bypass",
    "external-network",
];

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleManifest {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub icon: String,
    pub hub_api: u32,
    pub targets: Vec<ModuleTarget>,
    pub inject: InjectSpec,
    pub service: Option<ServiceSpec>,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleTarget {
    pub product: String,
    pub context: String,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InjectSpec {
    pub entry: Option<String>,
    #[serde(default)]
    pub styles: Vec<String>,
    pub run_at: String,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceSpec {
    pub entry: String,
    pub health_path: String,
    pub ready_timeout_ms: u64,
}

#[derive(Clone, Debug)]
pub struct InstalledModule {
    pub manifest: ModuleManifest,
    pub path: PathBuf,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModulePackagePreview {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub capabilities: Vec<String>,
    pub targets: Vec<String>,
    pub has_service: bool,
}

impl From<&ModuleManifest> for ModulePackagePreview {
    fn from(manifest: &ModuleManifest) -> Self {
        Self {
            id: manifest.id.clone(),
            name: manifest.name.clone(),
            version: manifest.version.clone(),
            description: manifest.description.clone(),
            capabilities: manifest.capabilities.clone(),
            targets: manifest
                .targets
                .iter()
                .map(|target| format!("{}/{}", target.product, target.context))
                .collect(),
            has_service: manifest.service.is_some(),
        }
    }
}

pub fn inspect_package(path: &Path) -> Result<ModulePackagePreview, String> {
    let (manifest, _) = inspect_archive(path)?;
    Ok(ModulePackagePreview::from(&manifest))
}

pub fn install_package(path: &Path, modules_dir: &Path) -> Result<InstalledModule, String> {
    let (manifest, _) = inspect_archive(path)?;
    fs::create_dir_all(modules_dir).map_err(|error| error.to_string())?;
    let temporary = tempfile::Builder::new()
        .prefix(".install-")
        .tempdir_in(modules_dir)
        .map_err(|error| error.to_string())?;
    extract_archive(path, temporary.path())?;
    validate_extracted(&manifest, temporary.path())?;

    let module_root = modules_dir.join(&manifest.id);
    fs::create_dir_all(&module_root).map_err(|error| error.to_string())?;
    let destination = module_root.join(&manifest.version);
    let backup = module_root.join(format!(
        ".{}.backup-{}",
        manifest.version,
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_nanos()
    ));
    if destination.exists() {
        fs::rename(&destination, &backup).map_err(|error| error.to_string())?;
    }
    let extracted = temporary.keep();
    if let Err(error) = fs::rename(&extracted, &destination) {
        if backup.exists() {
            let _ = fs::rename(&backup, &destination);
        }
        return Err(error.to_string());
    }
    if backup.exists() {
        let _ = fs::remove_dir_all(backup);
    }

    Ok(InstalledModule {
        manifest,
        path: destination,
    })
}

pub fn scan_installed(modules_dir: &Path) -> Result<BTreeMap<String, InstalledModule>, String> {
    let mut installed = BTreeMap::<String, InstalledModule>::new();
    let Ok(module_dirs) = fs::read_dir(modules_dir) else {
        return Ok(installed);
    };
    for module_dir in module_dirs.flatten().filter(|entry| entry.path().is_dir()) {
        let Ok(version_dirs) = fs::read_dir(module_dir.path()) else {
            continue;
        };
        for version_dir in version_dirs.flatten().filter(|entry| entry.path().is_dir()) {
            let path = version_dir.path();
            let Ok(manifest) = load_manifest(&path.join("manifest.json")) else {
                continue;
            };
            if validate_extracted(&manifest, &path).is_err() {
                continue;
            }
            let replace = installed
                .get(&manifest.id)
                .and_then(|current| {
                    Some(
                        Version::parse(&manifest.version).ok()?
                            > Version::parse(&current.manifest.version).ok()?,
                    )
                })
                .unwrap_or(true);
            if replace {
                installed.insert(manifest.id.clone(), InstalledModule { manifest, path });
            }
        }
    }
    Ok(installed)
}

fn inspect_archive(path: &Path) -> Result<(ModuleManifest, BTreeSet<String>), String> {
    let file = File::open(path).map_err(|error| error.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|error| format!("无效的 .cdpmod：{error}"))?;
    if archive.len() > MAX_ARCHIVE_FILES {
        return Err("模块包文件数量过多".into());
    }
    let mut names = BTreeSet::new();
    let mut total_size = 0_u64;
    for index in 0..archive.len() {
        let file = archive.by_index(index).map_err(|error| error.to_string())?;
        let name = safe_archive_name(file.name())?;
        if is_symlink(file.unix_mode()) {
            return Err(format!("模块包不允许符号链接：{name}"));
        }
        total_size = total_size.saturating_add(file.size());
        if total_size > MAX_ARCHIVE_BYTES {
            return Err("模块包解压后超过 100 MB".into());
        }
        if !file.is_dir() && !names.insert(name.clone()) {
            return Err(format!("模块包包含重复路径：{name}"));
        }
    }
    let mut manifest_file = archive
        .by_name("manifest.json")
        .map_err(|_| "模块包根目录缺少 manifest.json".to_string())?;
    let mut json = String::new();
    manifest_file
        .read_to_string(&mut json)
        .map_err(|error| error.to_string())?;
    let manifest: ModuleManifest =
        serde_json::from_str(&json).map_err(|error| format!("manifest.json 无效：{error}"))?;
    validate_manifest(&manifest, &names)?;
    Ok((manifest, names))
}

fn extract_archive(path: &Path, destination: &Path) -> Result<(), String> {
    let file = File::open(path).map_err(|error| error.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|error| error.to_string())?;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|error| error.to_string())?;
        let name = safe_archive_name(entry.name())?;
        if is_symlink(entry.unix_mode()) {
            return Err(format!("模块包不允许符号链接：{name}"));
        }
        let output = destination.join(&name);
        if entry.is_dir() {
            fs::create_dir_all(&output).map_err(|error| error.to_string())?;
            continue;
        }
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let mut output = File::create(output).map_err(|error| error.to_string())?;
        std::io::copy(&mut entry, &mut output).map_err(|error| error.to_string())?;
        output.flush().map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn validate_manifest(manifest: &ModuleManifest, names: &BTreeSet<String>) -> Result<(), String> {
    if manifest.schema_version != 1 || manifest.hub_api != 1 {
        return Err("模块包与当前 CDP Hub API 不兼容".into());
    }
    if manifest.id.is_empty()
        || !manifest
            .id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '-'))
    {
        return Err("模块 ID 只能包含英文字母、数字、点和连字符".into());
    }
    Version::parse(&manifest.version).map_err(|_| "模块版本必须使用 SemVer".to_string())?;
    if manifest.name.trim().is_empty() || manifest.description.trim().is_empty() {
        return Err("模块名称和描述不能为空".into());
    }
    if !manifest
        .targets
        .iter()
        .any(|target| target.product == "codex" && target.context == "main")
    {
        return Err("第一版只支持 Codex main renderer 模块".into());
    }
    if manifest.inject.run_at != "document-start" {
        return Err("第一版只支持 document-start 注入".into());
    }
    if manifest.inject.entry.is_none() && manifest.inject.styles.is_empty() {
        return Err("模块必须包含 JavaScript 入口或样式文件".into());
    }
    for capability in &manifest.capabilities {
        if !SUPPORTED_CAPABILITIES.contains(&capability.as_str()) {
            return Err(format!("不支持的模块能力：{capability}"));
        }
    }
    if manifest.service.is_some()
        != manifest
            .capabilities
            .iter()
            .any(|item| item == "local-service")
    {
        return Err("service 与 local-service 能力声明必须一致".into());
    }

    let mut referenced = vec![manifest.icon.as_str()];
    referenced.extend(manifest.inject.entry.iter().map(String::as_str));
    referenced.extend(manifest.inject.styles.iter().map(String::as_str));
    if let Some(service) = &manifest.service {
        if !service.health_path.starts_with('/') || service.ready_timeout_ms == 0 {
            return Err("模块服务健康检查配置无效".into());
        }
        referenced.push(&service.entry);
    }
    for path in referenced {
        let path = safe_archive_name(path)?;
        if !names.contains(&path) {
            return Err(format!("manifest 引用的文件不存在：{path}"));
        }
    }
    Ok(())
}

fn validate_extracted(manifest: &ModuleManifest, root: &Path) -> Result<(), String> {
    let mut names = BTreeSet::new();
    collect_files(root, root, &mut names)?;
    validate_manifest(manifest, &names)
}

fn collect_files(root: &Path, current: &Path, names: &mut BTreeSet<String>) -> Result<(), String> {
    for entry in fs::read_dir(current).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let metadata = fs::symlink_metadata(entry.path()).map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink() {
            return Err("已安装模块不允许符号链接".into());
        }
        if metadata.is_dir() {
            collect_files(root, &entry.path(), names)?;
        } else if metadata.is_file() {
            let relative = entry
                .path()
                .strip_prefix(root)
                .map_err(|error| error.to_string())?
                .to_string_lossy()
                .replace('\\', "/");
            names.insert(relative);
        }
    }
    Ok(())
}

fn load_manifest(path: &Path) -> Result<ModuleManifest, String> {
    let json = fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_json::from_str(&json).map_err(|error| error.to_string())
}

fn safe_archive_name(name: &str) -> Result<String, String> {
    if name.is_empty() || name.contains('\\') || name.contains('\0') {
        return Err(format!("不安全的模块包路径：{name}"));
    }
    let trimmed = name.trim_end_matches('/');
    let path = Path::new(trimmed);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
        || path
            .components()
            .any(|component| component.as_os_str() == "node_modules")
    {
        return Err(format!("不安全的模块包路径：{name}"));
    }
    Ok(trimmed.to_string())
}

fn is_symlink(mode: Option<u32>) -> bool {
    mode.is_some_and(|mode| mode & 0o170000 == 0o120000)
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Write};

    use zip::{write::SimpleFileOptions, ZipWriter};

    use super::{inspect_package, install_package};

    fn write_valid_package(path: &std::path::Path) {
        let file = File::create(path).unwrap();
        let mut archive = ZipWriter::new(file);
        let options = SimpleFileOptions::default();
        let manifest = r#"{
          "schemaVersion": 1,
          "id": "dev.example.module",
          "name": "Example",
          "version": "1.2.3",
          "description": "Example module",
          "icon": "assets/icon.png",
          "hubApi": 1,
          "targets": [{"product":"codex","context":"main"}],
          "inject": {"entry":"inject/index.js","styles":[],"runAt":"document-start"},
          "capabilities": ["renderer-injection"]
        }"#;
        for (name, contents) in [
            ("manifest.json", manifest),
            ("assets/icon.png", "icon"),
            ("inject/index.js", "globalThis.testModule = true;"),
        ] {
            archive.start_file(name, options).unwrap();
            archive.write_all(contents.as_bytes()).unwrap();
        }
        archive.finish().unwrap();
    }

    #[test]
    fn inspects_and_installs_valid_package() {
        let root = tempfile::tempdir().unwrap();
        let package = root.path().join("example.cdpmod");
        write_valid_package(&package);

        let preview = inspect_package(&package).unwrap();
        assert_eq!(preview.id, "dev.example.module");
        let installed = install_package(&package, &root.path().join("Modules")).unwrap();
        assert!(installed.path.join("inject/index.js").is_file());
    }

    #[test]
    fn rejects_parent_directory_entries() {
        let root = tempfile::tempdir().unwrap();
        let package = root.path().join("unsafe.cdpmod");
        let file = File::create(&package).unwrap();
        let mut archive = ZipWriter::new(file);
        archive
            .start_file("../escape.txt", SimpleFileOptions::default())
            .unwrap();
        archive.write_all(b"unsafe").unwrap();
        archive.finish().unwrap();

        assert!(inspect_package(&package).unwrap_err().contains("不安全"));
        assert!(!root.path().join("escape.txt").exists());
    }
}
