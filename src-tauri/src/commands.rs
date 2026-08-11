use crate::{
    injection,
    model::{LaunchPreparation, ProductView},
    module_package::{self, ModulePackagePreview},
    product,
    session::ProductSession,
    state::AppState,
};
use tauri::Manager;
use tauri_plugin_opener::OpenerExt;

#[tauri::command]
pub async fn list_products(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<Vec<ProductView>, String> {
    for view in state.product_views().await {
        let profile = view.profile;
        let product_id = profile.id.clone();
        if state.session(&product_id).await.is_none() {
            let cdp_profile = profile.clone();
            let cdp_port = tauri::async_runtime::spawn_blocking(move || {
                product::discover_cdp_port(&cdp_profile)
            })
            .await
            .map_err(|error| error.to_string())??;
            if let Some(port) = cdp_port {
                state
                    .set_product_launch_mode(&product_id, crate::model::LaunchMode::Injected)
                    .await?;
                state
                    .set_product_cdp_status(&product_id, "connecting")
                    .await?;
                let enabled_modules = view
                    .modules
                    .into_iter()
                    .filter(|module| module.enabled_for.contains(&product_id))
                    .map(|module| module.id)
                    .collect();
                if recover_existing_session(
                    &state,
                    app.clone(),
                    product_id.clone(),
                    port,
                    profile.contexts.clone(),
                    enabled_modules,
                )
                .await
                .is_err()
                {
                    state
                        .set_product_cdp_status(&product_id, "disconnected")
                        .await?;
                }
            }
        }

        let running =
            tauri::async_runtime::spawn_blocking(move || product::is_product_running(&profile))
                .await
                .map_err(|error| error.to_string())??;
        state
            .reconcile_product_running(&product_id, running)
            .await?;
    }
    Ok(state.product_views().await)
}

async fn recover_existing_session(
    state: &AppState,
    app: tauri::AppHandle,
    product_id: String,
    port: u16,
    contexts: Vec<crate::model::TargetContext>,
    enabled_modules: Vec<String>,
) -> Result<(), String> {
    let mut session = ProductSession::new(port, contexts);
    if session.refresh_targets().await? == 0 {
        return Err("未找到 Codex CDP 目标".into());
    }
    session.probe().await?;
    install_modules(state, &app, &product_id, &mut session, &enabled_modules).await?;

    let session = state.replace_session(product_id.clone(), session).await;
    state
        .set_product_cdp_status(&product_id, "connected")
        .await?;
    state
        .set_product_phase(
            &product_id,
            if enabled_modules.is_empty() {
                "running normally"
            } else if state.has_module_errors(&product_id).await {
                "partially failed"
            } else {
                "injected"
            },
        )
        .await?;
    spawn_session_monitor(app, product_id, session);
    Ok(())
}

async fn install_modules(
    state: &AppState,
    app: &tauri::AppHandle,
    product_id: &str,
    session: &mut ProductSession,
    module_ids: &[String],
) -> Result<(), String> {
    for module_id in module_ids {
        let service_url = match state.ensure_module_service(app, module_id).await {
            Ok(url) => url,
            Err(error) => {
                state
                    .set_module_error(product_id, module_id, Some(error.clone()))
                    .await?;
                return Err(error);
            }
        };
        let integration_error = match state
            .activate_agent_integration(app, product_id, module_id, service_url.as_deref())
            .await
        {
            Ok(error) => error,
            Err(error) => {
                state.stop_module_service(module_id).await;
                state
                    .set_module_error(product_id, module_id, Some(error.clone()))
                    .await?;
                return Err(error);
            }
        };
        let source = match state.module_source(module_id, service_url.as_deref()).await {
            Ok(source) => source,
            Err(error) => {
                state.stop_module_service(module_id).await;
                state.stop_agent_integration(module_id).await;
                state
                    .set_module_error(product_id, module_id, Some(error.clone()))
                    .await?;
                return Err(error);
            }
        };
        if let Err(error) = session
            .install_source(
                module_id.clone(),
                source,
                module_id == injection::TASKBOARD_MODULE_ID,
            )
            .await
        {
            state.stop_module_service(module_id).await;
            state.stop_agent_integration(module_id).await;
            state
                .set_module_error(product_id, module_id, Some(error.clone()))
                .await?;
            return Err(error);
        }
        state
            .set_module_error(product_id, module_id, integration_error)
            .await?;
    }
    Ok(())
}

#[tauri::command]
pub async fn inspect_module_package(path: String) -> Result<ModulePackagePreview, String> {
    tauri::async_runtime::spawn_blocking(move || {
        AppState::inspect_module_package(std::path::Path::new(&path))
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn install_module_package(
    path: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let preview_path = path.clone();
    let preview = tauri::async_runtime::spawn_blocking(move || {
        AppState::inspect_module_package(std::path::Path::new(&preview_path))
    })
    .await
    .map_err(|error| error.to_string())??;
    if matches!(
        preview.id.as_str(),
        injection::THEME_MODULE_ID
            | injection::ORANGE_GLOW_MODULE_ID
            | injection::TASKBOARD_MODULE_ID
    ) {
        return Err("模块 ID 与内置模块冲突".into());
    }
    let modules_dir = state.modules_dir();
    let module = tauri::async_runtime::spawn_blocking(move || {
        module_package::install_package(std::path::Path::new(&path), &modules_dir)
    })
    .await
    .map_err(|error| error.to_string())??;
    state.register_installed_module(module).await
}

fn spawn_session_monitor(
    app: tauri::AppHandle,
    product_id: String,
    session: std::sync::Arc<tokio::sync::Mutex<ProductSession>>,
) {
    let session = std::sync::Arc::downgrade(&session);
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            let Some(session) = session.upgrade() else {
                break;
            };
            if session.lock().await.refresh_and_inject().await.is_err() {
                let state = app.state::<AppState>();
                let _ = state
                    .set_product_cdp_status(&product_id, "disconnected")
                    .await;
                break;
            }
        }
    });
}

#[tauri::command]
pub async fn set_module_enabled(
    product_id: String,
    module_id: String,
    enabled: bool,
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let enabled_modules = state
        .set_module_enabled(&product_id, &module_id, enabled)
        .await?;

    if let Some(session) = state.session(&product_id).await {
        state.set_product_phase(&product_id, "injecting").await?;
        let mut session = session.lock().await;
        let result = if enabled {
            install_modules(
                &state,
                &app,
                &product_id,
                &mut session,
                std::slice::from_ref(&module_id),
            )
            .await
        } else {
            let result = injection::remove_module(&mut session, &module_id).await;
            state.stop_module_service(&module_id).await;
            state.stop_agent_integration(&module_id).await;
            state
                .set_module_error(&product_id, &module_id, None)
                .await?;
            result
        };
        if let Err(error) = result {
            state
                .set_product_phase(&product_id, "partially failed")
                .await?;
            return Err(error);
        }
        state
            .set_product_phase(
                &product_id,
                if enabled_modules.is_empty() {
                    "running normally"
                } else if state.has_module_errors(&product_id).await {
                    "partially failed"
                } else {
                    "injected"
                },
            )
            .await?;
    }
    Ok(())
}

#[tauri::command]
pub async fn prepare_launch(
    product_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<LaunchPreparation, String> {
    let (profile, enabled_modules) = state.launch_data(&product_id).await?;
    tauri::async_runtime::spawn_blocking(move || {
        product::prepare_launch(&profile, !enabled_modules.is_empty())
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn open_module_service(
    module_id: String,
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let url = state.browser_service_url(&module_id).await?;
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn launch_product(
    product_id: String,
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let (profile, enabled_modules) = state.launch_data(&product_id).await?;
    let previous_launch_mode = state.product_launch_mode(&product_id).await;
    let has_enabled_modules = !enabled_modules.is_empty();
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
                .set_product_launch_mode(&product_id, crate::model::LaunchMode::Injected)
                .await?;
            state
                .set_product_cdp_status(&product_id, "connecting")
                .await?;
            state
                .set_product_phase(&product_id, "connecting to CDP")
                .await?;
            let mut session = ProductSession::new(port, contexts);
            if let Err(error) = session.wait_for_target().await {
                state
                    .set_product_cdp_status(&product_id, "disconnected")
                    .await?;
                state
                    .set_product_phase(&product_id, "launch failed")
                    .await?;
                return Err(error);
            }
            if let Err(error) = session.probe().await {
                state
                    .set_product_cdp_status(&product_id, "disconnected")
                    .await?;
                state
                    .set_product_phase(&product_id, "launch failed")
                    .await?;
                return Err(error);
            }
            state
                .set_product_cdp_status(&product_id, "connected")
                .await?;
            if has_enabled_modules {
                state.set_product_phase(&product_id, "injecting").await?;
                if let Err(error) =
                    install_modules(&state, &app, &product_id, &mut session, &enabled_modules).await
                {
                    state
                        .set_product_phase(&product_id, "partially failed")
                        .await?;
                    return Err(error);
                }
            }
            let session = state.replace_session(product_id.clone(), session).await;
            if has_enabled_modules {
                state
                    .set_product_phase(
                        &product_id,
                        if state.has_module_errors(&product_id).await {
                            "partially failed"
                        } else {
                            "injected"
                        },
                    )
                    .await?;
            }
            spawn_session_monitor(app, product_id, session);
            Ok(())
        }
        Ok(None) => {
            if previous_launch_mode != Some(crate::model::LaunchMode::Injected) {
                state
                    .set_product_launch_mode(&product_id, crate::model::LaunchMode::Normal)
                    .await?;
                state
                    .set_product_cdp_status(&product_id, "not used")
                    .await?;
            }
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
