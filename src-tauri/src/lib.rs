// Module declarations
mod commands;
mod file_ops;
pub mod validator;
mod preview;
mod renamer;
pub mod models;

use commands::{get_file_info, generate_preview, validate_config, execute_batch_rename, undo_batch_rename, list_presets, save_preset, delete_preset, get_settings, set_default_preset, save_last_config};

use tauri::menu::{AboutMetadataBuilder, MenuBuilder, SubmenuBuilder};
use tauri_plugin_dialog::DialogExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let about_metadata = AboutMetadataBuilder::new()
                .authors(Some(vec!["Rhodri Hughes".to_string()]))
                .copyright(Some("© Rhodri Hughes".to_string()))
                .version(Some(app.package_info().version.to_string()))
                .website(Some("https://github.com/rhodrihughes/IngestRename".to_string()))
                .website_label(Some("GitHub Repository".to_string()))
                .build();

            let menu = {
                #[cfg(target_os = "macos")]
                {
                    // On macOS: standard app menu with native About item
                    MenuBuilder::new(app)
                        .item(
                            &SubmenuBuilder::new(app, "IngestRename")
                                .about(Some(about_metadata))
                                .separator()
                                .hide()
                                .hide_others()
                                .show_all()
                                .separator()
                                .quit()
                                .build()?
                        )
                        .build()?
                }
                #[cfg(not(target_os = "macos"))]
                {
                    use tauri::menu::MenuItemBuilder;
                    // On Windows/Linux: Help menu with About item
                    let about_item = MenuItemBuilder::new("About IngestRename")
                        .build(app)?;
                    let about_item_id = about_item.id().clone();

                    let menu = MenuBuilder::new(app)
                        .item(
                            &SubmenuBuilder::new(app, "Help")
                                .item(&about_item)
                                .build()?
                        )
                        .build()?;

                    // Handle the About click on Windows by showing a dialog
                    let app_handle = app.handle().clone();
                    app.on_menu_event(move |_win, event| {
                        if event.id() == &about_item_id {
                            let pkg = app_handle.package_info();
                            tauri::WebviewWindowBuilder::new(
                                &app_handle,
                                "about",
                                tauri::WebviewUrl::App("about".into()),
                            )
                            .title("About")
                            .inner_size(320.0, 200.0)
                            .resizable(false)
                            .build()
                            .ok();
                            // Fallback: show a simple message via dialog plugin
                            let _ = app_handle.dialog()
                                .message(format!(
                                    "{}\nVersion {}\n\nAuthor: Rhodri Hughes\n\nhttps://github.com/rhodrihughes/IngestRename",
                                    pkg.name, pkg.version
                                ))
                                .title("About IngestRename")
                                .blocking_show();
                        }
                    });

                    menu
                }
            };

            app.set_menu(menu)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_file_info,
            generate_preview,
            validate_config,
            execute_batch_rename,
            undo_batch_rename,
            list_presets,
            save_preset,
            delete_preset,
            get_settings,
            set_default_preset,
            save_last_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
