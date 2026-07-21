use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, PhysicalPosition, PhysicalSize, WindowEvent,
};

const DATA_VERSION: u32 = 1;
const MIN_WINDOW_HEIGHT: u32 = 600;
const EXPANDED_MIN_WINDOW_WIDTH: u32 = 806;
const COLLAPSED_MIN_WINDOW_WIDTH: u32 = 375;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EventStyle {
    color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CalendarEvent {
    id: String,
    title: String,
    date: String,
    all_day: bool,
    start_time: String,
    end_time: String,
    location: String,
    notes: String,
    style: EventStyle,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeletedCalendarEvent {
    #[serde(flatten)]
    event: CalendarEvent,
    deleted_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppSettings {
    theme: String,
    #[serde(default)]
    sidebar_collapsed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppData {
    version: u32,
    events: Vec<CalendarEvent>,
    #[serde(default)]
    deleted_events: Vec<DeletedCalendarEvent>,
    settings: AppSettings,
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            version: DATA_VERSION,
            events: Vec::new(),
            deleted_events: Vec::new(),
            settings: AppSettings {
                theme: "morning-mist".into(),
                sidebar_collapsed: false,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WindowState {
    version: u32,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    maximized: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            version: DATA_VERSION,
            x: 80,
            y: 60,
            width: 960,
            height: 620,
            maximized: false,
        }
    }
}

struct TrackedWindow {
    state: WindowState,
    last_saved: Instant,
}

struct WindowTracker(Mutex<TrackedWindow>);

fn portable_data_dir() -> Result<PathBuf, String> {
    let executable = std::env::current_exe()
        .map_err(|error| format!("実行ファイルの場所を確認できません: {error}"))?;
    let parent = executable
        .parent()
        .ok_or_else(|| "実行ファイルの親フォルダーがありません".to_string())?;
    Ok(parent.join("data"))
}

fn backup_path(path: &Path) -> PathBuf {
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("data");
    path.with_file_name(format!("{stem}.backup.json"))
}

fn corrupt_path(path: &Path) -> PathBuf {
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("data");
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    path.with_file_name(format!("{stem}.corrupt-{timestamp}.json"))
}

fn write_json_with_backup<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("データフォルダーを作成できません: {error}"))?;
    }

    let serialized = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("保存データをJSONへ変換できません: {error}"))?;
    let temp_path = path.with_extension("json.tmp");
    let mut temp_file = File::create(&temp_path)
        .map_err(|error| format!("一時保存ファイルを作成できません: {error}"))?;
    temp_file
        .write_all(&serialized)
        .map_err(|error| format!("一時保存ファイルへ書き込めません: {error}"))?;
    temp_file
        .sync_all()
        .map_err(|error| format!("一時保存ファイルを確定できません: {error}"))?;

    let backup = backup_path(path);
    if path.exists() {
        fs::copy(path, &backup)
            .map_err(|error| format!("更新前バックアップを作成できません: {error}"))?;
        fs::remove_file(path)
            .map_err(|error| format!("更新前ファイルを置き換えられません: {error}"))?;
    }

    if let Err(error) = fs::rename(&temp_path, path) {
        if backup.exists() {
            let _ = fs::copy(&backup, path);
        }
        return Err(format!("保存ファイルを確定できません: {error}"));
    }
    Ok(())
}

fn read_json_with_recovery<T: DeserializeOwned + Serialize>(
    path: &Path,
) -> Result<Option<T>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let parse = |target: &Path| -> Result<T, String> {
        let content =
            fs::read(target).map_err(|error| format!("保存ファイルを読み込めません: {error}"))?;
        serde_json::from_slice(&content)
            .map_err(|error| format!("保存ファイルの形式が正しくありません: {error}"))
    };

    match parse(path) {
        Ok(value) => Ok(Some(value)),
        Err(primary_error) => {
            let backup = backup_path(path);
            let recovered = parse(&backup).map_err(|_| primary_error)?;
            fs::copy(path, corrupt_path(path))
                .map_err(|error| format!("壊れた保存ファイルを退避できません: {error}"))?;
            fs::remove_file(path)
                .map_err(|error| format!("壊れた保存ファイルを置き換えられません: {error}"))?;
            write_json_with_backup(path, &recovered)?;
            Ok(Some(recovered))
        }
    }
}

