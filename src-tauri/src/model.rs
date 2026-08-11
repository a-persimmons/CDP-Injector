#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetContext {
    pub id: String,
    pub target_type: String,
    pub url_prefixes: Vec<String>,
    pub exclude_url_contains: Vec<String>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewCapability {
    pub supported: bool,
    pub restart_message: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductProfile {
    pub id: String,
    pub name: String,
    pub application_type: ApplicationType,
    pub application_paths: Vec<String>,
    pub process_names: Vec<String>,
    pub contexts: Vec<TargetContext>,
    pub preview: PreviewCapability,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ApplicationType {
    Electron,
    CodexAgent,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleSummary {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub capabilities: Vec<String>,
    pub enabled_for: Vec<String>,
    pub has_service: bool,
    pub browser_accessible: bool,
    pub agent_skills: Vec<String>,
    pub agent_commands: Vec<String>,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentIntegrationStatus {
    pub skill_status: String,
    pub command_status: String,
    pub skills: Vec<String>,
    pub commands: Vec<String>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleServiceView {
    pub module_id: String,
    pub host: String,
    pub port: u16,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductStatus {
    pub product_id: String,
    pub phase: String,
    pub launch_mode: Option<LaunchMode>,
    pub cdp_status: String,
    pub module_errors: std::collections::BTreeMap<String, String>,
    pub agent_integrations: std::collections::BTreeMap<String, AgentIntegrationStatus>,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductView {
    pub profile: ProductProfile,
    pub modules: Vec<ModuleSummary>,
    pub services: Vec<ModuleServiceView>,
    pub status: ProductStatus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LaunchMode {
    Normal,
    Injected,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchPreparation {
    pub mode: LaunchMode,
    pub restart_required: bool,
}

#[cfg(test)]
mod tests {
    use super::ProductProfile;

    #[test]
    fn product_profile_parses() {
        let profile: ProductProfile =
            serde_json::from_str(include_str!("../resources/products/codex.json")).unwrap();

        assert_eq!(profile.id, "codex");
        assert_eq!(profile.application_type, super::ApplicationType::CodexAgent);
        assert!(!profile.preview.supported);
    }
}
