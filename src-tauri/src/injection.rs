use std::fs;

use serde_json::json;

use crate::{module_package::InstalledModule, session::ProductSession};

pub const THEME_MODULE_ID: &str = "dev.cdp-injector.codex-theme";
pub const ORANGE_GLOW_MODULE_ID: &str = "dev.cdp-injector.codex-orange-glow";
pub const TASKBOARD_MODULE_ID: &str = "dev.dashi.taskboard";
const THEME_STYLE_ID: &str = "cdp-injector-theme-style";
const ORANGE_GLOW_STYLE_ID: &str = "cdp-injector-orange-glow-style";
const RUNTIME: &str = include_str!("../resources/runtime/cdp-injector.js");
const THEME_CSS: &str = include_str!("../../builtin-modules/codex-theme/inject/index.css");
const ORANGE_GLOW_CSS: &str =
    include_str!("../../builtin-modules/codex-orange-glow/inject/index.css");
const TASKBOARD_SOURCE: &str = include_str!("../../builtin-modules/taskboard/inject/index.js");

fn build_style_source(module_id: &str, style_id: &str, css: &str) -> String {
    let css = serde_json::to_string(css).expect("built-in CSS serializes");
    format!(
        r#"{RUNTIME}
(() => {{
  const id = {style_id:?};
  const apply = () => {{
    const parent = document.head || document.documentElement;
    if (!parent) return false;
    let style = document.getElementById(id);
    if (!style) {{
      style = document.createElement("style");
      style.id = id;
      style.dataset.cdpHubOwner = {module_id:?};
      parent.append(style);
    }}
    style.textContent = {css};
    return Boolean(style.isConnected && style.sheet && style.sheet.cssRules.length);
  }};
  if (apply()) return true;
  document.addEventListener("DOMContentLoaded", apply, {{ once: true }});
  return true;
}})();"#,
        style_id = style_id,
        module_id = module_id,
    )
}

pub fn build_module_source(module_id: &str, service_url: Option<&str>) -> Result<String, String> {
    match module_id {
        THEME_MODULE_ID => Ok(build_style_source(
            THEME_MODULE_ID,
            THEME_STYLE_ID,
            THEME_CSS,
        )),
        ORANGE_GLOW_MODULE_ID => Ok(build_style_source(
            ORANGE_GLOW_MODULE_ID,
            ORANGE_GLOW_STYLE_ID,
            ORANGE_GLOW_CSS,
        )),
        TASKBOARD_MODULE_ID => {
            let service_url = service_url.ok_or("任务看板服务尚未就绪")?;
            let service_url =
                serde_json::to_string(service_url).map_err(|error| error.to_string())?;
            Ok(format!(
                r#"{RUNTIME}
(() => {{
  const serviceUrl = {service_url};
  globalThis.__CODEX_TASKBOARD_URL__ = serviceUrl;
  globalThis.__CODEX_TASKBOARD_MANAGED_ORIGIN__ = new URL(serviceUrl).origin;
  globalThis.__CODEX_TASKBOARD_SOURCE_HASH__ = serviceUrl;
  {TASKBOARD_SOURCE}
  return Boolean(globalThis.__codexTaskboardInjection__);
}})();"#
            ))
        }
        _ => Err(format!("未知内置模块：{module_id}")),
    }
}

pub fn build_installed_module_source(
    module: &InstalledModule,
    service_url: Option<&str>,
) -> Result<String, String> {
    let manifest = &module.manifest;
    let module_id = serde_json::to_string(&manifest.id).map_err(|error| error.to_string())?;
    let version = serde_json::to_string(&manifest.version).map_err(|error| error.to_string())?;
    let service_url = serde_json::to_string(&service_url).map_err(|error| error.to_string())?;
    let mut style_sources = String::new();
    for (index, relative) in manifest.inject.styles.iter().enumerate() {
        let css =
            fs::read_to_string(module.path.join(relative)).map_err(|error| error.to_string())?;
        let css = serde_json::to_string(&css).map_err(|error| error.to_string())?;
        style_sources.push_str(&format!(
            r#"
  {{
    const style = document.createElement("style");
    style.dataset.cdpHubOwner = {module_id};
    style.dataset.cdpHubStyle = {index:?};
    style.textContent = {css};
    (document.head || document.documentElement).append(style);
  }}"#
        ));
    }
    let entry = manifest
        .inject
        .entry
        .as_ref()
        .map(|relative| fs::read_to_string(module.path.join(relative)))
        .transpose()
        .map_err(|error| error.to_string())?
        .unwrap_or_default();

    Ok(format!(
        r#"{RUNTIME}
(async () => {{
  const moduleId = {module_id};
  await globalThis.cdpHub.deactivate(moduleId);
  {style_sources}
  {entry}
  await globalThis.cdpHub.activate(moduleId, {{
    module: {{ id: moduleId, version: {version} }},
    product: {{ id: "codex" }},
    target: {{ url: location.href, title: document.title }},
    serviceUrl: {service_url}
  }});
  return true;
}})();"#
    ))
}

