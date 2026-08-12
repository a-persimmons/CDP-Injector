mod agent_integration;
mod cdp;
mod commands;
mod injection;
mod model;
mod module_package;
mod module_service;
mod product;
mod session;
mod state;

use state::AppState;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    Manager,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            app.manage(AppState::load(
                app.path().app_config_dir()?,
                app.path().app_cache_dir()?.join("Runtime/codex/bin"),
            )?);
            let menu = Menu::with_items(
                app,
                &[
                    &MenuItem::with_id(app, "show", "显示 CDP注入器", true, None::<&str>)?,
                    &PredefinedMenuItem::separator(app)?,
                    &MenuItem::with_id(app, "quit", "退出 CDP注入器", true, None::<&str>)?,
                ],
            )?;
            let mut tray = tauri::tray::TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("CDP注入器");
            if let Some(icon) = app.default_window_icon() {
                tray = tray.icon(icon.clone());
            }
            tray.build(app)?;
            Ok(())
        })
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_products,
            commands::set_module_enabled,
            commands::inspect_module_package,
            commands::install_module_package,
            commands::prepare_launch,
            commands::open_module_service,
            commands::launch_product,
            commands::restart_after_update
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app, event| {
        if let tauri::RunEvent::ExitRequested { api, .. } = event {
            let state = app.state::<AppState>();
            if !state.is_quitting() {
                api.prevent_exit();
                if state.begin_quit() {
                    let app = app.clone();
                    tauri::async_runtime::spawn(async move {
                        app.state::<AppState>().stop_all_services().await;
                        app.exit(0);
                    });
                }
            }
        }
    });
}
