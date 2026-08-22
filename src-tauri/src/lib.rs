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

mod credentials;
mod google;

const CALENDAR_DATA_VERSION: u32 = 3;
const WINDOW_STATE_VERSION: u32 = 2;
const MAX_WINDOW_PROFILES: usize = 12;
const MIN_WINDOW_HEIGHT: u32 = 600;
const EXPANDED_MIN_WINDOW_WIDTH: u32 = 806;
const COLLAPSED_MIN_WINDOW_WIDTH: u32 = 375;
static APP_DATA_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EventStyle {
    color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
enum EventRecurrence {
    Simple {
        frequency: String,
        interval: u32,
        #[serde(default)]
        week_days: Vec<u8>,
        monthly_mode: String,
        end: RecurrenceEnd,
        #[serde(default)]
        excluded_dates: Vec<String>,
    },
    Google {
        #[serde(default)]
        lines: Vec<String>,
        #[serde(default = "default_time_zone")]
        time_zone: String,
        #[serde(default)]
        excluded_dates: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum RecurrenceEnd {
    Never,
    Until { date: String },
    Count { count: u32 },
}

fn default_time_zone() -> String {
    "Asia/Tokyo".into()
}

fn default_yearly_recurrence() -> EventRecurrence {
    EventRecurrence::Simple {
        frequency: "yearly".into(),
        interval: 1,
        week_days: Vec::new(),
        monthly_mode: "day-of-month".into(),
        end: RecurrenceEnd::Never,
        excluded_dates: Vec::new(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecurrenceException {
    master_id: String,
    original_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoogleEventLink {
    account_id: String,
    calendar_id: String,
    event_id: String,
    #[serde(default)]
    etag: String,
    #[serde(default)]
    google_updated_at: String,
    #[serde(default)]
    local_updated_at: String,
    #[serde(default)]
    recurring_event_id: Option<String>,
    #[serde(default)]
    original_start: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
enum EventOrigin {
    #[default]
    Local,
    Google {
        account_id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SyncConflict {
    account_id: String,
    detected_at: String,
    reason: String,
    message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CalendarEvent {
    id: String,
    title: String,
    date: String,
    #[serde(default)]
    annual: bool,
    #[serde(default)]
    recurrence: Option<EventRecurrence>,
    #[serde(default)]
    recurrence_exception: Option<RecurrenceException>,
    all_day: bool,
    start_time: String,
    end_time: String,
    location: String,
    notes: String,
    style: EventStyle,
    #[serde(default)]
    origin: EventOrigin,
    #[serde(default)]
    sync_targets: Vec<String>,
    #[serde(default)]
    google_links: Vec<GoogleEventLink>,
    #[serde(default)]
    sync_conflict: Option<SyncConflict>,
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
    #[serde(default)]
    window_display_mode: WindowDisplayMode,
    #[serde(default)]
    google: GoogleIntegrationSettings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
enum WindowDisplayMode {
    #[default]
    Taskbar,
    Tray,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoogleOAuthClient {
    client_id: String,
    #[serde(default)]
    client_secret: String,
    #[serde(default)]
    project_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoogleAccount {
    id: String,
    email: String,
    #[serde(default)]
    display_name: String,
    #[serde(default)]
    calendar_id: String,
    #[serde(default)]
    calendar_name: String,
    #[serde(default = "default_true")]
    sync_enabled: bool,
    #[serde(default)]
    sync_token: String,
    #[serde(default)]
    connected_at: String,
    #[serde(default)]
    last_sync_at: String,
    #[serde(default)]
    last_error: String,
    #[serde(default)]
    needs_reauth: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct GoogleIntegrationSettings {
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    client: Option<GoogleOAuthClient>,
    #[serde(default)]
    accounts: Vec<GoogleAccount>,
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
            version: CALENDAR_DATA_VERSION,
            events: Vec::new(),
            deleted_events: Vec::new(),
            settings: AppSettings {
                theme: "morning-mist".into(),
                sidebar_collapsed: false,
                window_display_mode: WindowDisplayMode::Taskbar,
                google: GoogleIntegrationSettings::default(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WindowPlacement {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    maximized: bool,
}

impl Default for WindowPlacement {
    fn default() -> Self {
        Self {
            x: 80,
            y: 60,
            width: 960,
            height: 620,
            maximized: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WindowProfile {
    layout_id: String,
    #[serde(flatten)]
    placement: WindowPlacement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WindowStateFile {
    version: u32,
    profiles: Vec<WindowProfile>,
}

impl Default for WindowStateFile {
    fn default() -> Self {
        Self {
            version: WINDOW_STATE_VERSION,
            profiles: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyWindowState {
    version: u32,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    maximized: bool,
}

impl From<LegacyWindowState> for WindowPlacement {
    fn from(value: LegacyWindowState) -> Self {
        Self {
            x: value.x,
            y: value.y,
            width: value.width,
            height: value.height,
            maximized: value.maximized,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum WindowStateOnDisk {
    Current(WindowStateFile),
    Legacy(LegacyWindowState),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MonitorGeometry {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    work_x: i32,
    work_y: i32,
    work_width: u32,
    work_height: u32,
    scale_milli: u32,
}

struct TrackedWindow {
    layout_id: String,
    state: WindowPlacement,
    last_saved: Instant,
    restoring: bool,
}

struct WindowTracker(Mutex<TrackedWindow>);
struct DisplayModeState(Mutex<WindowDisplayMode>);

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
    let mut data: AppData = read_json_with_recovery(&path)?.unwrap_or_default();
    if data.version == 1 || data.version == 2 {
        let migration_backup =
            path.with_file_name(format!("calendar-data.v{}.backup.json", data.version));
        if path.exists() && !migration_backup.exists() {
            fs::copy(&path, &migration_backup)
                .map_err(|error| format!("移行前バックアップを作成できません: {error}"))?;
        }
        for event in data.events.iter_mut() {
            if event.annual && event.recurrence.is_none() {
                event.recurrence = Some(default_yearly_recurrence());
            }
        }
        for deleted in data.deleted_events.iter_mut() {
            if deleted.event.annual && deleted.event.recurrence.is_none() {
                deleted.event.recurrence = Some(default_yearly_recurrence());
            }
        }
        data.settings.google.accounts.truncate(3);
        data.version = CALENDAR_DATA_VERSION;
        write_json_with_backup(&path, &data)?;
    } else if data.version != CALENDAR_DATA_VERSION {
        return Err(format!("未対応の保存形式です（version: {}）", data.version));
    }
    if !path.exists() {
        write_json_with_backup(&path, &data)?;
    }
    Ok(data)
}

#[tauri::command]
fn load_app_data() -> Result<AppData, String> {
    let _guard = APP_DATA_LOCK
        .lock()
        .map_err(|_| "予定データの読み込みを開始できません".to_string())?;
    load_app_data_inner()
}

#[tauri::command]
fn save_app_data(data: AppData) -> Result<(), String> {
    let _guard = APP_DATA_LOCK
        .lock()
        .map_err(|_| "予定データの保存を開始できません".to_string())?;
    if data.version != CALENDAR_DATA_VERSION {
        return Err(format!("未対応の保存形式です（version: {}）", data.version));
    }
    if data
        .events
        .iter()
        .any(|event| event.id.trim().is_empty() || event.title.trim().is_empty())
    {
        return Err("予定名またはIDが空の予定は保存できません".into());
    }
    if data.settings.google.accounts.len() > 3 {
        return Err("Googleアカウントは3件まで接続できます".into());
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

fn apply_window_display_mode<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    mode: WindowDisplayMode,
) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "メインウィンドウを確認できません".to_string())?;
    window
        .set_skip_taskbar(mode == WindowDisplayMode::Tray)
        .map_err(|error| format!("タスクバー表示を変更できません: {error}"))?;
    if let Some(tray) = app.tray_by_id("main") {
        tray.set_visible(mode != WindowDisplayMode::Taskbar)
            .map_err(|error| format!("タスクトレイ表示を変更できません: {error}"))?;
    }
    Ok(())
}

#[tauri::command]
fn set_window_display_mode(
    app: tauri::AppHandle,
    state: tauri::State<DisplayModeState>,
    mode: WindowDisplayMode,
) -> Result<(), String> {
    apply_window_display_mode(&app, mode)?;
    let mut current = state
        .0
        .lock()
        .map_err(|_| "ウィンドウ表示設定を更新できません".to_string())?;
    *current = mode;
    Ok(())
}

fn monitor_geometries<R: tauri::Runtime>(
    window: &tauri::WebviewWindow<R>,
) -> Result<Vec<MonitorGeometry>, String> {
    window
        .available_monitors()
        .map(|monitors| {
            monitors
                .iter()
                .map(|monitor| {
                    let position = monitor.position();
                    let size = monitor.size();
                    let work_area = monitor.work_area();
                    MonitorGeometry {
                        x: position.x,
                        y: position.y,
                        width: size.width,
                        height: size.height,
                        work_x: work_area.position.x,
                        work_y: work_area.position.y,
                        work_width: work_area.size.width,
                        work_height: work_area.size.height,
                        scale_milli: (monitor.scale_factor() * 1000.0).round() as u32,
                    }
                })
                .collect()
        })
        .map_err(|error| format!("モニター構成を確認できません: {error}"))
}

fn monitor_layout_id(monitors: &[MonitorGeometry]) -> String {
    let mut sorted = monitors.to_vec();
    sorted.sort_by(|left, right| {
        (
            left.x,
            left.y,
            left.width,
            left.height,
            left.work_x,
            left.work_y,
            left.work_width,
            left.work_height,
            left.scale_milli,
        )
            .cmp(&(
                right.x,
                right.y,
                right.width,
                right.height,
                right.work_x,
                right.work_y,
                right.work_width,
                right.work_height,
                right.scale_milli,
            ))
    });

    let mut hash = 0xcbf29ce484222325_u64;
    for monitor in &sorted {
        let value = format!(
            "{}:{}:{}:{}:{}:{}:{}:{}:{};",
            monitor.x,
            monitor.y,
            monitor.width,
            monitor.height,
            monitor.work_x,
            monitor.work_y,
            monitor.work_width,
            monitor.work_height,
            monitor.scale_milli
        );
        for byte in value.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }
    format!("layout-{}-{hash:016x}", sorted.len())
}

fn current_monitor_layout<R: tauri::Runtime>(
    window: &tauri::WebviewWindow<R>,
) -> Result<(String, Vec<MonitorGeometry>), String> {
    let monitors = monitor_geometries(window)?;
    let layout_id = monitor_layout_id(&monitors);
    Ok((layout_id, monitors))
}

fn fit_window_placement(
    monitors: &[MonitorGeometry],
    placement: &WindowPlacement,
    min_width: u32,
) -> Option<WindowPlacement> {
    let window_left = i64::from(placement.x);
    let window_top = i64::from(placement.y);
    let window_right = window_left + i64::from(placement.width);
    let window_bottom = window_top + i64::from(placement.height);

    let monitor = monitors
        .iter()
        .map(|monitor| {
            let monitor_left = i64::from(monitor.work_x);
            let monitor_top = i64::from(monitor.work_y);
            let monitor_right = monitor_left + i64::from(monitor.work_width);
            let monitor_bottom = monitor_top + i64::from(monitor.work_height);
            let width = window_right.min(monitor_right) - window_left.max(monitor_left);
            let height = window_bottom.min(monitor_bottom) - window_top.max(monitor_top);
            let area = width.max(0) * height.max(0);
            (area, monitor)
        })
        .max_by_key(|(area, _)| *area)
        .and_then(|(area, monitor)| (area > 0).then_some(monitor))?;

    let width = placement
        .width
        .max(min_width)
        .min(monitor.work_width.max(min_width));
    let height = placement
        .height
        .max(MIN_WINDOW_HEIGHT)
        .min(monitor.work_height.max(MIN_WINDOW_HEIGHT));
    let max_x = if width <= monitor.work_width {
        monitor
            .work_x
            .saturating_add((monitor.work_width - width) as i32)
    } else {
        monitor.work_x
    };
    let max_y = if height <= monitor.work_height {
        monitor
            .work_y
            .saturating_add((monitor.work_height - height) as i32)
    } else {
        monitor.work_y
    };

    Some(WindowPlacement {
        x: placement.x.clamp(monitor.work_x, max_x),
        y: placement.y.clamp(monitor.work_y, max_y),
        width,
        height,
        maximized: placement.maximized,
    })
}

fn load_window_state_file(layout_id: &str) -> WindowStateFile {
    let Ok(path) = portable_data_dir().map(|directory| directory.join("window-state.json")) else {
        return WindowStateFile::default();
    };
    let loaded: Option<WindowStateOnDisk> = read_json_with_recovery(&path).ok().flatten();

    match loaded {
        Some(WindowStateOnDisk::Current(state)) if state.version == WINDOW_STATE_VERSION => state,
        Some(WindowStateOnDisk::Legacy(legacy)) if legacy.version == 1 => {
            let legacy_backup = path.with_file_name("window-state.v1.backup.json");
            if path.exists() && !legacy_backup.exists() {
                let _ = fs::copy(&path, legacy_backup);
            }
            let state = WindowStateFile {
                version: WINDOW_STATE_VERSION,
                profiles: vec![WindowProfile {
                    layout_id: layout_id.to_string(),
                    placement: legacy.into(),
                }],
            };
            let _ = write_json_with_backup(&path, &state);
            state
        }
        _ => WindowStateFile::default(),
    }
}

fn upsert_window_profile(
    mut state_file: WindowStateFile,
    layout_id: &str,
    placement: &WindowPlacement,
) -> WindowStateFile {
    state_file.version = WINDOW_STATE_VERSION;
    state_file
        .profiles
        .retain(|profile| profile.layout_id != layout_id);
    state_file.profiles.push(WindowProfile {
        layout_id: layout_id.to_string(),
        placement: placement.clone(),
    });
    if state_file.profiles.len() > MAX_WINDOW_PROFILES {
        let excess = state_file.profiles.len() - MAX_WINDOW_PROFILES;
        state_file.profiles.drain(0..excess);
    }
    state_file
}

fn load_window_placement(layout_id: &str) -> Option<WindowPlacement> {
    load_window_state_file(layout_id)
        .profiles
        .into_iter()
        .find(|profile| profile.layout_id == layout_id)
        .map(|profile| profile.placement)
}

fn save_window_state(layout_id: &str, placement: &WindowPlacement) {
    if let Ok(path) = portable_data_dir().map(|directory| directory.join("window-state.json")) {
        let state_file =
            upsert_window_profile(load_window_state_file(layout_id), layout_id, placement);
        let _ = write_json_with_backup(&path, &state_file);
    }
}

fn capture_window_placement<R: tauri::Runtime>(
    window: &tauri::WebviewWindow<R>,
    maximized: bool,
) -> WindowPlacement {
    let position = window
        .outer_position()
        .unwrap_or(PhysicalPosition::new(80, 60));
    let size = window.outer_size().unwrap_or(PhysicalSize::new(960, 620));
    WindowPlacement {
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
        maximized,
    }
}

fn restore_window_for_layout<R: tauri::Runtime>(
    window: &tauri::WebviewWindow<R>,
    layout_id: &str,
    monitors: &[MonitorGeometry],
    min_width: u32,
) -> Result<WindowPlacement, String> {
    if window.is_maximized().unwrap_or(false) {
        window
            .unmaximize()
            .map_err(|error| format!("ウィンドウを通常表示へ戻せません: {error}"))?;
    }

    let placement = load_window_placement(layout_id)
        .as_ref()
        .and_then(|saved| fit_window_placement(monitors, saved, min_width));

    let placement = if let Some(placement) = placement {
        window
            .set_size(PhysicalSize::new(placement.width, placement.height))
            .map_err(|error| format!("保存したウィンドウサイズへ戻せません: {error}"))?;
        window
            .set_position(PhysicalPosition::new(placement.x, placement.y))
            .map_err(|error| format!("保存したウィンドウ位置へ戻せません: {error}"))?;
        placement
    } else {
        let default = WindowPlacement::default();
        window
            .set_size(PhysicalSize::new(
                default.width.max(min_width),
                default.height.max(MIN_WINDOW_HEIGHT),
            ))
            .map_err(|error| format!("既定のウィンドウサイズへ戻せません: {error}"))?;
        window
            .center()
            .map_err(|error| format!("ウィンドウを画面中央へ戻せません: {error}"))?;
        capture_window_placement(window, false)
    };

    if placement.maximized {
        window
            .maximize()
            .map_err(|error| format!("最大化状態へ戻せません: {error}"))?;
    }
    Ok(placement)
}

fn show_main_window<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        if let Ok((layout_id, monitors)) = current_monitor_layout(&window) {
            let should_restore = app
                .state::<WindowTracker>()
                .0
                .lock()
                .map(|tracked| tracked.layout_id != layout_id)
                .unwrap_or(false);
            if should_restore {
                if let Ok(mut tracked) = app.state::<WindowTracker>().0.lock() {
                    tracked.restoring = true;
                }
                let sidebar_collapsed = load_app_data_inner()
                    .map(|data| data.settings.sidebar_collapsed)
                    .unwrap_or(false);
                let restored = restore_window_for_layout(
                    &window,
                    &layout_id,
                    &monitors,
                    sidebar_min_width(sidebar_collapsed),
                );
                if let Ok(mut tracked) = app.state::<WindowTracker>().0.lock() {
                    if let Ok(placement) = restored {
                        tracked.layout_id = layout_id.clone();
                        tracked.state = placement.clone();
                        tracked.last_saved = Instant::now();
                        save_window_state(&layout_id, &placement);
                    }
                    tracked.restoring = false;
                }
            }
        }
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(WindowTracker(Mutex::new(TrackedWindow {
            layout_id: String::new(),
            state: WindowPlacement::default(),
            last_saved: Instant::now(),
            restoring: true,
        })))
        .manage(DisplayModeState(Mutex::new(WindowDisplayMode::Taskbar)))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            let app_data = load_app_data_inner().unwrap_or_default();
            let display_mode = app_data.settings.window_display_mode;
            if let Ok(mut current) = app.state::<DisplayModeState>().0.lock() {
                *current = display_mode;
            }
            let show_item =
                MenuItem::with_id(app, "show-calendar", "カレンダーを表示", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit-calendar", "終了", true, None::<&str>)?;
            let tray_menu = Menu::with_items(app, &[&show_item, &quit_item])?;
            let mut tray_builder = TrayIconBuilder::with_id("main")
                .menu(&tray_menu)
                .tooltip("Koyomado")
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
            apply_window_display_mode(app.handle(), display_mode).map_err(std::io::Error::other)?;

            if let Some(window) = app.get_webview_window("main") {
                let sidebar_collapsed = app_data.settings.sidebar_collapsed;
                let min_width = sidebar_min_width(sidebar_collapsed);
                window.set_min_size(Some(PhysicalSize::new(min_width, MIN_WINDOW_HEIGHT)))?;

                let (layout_id, monitors) =
                    current_monitor_layout(&window).map_err(std::io::Error::other)?;
                let placement =
                    restore_window_for_layout(&window, &layout_id, &monitors, min_width)
                        .map_err(std::io::Error::other)?;
                if let Ok(mut tracked) = app.state::<WindowTracker>().0.lock() {
                    tracked.layout_id = layout_id.clone();
                    tracked.state = placement.clone();
                    tracked.last_saved = Instant::now();
                    tracked.restoring = false;
                }
                save_window_state(&layout_id, &placement);
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let display_mode = window
                    .state::<DisplayModeState>()
                    .0
                    .lock()
                    .map(|mode| *mode)
                    .unwrap_or_default();
                if display_mode != WindowDisplayMode::Taskbar {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }

            if matches!(event, WindowEvent::Resized(_)) && window.is_minimized().unwrap_or(false) {
                let display_mode = window
                    .state::<DisplayModeState>()
                    .0
                    .lock()
                    .map(|mode| *mode)
                    .unwrap_or_default();
                if display_mode == WindowDisplayMode::Tray {
                    let _ = window.hide();
                }
            }

            let tracker = window.state::<WindowTracker>();
            let maximized = window.is_maximized().unwrap_or(false);
            let save_candidate = {
                let Ok(mut tracked) = tracker.0.lock() else {
                    return;
                };
                if tracked.restoring {
                    return;
                }

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
                if !tracked.layout_id.is_empty()
                    && (final_event || tracked.last_saved.elapsed() >= Duration::from_millis(700))
                {
                    tracked.last_saved = Instant::now();
                    Some((tracked.layout_id.clone(), tracked.state.clone()))
                } else {
                    None
                }
            };

            if let Some((layout_id, placement)) = save_candidate {
                let layout_is_current = window
                    .app_handle()
                    .get_webview_window("main")
                    .and_then(|webview_window| current_monitor_layout(&webview_window).ok())
                    .map(|(current_layout_id, _)| current_layout_id == layout_id)
                    .unwrap_or(false);
                if layout_is_current {
                    save_window_state(&layout_id, &placement);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            load_app_data,
            save_app_data,
            get_data_directory,
            set_sidebar_window_mode,
            set_window_display_mode,
            google::google_connect_account,
            google::google_list_calendars,
            google::google_credential_statuses,
            google::google_disconnect_account,
            google::google_sync_all
        ])
        .run(tauri::generate_context!())
        .expect("Koyomadoの起動に失敗しました");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn monitor(x: i32, width: u32) -> MonitorGeometry {
        MonitorGeometry {
            x,
            y: 0,
            width,
            height: 1080,
            work_x: x,
            work_y: 0,
            work_width: width,
            work_height: 1040,
            scale_milli: 1000,
        }
    }

    #[test]
    fn default_data_uses_current_version() {
        let data = AppData::default();
        assert_eq!(data.version, CALENDAR_DATA_VERSION);
        assert!(data.events.is_empty());
        assert_eq!(data.settings.theme, "morning-mist");
    }

    #[test]
    fn version_one_event_without_annual_flag_remains_compatible() {
        let event: CalendarEvent = serde_json::from_value(serde_json::json!({
            "id": "legacy-event",
            "title": "以前の予定",
            "date": "2026-07-22",
            "allDay": true,
            "startTime": "",
            "endTime": "",
            "location": "",
            "notes": "",
            "style": { "color": "#78a88f" },
            "createdAt": "2026-07-21T00:00:00.000Z",
            "updatedAt": "2026-07-21T00:00:00.000Z"
        }))
        .expect("version 1 event should deserialize");

        assert!(!event.annual);
    }

    #[test]
    fn version_three_recurrence_uses_frontend_camel_case_fields() {
        let recurrence: EventRecurrence = serde_json::from_value(serde_json::json!({
            "kind": "simple",
            "frequency": "weekly",
            "interval": 1,
            "weekDays": [1, 5],
            "monthlyMode": "day-of-month",
            "end": { "type": "never" },
            "excludedDates": ["2026-08-24"]
        }))
        .expect("frontend recurrence must deserialize");
        let serialized = serde_json::to_value(&recurrence).expect("recurrence must serialize");
        assert_eq!(serialized["weekDays"], serde_json::json!([1, 5]));
        assert_eq!(serialized["monthlyMode"], "day-of-month");
        assert_eq!(
            serialized["excludedDates"],
            serde_json::json!(["2026-08-24"])
        );
        assert!(serialized.get("week_days").is_none());
    }

    #[test]
    fn google_origin_uses_frontend_account_id_field() {
        let origin: EventOrigin = serde_json::from_value(serde_json::json!({
            "kind": "google",
            "accountId": "google-account"
        }))
        .expect("frontend origin must deserialize");
        let serialized = serde_json::to_value(&origin).expect("origin must serialize");
        assert_eq!(serialized["accountId"], "google-account");
        assert!(serialized.get("account_id").is_none());
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
        assert_eq!(WindowStateFile::default().version, WINDOW_STATE_VERSION);
        assert_eq!(WindowPlacement::default().height, MIN_WINDOW_HEIGHT + 20);
    }

    #[test]
    fn sidebar_state_controls_the_minimum_window_width() {
        assert_eq!(sidebar_min_width(false), 806);
        assert_eq!(sidebar_min_width(true), 375);
    }

    #[test]
    fn monitor_layout_id_is_independent_of_enumeration_order() {
        let left = monitor(-1920, 1920);
        let center = monitor(0, 1920);
        let right = monitor(1920, 2560);

        assert_eq!(
            monitor_layout_id(&[left.clone(), center.clone(), right.clone()]),
            monitor_layout_id(&[right, left, center])
        );
    }

    #[test]
    fn two_and_three_monitor_layouts_have_different_ids() {
        let first = monitor(0, 1920);
        let second = monitor(1920, 1920);
        let third = monitor(-1920, 1920);

        assert_ne!(
            monitor_layout_id(&[first.clone(), second.clone()]),
            monitor_layout_id(&[first, second, third])
        );
    }

    #[test]
    fn profiles_keep_separate_positions_for_each_monitor_layout() {
        let first = WindowPlacement {
            x: 120,
            y: 80,
            ..WindowPlacement::default()
        };
        let second = WindowPlacement {
            x: 2240,
            y: 160,
            ..WindowPlacement::default()
        };
        let updated_first = WindowPlacement {
            x: 360,
            y: 240,
            ..WindowPlacement::default()
        };

        let state = upsert_window_profile(WindowStateFile::default(), "three-monitors", &first);
        let state = upsert_window_profile(state, "two-monitors", &second);
        let state = upsert_window_profile(state, "three-monitors", &updated_first);

        assert_eq!(state.profiles.len(), 2);
        assert_eq!(state.profiles[0].layout_id, "two-monitors");
        assert_eq!(state.profiles[0].placement, second);
        assert_eq!(state.profiles[1].layout_id, "three-monitors");
        assert_eq!(state.profiles[1].placement, updated_first);
    }

    #[test]
    fn profile_history_is_bounded() {
        let mut state = WindowStateFile::default();
        for index in 0..(MAX_WINDOW_PROFILES + 2) {
            state = upsert_window_profile(
                state,
                &format!("layout-{index}"),
                &WindowPlacement::default(),
            );
        }

        assert_eq!(state.profiles.len(), MAX_WINDOW_PROFILES);
        assert_eq!(state.profiles[0].layout_id, "layout-2");
    }

    #[test]
    fn partially_visible_window_is_clamped_inside_work_area() {
        let placement = WindowPlacement {
            x: -100,
            y: -40,
            width: 960,
            height: 620,
            maximized: false,
        };

        let fitted = fit_window_placement(&[monitor(0, 1920)], &placement, 806)
            .expect("partially visible window should be recoverable");

        assert_eq!(fitted.x, 0);
        assert_eq!(fitted.y, 0);
        assert_eq!(fitted.width, 960);
        assert_eq!(fitted.height, 620);
    }

    #[test]
    fn completely_offscreen_window_is_rejected() {
        let placement = WindowPlacement {
            x: 5000,
            y: 5000,
            ..WindowPlacement::default()
        };

        assert!(fit_window_placement(&[monitor(0, 1920)], &placement, 806).is_none());
    }

    #[test]
    fn legacy_window_state_is_readable_for_migration() {
        let parsed: WindowStateOnDisk = serde_json::from_value(serde_json::json!({
            "version": 1,
            "x": 180,
            "y": 90,
            "width": 960,
            "height": 620,
            "maximized": false
        }))
        .expect("version 1 state should deserialize");

        match parsed {
            WindowStateOnDisk::Legacy(state) => {
                assert_eq!(state.version, 1);
                assert_eq!(state.x, 180);
            }
            WindowStateOnDisk::Current(_) => panic!("version 1 state must use legacy format"),
        }
    }
}