pub async fn remove_module(session: &mut ProductSession, module_id: &str) -> Result<(), String> {
    let expression = match module_id {
        THEME_MODULE_ID => format!(
            "document.getElementById({THEME_STYLE_ID:?})?.remove(); globalThis.cdpHub?.deactivate({module_id:?});"
        ),
        ORANGE_GLOW_MODULE_ID => format!(
            "document.getElementById({ORANGE_GLOW_STYLE_ID:?})?.remove(); globalThis.cdpHub?.deactivate({module_id:?});"
        ),
        TASKBOARD_MODULE_ID => format!(
            "globalThis.__codexTaskboardInjection__?.destroy?.(); delete globalThis.__CODEX_TASKBOARD_URL__; delete globalThis.__CODEX_TASKBOARD_MANAGED_ORIGIN__; delete globalThis.__CODEX_TASKBOARD_SOURCE_HASH__; globalThis.cdpHub?.deactivate({module_id:?});"
        ),
        _ => format!("globalThis.cdpHub?.deactivate({module_id:?});"),
    };
    session
        .remove_source(
            module_id,
            json!({
                "expression": expression,
                "awaitPromise": true
            }),
        )
        .await
}

#[cfg(test)]
mod tests {
    use crate::module_package::{InjectSpec, InstalledModule, ModuleManifest, ModuleTarget};

    use super::{
        build_installed_module_source, build_module_source, ORANGE_GLOW_MODULE_ID,
        TASKBOARD_MODULE_ID, THEME_MODULE_ID,
    };

    #[test]
    fn builds_independent_module_sources() {
        let theme = build_module_source(THEME_MODULE_ID, None).unwrap();
        let glow = build_module_source(ORANGE_GLOW_MODULE_ID, None).unwrap();

        assert!(theme.contains("cdp-injector-theme-style"));
        assert!(theme.contains("--cdp-injector-accent"));
        assert!(glow.contains("cdp-injector-orange-glow-style"));
        assert!(glow.contains("--cdp-injector-orange"));
        assert!(theme.contains("return Boolean"));
        assert!(glow.contains("return Boolean"));
        assert!(theme.contains("DOMContentLoaded"));
    }

    #[test]
    fn taskboard_source_requires_and_uses_its_service_url() {
        assert!(build_module_source(TASKBOARD_MODULE_ID, None).is_err());

        let source =
            build_module_source(TASKBOARD_MODULE_ID, Some("http://127.0.0.1:43123/")).unwrap();
        assert!(source.contains("http://127.0.0.1:43123/"));
        assert!(source.contains("__codexTaskboardInjection__"));
        assert!(source.contains("return Boolean"));
    }

    #[test]
    fn builds_installed_module_lifecycle_source() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("inject")).unwrap();
        std::fs::write(
            root.path().join("inject/index.js"),
            "cdpHub.register({ id: 'dev.example.module', activate() {} });",
        )
        .unwrap();
        std::fs::write(
            root.path().join("inject/index.css"),
            "body { color: orange; }",
        )
        .unwrap();
        let module = InstalledModule {
            path: root.path().into(),
            manifest: ModuleManifest {
                schema_version: 1,
                id: "dev.example.module".into(),
                name: "Example".into(),
                version: "1.0.0".into(),
                description: "Example module".into(),
                icon: "icon.png".into(),
                hub_api: 1,
                targets: vec![ModuleTarget {
                    product: "codex".into(),
                    context: "main".into(),
                }],
                inject: InjectSpec {
                    entry: Some("inject/index.js".into()),
                    styles: vec!["inject/index.css".into()],
                    run_at: "document-start".into(),
                },
                service: None,
                agent_integration: None,
                capabilities: vec!["renderer-injection".into()],
            },
        };

        let source = build_installed_module_source(&module, None).unwrap();
        assert!(source.contains("dev.example.module"));
        assert!(source.contains("body { color: orange; }"));
        assert!(source.contains("cdpHub.activate"));
        assert!(source.contains("serviceUrl: null"));
    }
}
