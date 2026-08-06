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
    pub application_paths: Vec<String>,
    pub process_names: Vec<String>,
    pub contexts: Vec<TargetContext>,
    pub preview: PreviewCapability,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleSummary {
    pub id: String,
    pub name: String,
    pub version: String,
    pub enabled_for: Vec<String>,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductStatus {
    pub product_id: String,
    pub phase: String,
    pub module_errors: std::collections::BTreeMap<String, String>,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductView {
    pub profile: ProductProfile,
    pub modules: Vec<ModuleSummary>,
    pub status: ProductStatus,
}

#[cfg(test)]
mod tests {
    use super::ProductProfile;

    #[test]
    fn product_profile_parses() {
        let profile: ProductProfile =
            serde_json::from_str(include_str!("../resources/products/codex.json")).unwrap();

        assert_eq!(profile.id, "codex");
        assert!(!profile.preview.supported);
    }
}
