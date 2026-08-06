use crate::{
    model::{LaunchPreparation, ProductView},
    product,
    state::AppState,
};

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

#[tauri::command]
pub async fn prepare_launch(
    product_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<LaunchPreparation, String> {
    let (profile, has_enabled_modules) = state.launch_data(&product_id).await?;
    tauri::async_runtime::spawn_blocking(move || {
        product::prepare_launch(&profile, has_enabled_modules)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn launch_product(
    product_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let (profile, has_enabled_modules) = state.launch_data(&product_id).await?;
    let preparation = tauri::async_runtime::spawn_blocking({
        let profile = profile.clone();
        move || product::prepare_launch(&profile, has_enabled_modules)
    })
    .await
    .map_err(|error| error.to_string())??;

    state
        .set_product_phase(
            &product_id,
            if preparation.restart_required {
                "stopping"
            } else {
                "starting"
            },
        )
        .await?;

    let result = tauri::async_runtime::spawn_blocking(move || {
        product::launch_product(&profile, preparation.mode)
    })
    .await
    .map_err(|error| error.to_string())?;

    match result {
        Ok(Some(_port)) => state
            .set_product_phase(&product_id, "connecting to CDP")
            .await,
        Ok(None) => state
            .set_product_phase(&product_id, "running normally")
            .await,
        Err(error) => {
            state
                .set_product_phase(&product_id, "launch failed")
                .await?;
            Err(error)
        }
    }
}
