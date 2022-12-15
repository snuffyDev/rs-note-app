#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

#[macro_export]
macro_rules! throw {
  ($($arg:tt)*) => {{
    return Err(format!($($arg)*))
  }};
}

mod core;
mod data;

use std::sync::Mutex;

use data::{AppData, Data, Store};
// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    let ctx = tauri::generate_context!();

    let paths = AppData::initialize_from_config(ctx.config());
    let store = Store::new(paths);

    let app = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            greet,
            data::save_file,
            data::get_files
        ])
        .manage(Data(Mutex::new(store)))
        .build(ctx)
        .expect("error while running tauri application");

    app.run(|_app_handle, e| match e {
        _ => {}
    });
}
