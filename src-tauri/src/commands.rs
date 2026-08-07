use crate::{
    injection,
    model::{LaunchPreparation, ProductView},
    product,
    session::ProductSession,
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
        .await?;

    if module_id == "dev.cdp-injector.codex-theme" {
        if let Some(session) = state.session(&product_id).await {
            state
                .set_product_phase(
                    &product_id,
                    if enabled {
                        "injecting"
                    } else {
                        "running normally"
                    },
                )
                .await?;
            let mut session = session.lock().await;
            let result = if enabled {
                injection::install_theme(&mut session).await
            } else {
                injection::remove_theme(&mut session).await
            };
            if let Err(error) = result {
                state
                    .set_product_phase(&product_id, "partially failed")
                    .await?;
                return Err(error);
            }
            if enabled {
                state.set_product_phase(&product_id, "injected").await?;
            }
        }
    }
    Ok(())
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

    let contexts = profile.contexts.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        product::launch_product(&profile, preparation.mode)
    })
    .await
    .map_err(|error| error.to_string())?;

    match result {
        Ok(Some(port)) => {
            state
                .set_product_phase(&product_id, "connecting to CDP")
                .await?;
            let mut session = ProductSession::new(port, contexts);
            if let Err(error) = session.wait_for_target().await {
                state
                    .set_product_phase(&product_id, "launch failed")
                    .await?;
                return Err(error);
            }
            if let Err(error) = session.probe().await {
                state
                    .set_product_phase(&product_id, "launch failed")
                    .await?;
                return Err(error);
            }
            if has_enabled_modules {
                state.set_product_phase(&product_id, "injecting").await?;
                if let Err(error) = injection::install_theme(&mut session).await {
                    state
                        .set_product_phase(&product_id, "partially failed")
                        .await?;
                    return Err(error);
                }
            }
            let session = state.replace_session(product_id.clone(), session).await;
            if has_enabled_modules {
                state.set_product_phase(&product_id, "injected").await?;
            }
            let session = std::sync::Arc::downgrade(&session);
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    let Some(session) = session.upgrade() else {
                        break;
                    };
                    if session.lock().await.refresh_and_inject().await.is_err() {
                        break;
                    }
                }
            });
            Ok(())
        }
        Ok(None) => {
            state
                .set_product_phase(&product_id, "running normally")
                .await
        }
        Err(error) => {
            state
                .set_product_phase(&product_id, "launch failed")
                .await?;
            Err(error)
        }
    }
}
