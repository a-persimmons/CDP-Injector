mod cdp;
mod commands;
mod injection;
mod model;
mod module_service;
mod product;
mod session;
mod state;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(AppState::load(app.path().app_config_dir()?)?);
            tauri::tray::TrayIconBuilder::new().build(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_products,
            commands::set_module_enabled,
            commands::prepare_launch,
            commands::open_module_service,
            commands::launch_product
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
