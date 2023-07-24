use crate::file_view::{FileView, SearchResult};
use crate::idx_store::IDX_STORE;
use log::info;

use crate::walk_metrics::WalkMatrixView;
use crate::{indexing, utils, walk_exec};

use crate::user_setting::USER_SETTING;
use tauri::{AppHandle, Manager};
use tauri::{CustomMenuItem, SystemTrayMenu};
use tauri::{SystemTray, SystemTrayEvent};
use tauri::{Window, Wry};
static mut WINDOW: Option<Window<Wry>> = None;

#[derive(Clone, serde::Serialize)]
struct Payload {
  message: String,
}

#[tauri::command]
async fn walk_metrics() -> WalkMatrixView {
  unsafe { walk_exec::get_walk_matrix() }
}

#[tauri::command]
async fn change_theme(theme: u8) {
  USER_SETTING.write().unwrap().set_theme(theme);
}

#[tauri::command]
async fn get_theme() -> u8 {
  USER_SETTING.read().unwrap().theme()
}

#[tauri::command]
async fn get_lang() -> String {
  USER_SETTING.write().unwrap().lang().to_string()
}

#[tauri::command]
async fn change_lang(lang: String) {
  USER_SETTING.write().unwrap().set_lang(lang);
}
#[tauri::command]
async fn add_exclude_path(path: String) -> u8 {
  if path.eq("/") {
    return 1;
  }
  if USER_SETTING
    .read()
    .unwrap()
    .exclude_index_path()
    .iter()
    .any(|x| x.eq(&path))
  {
    return 1;
  }

  match USER_SETTING.write().unwrap().add_exclude_index_path(path) {
    Ok(_) => 0,
    Err(_) => 1,
  }
}

#[tauri::command]
async fn get_exclude_paths() -> Vec<String> {
  USER_SETTING
    .read()
    .unwrap()
    .exclude_index_path()
    .iter()
    .map(|x| x.to_string())
    .collect()
}
#[tauri::command]
async fn remove_exclude_path(path: String) {
  USER_SETTING
    .write()
    .unwrap()
    .remove_exclude_index_path(path);
}
#[tauri::command]
async fn upgrade() {
  let _ = webbrowser::open("https://github.com/naaive/orange/releases");
}
#[tauri::command]
async fn suggest(kw: String) -> Vec<FileView> {
  info!("suggest kw :{}", kw);
  IDX_STORE.suggest(kw, 20)
}

#[tauri::command]
async fn search(mut kw: String, is_dir_opt: Option<bool>, ext_opt: Option<String>) -> SearchResult {
  info!("search kw :{}", kw);
  if kw.eq("") {
    kw = "*".to_string();
  }
  IDX_STORE.search_with_filter(kw, 100, is_dir_opt, ext_opt)
}

#[tauri::command]
async fn open_file_in_terminal(kw: String) {
  utils::open_file_path_in_terminal(kw.as_str());
}

#[tauri::command]
async fn open_file_path(kw: String) {
  utils::open_file_path(kw.as_str());
}
#[tauri::command]
async fn reindex() {
  indexing::reindex();
}

fn handle_tray_event(app: &AppHandle, event: SystemTrayEvent) {
  match event {
    SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
      "quit" => {
        std::process::exit(0);
      }
      "reindex" => {
        app
          .get_window("main")
          .unwrap()
          .emit(
            "reindex",
            Payload {
              message: "".to_string(),
            },
          )
          .unwrap();
        indexing::reindex();
      }
      "upgrade" => {
        let _ = webbrowser::open("https://github.com/naaive/orange/releases");
      }
      "hide" => {
        app.get_window("main").unwrap().hide().unwrap();
      }
      _ => {}
    },
    SystemTrayEvent::DoubleClick { .. } => {
      app.get_window("main").unwrap().show().unwrap();
    }
    _ => {}
  }
}

fn build_tray() -> SystemTray {
  let quit = CustomMenuItem::new("quit".to_string(), "Quit");
  // let reindex = CustomMenuItem::new("reindex".to_string(), "Reindex");
  let upgrade = CustomMenuItem::new("upgrade".to_string(), "Upgrade");
  let hide = CustomMenuItem::new("hide".to_string(), "Hide Window");
  let tray_menu = SystemTrayMenu::new()
    .add_item(upgrade)
    // .add_item(reindex)
    .add_item(hide)
    .add_item(quit);
  SystemTray::new().with_menu(tray_menu)
}

pub fn show() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
      open_file_path,
      open_file_in_terminal,
      search,
      suggest,
      walk_metrics,
      change_theme,
      get_theme,
      upgrade,
      remove_exclude_path,
      add_exclude_path,
      change_lang,
      get_lang,
      reindex,
      get_exclude_paths
    ])
    .system_tray(build_tray())
    .on_system_tray_event(|app, event| handle_tray_event(app, event))
    .on_window_event(|event| match event.event() {
      tauri::WindowEvent::CloseRequested { api, .. } => {
        event.window().hide().unwrap();
        api.prevent_close();
      }
      _ => {}
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
