mod commands;
mod model;
mod product;
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
            commands::launch_product
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
