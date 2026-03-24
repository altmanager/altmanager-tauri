#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Command;
use std::sync::{Arc, Mutex};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, RunEvent, WindowEvent,
};

struct ServerProcess(Arc<Mutex<Option<std::process::Child>>>);

fn spawn_server(app: &AppHandle) -> std::process::Child {
    #[cfg(dev)]
    let binary_path = {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let triple = env!("TAURI_ENV_TARGET_TRIPLE");
        manifest_dir
            .join("binaries")
            .join(format!("altmanager-ws-{}", triple))
    };

    #[cfg(not(dev))]
    let binary_path = std::env::current_exe()
        .expect("failed to get current exe path")
        .parent()
        .expect("failed to get exe directory")
        .join("altmanager-ws");

    // suppress unused variable warning in release builds
    let _ = app;

    Command::new(binary_path)
        .spawn()
        .expect("failed to start altmanager-ws server")
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let child = spawn_server(&app.handle());
            app.manage(ServerProcess(Arc::new(Mutex::new(Some(child)))));

            let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .show_menu_on_left_click(true)
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.0.as_str() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        } else {
                            tauri::WebviewWindowBuilder::new(
                                app,
                                "main",
                                tauri::WebviewUrl::App("/".into()),
                            )
                            .title("AltManager")
                            .inner_size(900.0, 800.0)
                            .build()
                            .expect("failed to recreate window");
                        }
                    }
                    "quit" => {
                        let state = app.state::<ServerProcess>();
                        let mut guard = state.0.lock().unwrap();
                        if let Some(mut child) = guard.take() {
                            let _ = child.kill();
                            let _ = child.wait();
                        }

                        std::thread::spawn(|| {
                            std::thread::sleep(std::time::Duration::from_millis(50));
                            std::process::exit(0);
                        });
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::DoubleClick { .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                window.hide().ok();
                api.prevent_close();
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            if let RunEvent::ExitRequested { api, .. } = event {
                api.prevent_exit();
            }
        });
}