fn load_app_data_inner() -> Result<AppData, String> {
    let path = portable_data_dir()?.join("calendar-data.json");
    let data: AppData = read_json_with_recovery(&path)?.unwrap_or_default();
    if data.version != DATA_VERSION {
        return Err(format!("未対応の保存形式です（version: {}）", data.version));
    }
    if !path.exists() {
        write_json_with_backup(&path, &data)?;
    }
    Ok(data)
}

#[tauri::command]
fn load_app_data() -> Result<AppData, String> {
    load_app_data_inner()
}

#[tauri::command]
fn save_app_data(data: AppData) -> Result<(), String> {
    if data.version != DATA_VERSION {
        return Err(format!("未対応の保存形式です（version: {}）", data.version));
    }
    if data
        .events
        .iter()
        .any(|event| event.id.trim().is_empty() || event.title.trim().is_empty())
    {
        return Err("予定名またはIDが空の予定は保存できません".into());
    }
    write_json_with_backup(&portable_data_dir()?.join("calendar-data.json"), &data)
}

#[tauri::command]
fn get_data_directory() -> Result<String, String> {
    Ok(portable_data_dir()?.display().to_string())
}

fn sidebar_min_width(collapsed: bool) -> u32 {
    if collapsed {
        COLLAPSED_MIN_WINDOW_WIDTH
    } else {
        EXPANDED_MIN_WINDOW_WIDTH
    }
}

fn apply_sidebar_window_mode<R: tauri::Runtime>(
    window: &tauri::WebviewWindow<R>,
    collapsed: bool,
) -> Result<(), String> {
    let min_width = sidebar_min_width(collapsed);
    window
        .set_min_size(Some(PhysicalSize::new(min_width, MIN_WINDOW_HEIGHT)))
        .map_err(|error| format!("ウィンドウの最小サイズを変更できません: {error}"))?;

    if !collapsed {
        let size = window
            .outer_size()
            .map_err(|error| format!("現在のウィンドウサイズを確認できません: {error}"))?;
        if size.width < EXPANDED_MIN_WINDOW_WIDTH {
            window
                .set_size(PhysicalSize::new(
                    EXPANDED_MIN_WINDOW_WIDTH,
                    size.height.max(MIN_WINDOW_HEIGHT),
                ))
                .map_err(|error| format!("サイドバー表示幅へ広げられません: {error}"))?;
        }
    }
    Ok(())
}

#[tauri::command]
fn set_sidebar_window_mode(window: tauri::WebviewWindow, collapsed: bool) -> Result<(), String> {
    apply_sidebar_window_mode(&window, collapsed)
}

fn load_window_state() -> Option<WindowState> {
    let path = portable_data_dir().ok()?.join("window-state.json");
    read_json_with_recovery(&path)
        .ok()
        .flatten()
        .filter(|state: &WindowState| state.version == DATA_VERSION)
}

fn save_window_state(state: &WindowState) {
    if let Ok(path) = portable_data_dir().map(|directory| directory.join("window-state.json")) {
        let _ = write_json_with_backup(&path, state);
    }
}

fn state_is_visible<R: tauri::Runtime>(
    window: &tauri::WebviewWindow<R>,
    state: &WindowState,
) -> bool {
    window
        .available_monitors()
        .map(|monitors| {
            monitors.iter().any(|monitor| {
                let position = monitor.position();
                let size = monitor.size();
                let right = position.x.saturating_add(size.width as i32);
                let bottom = position.y.saturating_add(size.height as i32);
                state.x < right - 80
                    && state.y < bottom - 60
                    && state.x.saturating_add(state.width as i32) > position.x + 80
                    && state.y.saturating_add(state.height as i32) > position.y + 60
            })
        })
        .unwrap_or(false)
}

