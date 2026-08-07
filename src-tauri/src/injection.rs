use serde_json::json;

use crate::session::ProductSession;

const MODULE_ID: &str = "dev.cdp-injector.codex-theme";
const STYLE_ID: &str = "cdp-injector-theme-style";
const RUNTIME: &str = include_str!("../resources/runtime/cdp-injector.js");
const THEME_CSS: &str = include_str!("../../builtin-modules/codex-theme/inject/index.css");

pub fn build_theme_source() -> String {
    let css = serde_json::to_string(THEME_CSS).expect("built-in theme CSS serializes");
    format!(
        r#"{RUNTIME}
(() => {{
  const id = {style_id:?};
  let style = document.getElementById(id);
  if (!style) {{
    style = document.createElement("style");
    style.id = id;
    style.dataset.cdpHubOwner = {module_id:?};
    (document.head || document.documentElement).append(style);
  }}
  style.textContent = {css};
}})();"#,
        style_id = STYLE_ID,
        module_id = MODULE_ID,
    )
}

pub async fn install_theme(session: &mut ProductSession) -> Result<(), String> {
    session.install_source(build_theme_source()).await
}

pub async fn remove_theme(session: &mut ProductSession) -> Result<(), String> {
    session
        .remove_source(
            json!({
                "expression": format!(
                    "document.getElementById({STYLE_ID:?})?.remove(); globalThis.cdpHub?.deactivate({MODULE_ID:?});"
                ),
                "awaitPromise": true
            }),
        )
        .await
}

#[cfg(test)]
mod tests {
    use super::build_theme_source;

    #[test]
    fn builds_theme_source() {
        let source = build_theme_source();

        assert!(source.contains("globalThis.cdpHub?.apiVersion === 1"));
        assert!(source.contains("cdp-injector-theme-style"));
        assert!(source.contains("dev.cdp-injector.codex-theme"));
        assert_eq!(source.matches("--cdp-injector-accent").count(), 2);
    }
}
