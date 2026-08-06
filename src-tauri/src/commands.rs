use crate::{model::ProductView, state::AppState};

#[tauri::command]
pub async fn list_products(state: tauri::State<'_, AppState>) -> Result<Vec<ProductView>, String> {
    Ok(state.product_views().await)
}

#[tauri::command]
pub async fn set_module_enabled(
    product_id: String,
    module_id: String,
    enabled: bool,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state
        .set_module_enabled(&product_id, &module_id, enabled)
        .await
}