fn show_main_window<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(WindowTracker(Mutex::new(TrackedWindow {
            state: WindowState::default(),
            last_saved: Instant::now(),
        })))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            let show_item =
                MenuItem::with_id(app, "show-calendar", "カレンダーを表示", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit-calendar", "終了", true, None::<&str>)?;
            let tray_menu = Menu::with_items(app, &[&show_item, &quit_item])?;
            let mut tray_builder = TrayIconBuilder::with_id("main")
                .menu(&tray_menu)
                .tooltip("Y-TEC Calendar")
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show-calendar" => show_main_window(app),
                    "quit-calendar" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if matches!(
                        event,
                        TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        }
                    ) {
                        show_main_window(tray.app_handle());
                    }
                });
            if let Some(icon) = app.default_window_icon() {
                tray_builder = tray_builder.icon(icon.clone());
            }
            tray_builder.build(app)?;

            if let Some(window) = app.get_webview_window("main") {
                let sidebar_collapsed = load_app_data_inner()
                    .map(|data| data.settings.sidebar_collapsed)
                    .unwrap_or(false);
                let min_width = sidebar_min_width(sidebar_collapsed);
                window.set_min_size(Some(PhysicalSize::new(min_width, MIN_WINDOW_HEIGHT)))?;

                if let Some(state) = load_window_state() {
                    if state_is_visible(&window, &state) {
                        window.set_position(PhysicalPosition::new(state.x, state.y))?;
                        window.set_size(PhysicalSize::new(
                            state.width.max(min_width),
                            state.height.max(MIN_WINDOW_HEIGHT),
                        ))?;
                    } else {
                        window.center()?;
                    }
                    if state.maximized {
                        window.maximize()?;
                    }
                }

                let position = window
                    .outer_position()
                    .unwrap_or(PhysicalPosition::new(80, 60));
                let size = window.outer_size().unwrap_or(PhysicalSize::new(960, 620));
                let maximized = window.is_maximized().unwrap_or(false);
                if let Ok(mut tracked) = app.state::<WindowTracker>().0.lock() {
                    tracked.state = WindowState {
                        version: DATA_VERSION,
                        x: position.x,
                        y: position.y,
                        width: size.width,
                        height: size.height,
                        maximized,
                    };
                }
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            let tracker = window.state::<WindowTracker>();
            let Ok(mut tracked) = tracker.0.lock() else {
                return;
            };
            let maximized = window.is_maximized().unwrap_or(false);
            tracked.state.maximized = maximized;

            match event {
                WindowEvent::Moved(position) if !maximized => {
                    tracked.state.x = position.x;
                    tracked.state.y = position.y;
                }
                WindowEvent::Resized(size) if !maximized => {
                    tracked.state.width = size.width;
                    tracked.state.height = size.height;
                }
                _ => {}
            }

            let final_event = matches!(
                event,
                WindowEvent::CloseRequested { .. } | WindowEvent::Focused(false)
            );
            if final_event || tracked.last_saved.elapsed() >= Duration::from_millis(700) {
                save_window_state(&tracked.state);
                tracked.last_saved = Instant::now();
            }

            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            load_app_data,
            save_app_data,
            get_data_directory,
            set_sidebar_window_mode
        ])
        .run(tauri::generate_context!())
        .expect("Y-TEC Calendarの起動に失敗しました");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_data_uses_current_version() {
        let data = AppData::default();
        assert_eq!(data.version, DATA_VERSION);
        assert!(data.events.is_empty());
        assert_eq!(data.settings.theme, "morning-mist");
    }

    #[test]
    fn backup_name_is_stable() {
        let path = PathBuf::from("data/calendar-data.json");
        assert_eq!(
            backup_path(&path),
            PathBuf::from("data/calendar-data.backup.json")
        );
    }

    #[test]
    fn minimum_window_height_keeps_the_full_calendar_visible() {
        assert_eq!(MIN_WINDOW_HEIGHT, 600);
    }

    #[test]
    fn sidebar_state_controls_the_minimum_window_width() {
        assert_eq!(sidebar_min_width(false), 806);
        assert_eq!(sidebar_min_width(true), 375);
    }
}
