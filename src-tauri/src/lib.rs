use auto_launch::AutoLaunch;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::HashSet,
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
#[cfg(windows)]
use windows::{
    core::HSTRING,
    ApplicationModel::{StartupTask, StartupTaskState},
};

mod credentials;
mod google;

const CALENDAR_DATA_VERSION: u32 = 5;
const WINDOW_STATE_VERSION: u32 = 2;
const MAX_WINDOW_PROFILES: usize = 12;
const MIN_WINDOW_HEIGHT: u32 = 600;
const EXPANDED_MIN_WINDOW_WIDTH: u32 = 806;
const COLLAPSED_MIN_WINDOW_WIDTH: u32 = 375;
const AUTOSTART_ENTRY_NAME: &str = "Koyomado";
const PACKAGED_AUTOSTART_TASK_ID: &str = "KoyomadoStartup";
static APP_DATA_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EventReminders {
    #[serde(default = "default_true")]
    use_google_default: bool,
    #[serde(default)]
    popup_minutes: Vec<u32>,
    #[serde(default)]
    email_minutes: Vec<u32>,
}

impl Default for EventReminders {
    fn default() -> Self {
        Self {
            use_google_default: true,
            popup_minutes: Vec::new(),
            email_minutes: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CalendarEvent {
    id: String,
    title: String,
    date: String,
    #[serde(default)]
    end_date: String,
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
    #[serde(default)]
    reminders: EventReminders,
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

fn normalize_event_range(event: &mut CalendarEvent) -> bool {
    if event.end_date.is_empty() || event.end_date < event.date {
        event.end_date = event.date.clone();
        return true;
    }
    false
}

fn normalize_reminder_minutes(minutes: &mut Vec<u32>) -> bool {
    let previous = minutes.clone();
    minutes.retain(|minutes| *minutes <= 40_320);
    minutes.sort_unstable();
    minutes.dedup();
    minutes.truncate(5);
    *minutes != previous
}

fn normalize_event_reminders(event: &mut CalendarEvent) -> bool {
    let mut changed = normalize_reminder_minutes(&mut event.reminders.email_minutes);
    changed |= normalize_reminder_minutes(&mut event.reminders.popup_minutes);
    let available_popup_slots = 5usize.saturating_sub(event.reminders.email_minutes.len());
    if event.reminders.popup_minutes.len() > available_popup_slots {
        event
            .reminders
            .popup_minutes
            .truncate(available_popup_slots);
        changed = true;
    }
    changed
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
    #[serde(default = "default_ui_scale_percent")]
    ui_scale_percent: u8,
    #[serde(default)]
    window_display_mode: WindowDisplayMode,
    #[serde(default)]
    notifications: NotificationSettings,
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

fn default_ui_scale_percent() -> u8 {
    100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CustomNotificationSound {
    display_name: String,
    stored_file_name: String,
    mime_type: String,
    kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NotificationSettings {
    sound_id: String,
    volume: u8,
    #[serde(default = "default_notification_sound_duration_seconds")]
    sound_duration_seconds: u8,
    #[serde(default)]
    custom_sound: Option<CustomNotificationSound>,
}

fn default_notification_sound_duration_seconds() -> u8 {
    12
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            sound_id: "gentle-chimes".into(),
            volume: 35,
            sound_duration_seconds: default_notification_sound_duration_seconds(),
            custom_sound: None,
        }
    }
}

const NOTIFICATION_SOUND_IDS: [&str; 7] = [
    "gentle-chimes",
    "deep-drop",
    "small-bell",
    "gentle-piano",
    "quiet-kalimba",
    "custom",
    "silent",
];

fn normalize_notification_settings(settings: &mut NotificationSettings) -> bool {
    let mut changed = false;
    if settings.volume > 100 {
        settings.volume = 100;
        changed = true;
    }
    if !(3..=60).contains(&settings.sound_duration_seconds) {
        settings.sound_duration_seconds = settings.sound_duration_seconds.clamp(3, 60);
        changed = true;
    }
    if !NOTIFICATION_SOUND_IDS.contains(&settings.sound_id.as_str())
        || (settings.sound_id == "custom" && settings.custom_sound.is_none())
    {
        settings.sound_id = "gentle-chimes".into();
        changed = true;
    }
    changed
}

fn normalize_ui_scale_percent(settings: &mut AppSettings) -> bool {
    let previous = settings.ui_scale_percent;
    let clamped = settings.ui_scale_percent.clamp(80, 130);
    settings.ui_scale_percent = ((u16::from(clamped) + 2) / 5 * 5) as u8;
    settings.ui_scale_percent != previous
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
    #[serde(default)]
    default_sync_targets: Vec<String>,
}

fn normalize_google_default_sync_targets(google: &mut GoogleIntegrationSettings) -> bool {
    let active_account_ids = google
        .accounts
        .iter()
        .filter(|account| account.sync_enabled)
        .map(|account| account.id.clone())
        .collect::<HashSet<_>>();
    let previous = google.default_sync_targets.clone();
    let mut seen = HashSet::new();
    google.default_sync_targets.retain(|account_id| {
        active_account_ids.contains(account_id) && seen.insert(account_id.clone())
    });
    google.default_sync_targets != previous
}

fn google_default_sync_targets_are_valid(google: &GoogleIntegrationSettings) -> bool {
    let active_account_ids = google
        .accounts
        .iter()
        .filter(|account| account.sync_enabled)
        .map(|account| account.id.as_str())
        .collect::<HashSet<_>>();
    let mut seen = HashSet::new();
    google.default_sync_targets.iter().all(|account_id| {
        active_account_ids.contains(account_id.as_str()) && seen.insert(account_id.as_str())
    })
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
                ui_scale_percent: default_ui_scale_percent(),
                window_display_mode: WindowDisplayMode::Taskbar,
                notifications: NotificationSettings::default(),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StartupBackend {
    Packaged,
    Portable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimeProfile {
    data_dir: PathBuf,
    startup_backend: StartupBackend,
}

fn runtime_profile(
    packaged: bool,
    current_exe: &Path,
    package_local_state: Option<&Path>,
) -> Result<RuntimeProfile, String> {
    let (data_dir, startup_backend) = if packaged {
        let package_local_state = package_local_state
            .ok_or_else(|| "Windowsのパッケージ用LocalStateを確認できません".to_string())?;
        let is_local_state = package_local_state
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case("LocalState"));
        if !is_local_state {
            return Err(
                "Microsoft Store版の保存先がパッケージ用LocalStateではありません".to_string(),
            );
        }
        (
            package_local_state.join("Koyomado").join("data"),
            StartupBackend::Packaged,
        )
    } else {
        let parent = current_exe
            .parent()
            .ok_or_else(|| "実行ファイルの親フォルダーがありません".to_string())?;
        (parent.join("data"), StartupBackend::Portable)
    };

    Ok(RuntimeProfile {
        data_dir,
        startup_backend,
    })
}

#[cfg(windows)]
fn current_process_has_package_identity() -> Result<bool, String> {
    use windows_sys::Win32::{
        Foundation::{APPMODEL_ERROR_NO_PACKAGE, ERROR_INSUFFICIENT_BUFFER},
        Storage::Packaging::Appx::GetCurrentPackageFullName,
    };

    let mut package_full_name_length = 0;
    // パッケージ名そのものは読み取らず、ログにも出力しない。
    let status =
        unsafe { GetCurrentPackageFullName(&mut package_full_name_length, std::ptr::null_mut()) };
    match status {
        ERROR_INSUFFICIENT_BUFFER | 0 => Ok(true),
        APPMODEL_ERROR_NO_PACKAGE => Ok(false),
        error => Err(format!(
            "Windowsのパッケージ識別情報を確認できません: {error}"
        )),
    }
}

#[cfg(not(windows))]
fn current_process_has_package_identity() -> Result<bool, String> {
    Ok(false)
}

fn current_startup_backend() -> Result<StartupBackend, String> {
    if current_process_has_package_identity()? {
        Ok(StartupBackend::Packaged)
    } else {
        Ok(StartupBackend::Portable)
    }
}

#[cfg(windows)]
fn current_package_local_state() -> Result<PathBuf, String> {
    use windows::Storage::ApplicationData;

    let application_data = ApplicationData::Current()
        .map_err(|error| format!("Windowsのパッケージ保存領域を取得できません: {error}"))?;
    let local_folder = application_data
        .LocalFolder()
        .map_err(|error| format!("WindowsのLocalStateフォルダーを取得できません: {error}"))?;
    let path = local_folder
        .Path()
        .map_err(|error| format!("WindowsのLocalStateパスを取得できません: {error}"))?;
    Ok(PathBuf::from(path.to_string()))
}

#[cfg(not(windows))]
fn current_package_local_state() -> Result<PathBuf, String> {
    Err("このOSではMicrosoft Store版の保存領域を取得できません".to_string())
}

fn current_runtime_profile() -> Result<RuntimeProfile, String> {
    let executable = std::env::current_exe()
        .map_err(|error| format!("実行ファイルの場所を確認できません: {error}"))?;
    let startup_backend = current_startup_backend()?;
    let package_local_state = if startup_backend == StartupBackend::Packaged {
        Some(current_package_local_state()?)
    } else {
        None
    };
    runtime_profile(
        startup_backend == StartupBackend::Packaged,
        &executable,
        package_local_state.as_deref(),
    )
}

fn portable_data_dir() -> Result<PathBuf, String> {
    current_runtime_profile().map(|profile| profile.data_dir)
}

fn auto_start_script_path() -> Result<PathBuf, String> {
    let local_app_data = std::env::var_os("LOCALAPPDATA")
        .ok_or_else(|| "Windowsのローカル保存場所を確認できません".to_string())?;
    Ok(PathBuf::from(local_app_data)
        .join("Koyomado")
        .join("autostart.vbs"))
}

fn build_auto_start_script(target: &Path) -> Result<String, String> {
    let target = target
        .to_str()
        .ok_or_else(|| "自動起動するファイルの場所を文字列へ変換できません".to_string())?;
    if target.contains('\r') || target.contains('\n') {
        return Err("自動起動するファイルの場所に使用できない文字が含まれています".into());
    }
    let target = target.replace('"', "\"\"");
    Ok(format!(
        "Option Explicit\r\n\
Dim fso, shell, target, retry, alreadyRunning, process\r\n\
target = \"{target}\"\r\n\
Set fso = CreateObject(\"Scripting.FileSystemObject\")\r\n\
Set shell = CreateObject(\"WScript.Shell\")\r\n\
For retry = 1 To 150\r\n\
  alreadyRunning = False\r\n\
  On Error Resume Next\r\n\
  For Each process In GetObject(\"winmgmts:\\\\.\\root\\cimv2\").ExecQuery(\"SELECT ExecutablePath FROM Win32_Process WHERE Name = 'koyomado.exe'\")\r\n\
    If Not IsNull(process.ExecutablePath) Then\r\n\
      If LCase(process.ExecutablePath) = LCase(target) Then alreadyRunning = True\r\n\
    End If\r\n\
  Next\r\n\
  On Error GoTo 0\r\n\
  If alreadyRunning Then WScript.Quit 0\r\n\
  If fso.FileExists(target) Then\r\n\
    shell.Run Chr(34) & target & Chr(34), 1, False\r\n\
    WScript.Quit 0\r\n\
  End If\r\n\
  WScript.Sleep 2000\r\n\
Next\r\n"
    ))
}

fn encode_utf16le_with_bom(content: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(content.len() * 2 + 2);
    bytes.extend_from_slice(&[0xff, 0xfe]);
    for unit in content.encode_utf16() {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    bytes
}

fn auto_start_manager(script_path: &Path) -> AutoLaunch {
    let script_argument = format!("\"{}\"", script_path.display());
    AutoLaunch::new(AUTOSTART_ENTRY_NAME, "wscript.exe", &[script_argument])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PackagedStartupState {
    Disabled,
    DisabledByUser,
    Enabled,
    DisabledByPolicy,
    EnabledByPolicy,
    Unknown,
}

fn packaged_startup_state_is_enabled(state: PackagedStartupState) -> Result<bool, String> {
    match state {
        PackagedStartupState::Disabled => Ok(false),
        PackagedStartupState::DisabledByUser => Err(
            "Windowsの「スタートアップ アプリ」設定またはタスク マネージャーで、Koyomadoの自動起動がユーザーにより無効化されています。Windows側で有効にしてください。".into(),
        ),
        PackagedStartupState::Enabled | PackagedStartupState::EnabledByPolicy => Ok(true),
        PackagedStartupState::DisabledByPolicy => Err(
            "Windowsのポリシーまたはこのデバイスの設定により、Koyomadoの自動起動は無効化されています。管理者に確認してください。".into(),
        ),
        PackagedStartupState::Unknown => {
            Err("Windowsの自動起動の状態を確認できません。".into())
        }
    }
}

#[cfg(windows)]
fn packaged_startup_state_from_windows(state: StartupTaskState) -> PackagedStartupState {
    if state == StartupTaskState::Disabled {
        PackagedStartupState::Disabled
    } else if state == StartupTaskState::DisabledByUser {
        PackagedStartupState::DisabledByUser
    } else if state == StartupTaskState::Enabled {
        PackagedStartupState::Enabled
    } else if state == StartupTaskState::DisabledByPolicy {
        PackagedStartupState::DisabledByPolicy
    } else if state == StartupTaskState::EnabledByPolicy {
        PackagedStartupState::EnabledByPolicy
    } else {
        PackagedStartupState::Unknown
    }
}

#[cfg(windows)]
fn packaged_startup_task() -> Result<StartupTask, String> {
    StartupTask::GetAsync(&HSTRING::from(PACKAGED_AUTOSTART_TASK_ID))
        .map_err(|error| format!("MSIX版の自動起動設定を確認できません: {error}"))?
        .get()
        .map_err(|error| format!("MSIX版の自動起動設定を確認できません: {error}"))
}

#[cfg(windows)]
fn packaged_startup_state(task: &StartupTask) -> Result<PackagedStartupState, String> {
    task.State()
        .map(packaged_startup_state_from_windows)
        .map_err(|error| format!("MSIX版の自動起動状態を確認できません: {error}"))
}

#[cfg(windows)]
fn get_packaged_auto_start_state() -> Result<bool, String> {
    let task = packaged_startup_task()?;
    packaged_startup_state_is_enabled(packaged_startup_state(&task)?)
}

#[cfg(windows)]
fn install_packaged_auto_start() -> Result<(), String> {
    let task = packaged_startup_task()?;
    match packaged_startup_state_is_enabled(packaged_startup_state(&task)?)? {
        true => Ok(()),
        false => {
            let requested_state = task
                .RequestEnableAsync()
                .map_err(|error| format!("MSIX版の自動起動を要求できません: {error}"))?
                .get()
                .map_err(|error| format!("MSIX版の自動起動を要求できません: {error}"))?;
            match packaged_startup_state_is_enabled(packaged_startup_state_from_windows(
                requested_state,
            ))? {
                true => Ok(()),
                false => Err("Windowsの自動起動を有効にできませんでした。".into()),
            }
        }
    }
}

#[cfg(windows)]
fn disable_packaged_auto_start() -> Result<(), String> {
    let task = packaged_startup_task()?;
    let state = packaged_startup_state(&task)?;
    match packaged_startup_state_is_enabled(state)? {
        false => Ok(()),
        true if state == PackagedStartupState::EnabledByPolicy => Err(
            "WindowsのポリシーによりKoyomadoの自動起動は有効です。アプリから変更できません。"
                .into(),
        ),
        true => {
            task.Disable()
                .map_err(|error| format!("MSIX版の自動起動を解除できません: {error}"))?;
            match packaged_startup_state_is_enabled(packaged_startup_state(&task)?)? {
                false => Ok(()),
                true => Err("Windowsの自動起動を無効にできませんでした。".into()),
            }
        }
    }
}

#[cfg(not(windows))]
fn get_packaged_auto_start_state() -> Result<bool, String> {
    Err("MSIX版の自動起動はWindowsでのみ利用できます。".into())
}

#[cfg(not(windows))]
fn install_packaged_auto_start() -> Result<(), String> {
    Err("MSIX版の自動起動はWindowsでのみ利用できます。".into())
}

#[cfg(not(windows))]
fn disable_packaged_auto_start() -> Result<(), String> {
    Err("MSIX版の自動起動はWindowsでのみ利用できます。".into())
}

fn install_resilient_auto_start() -> Result<(), String> {
    let target = std::env::current_exe()
        .map_err(|error| format!("自動起動するファイルの場所を確認できません: {error}"))?;
    let script_path = auto_start_script_path()?;
    let script = build_auto_start_script(&target)?;
    let parent = script_path
        .parent()
        .ok_or_else(|| "自動起動用フォルダーを確認できません".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("自動起動用フォルダーを作成できません: {error}"))?;
    fs::write(&script_path, encode_utf16le_with_bom(&script))
        .map_err(|error| format!("自動起動用ファイルを保存できません: {error}"))?;
    auto_start_manager(&script_path)
        .enable()
        .map_err(|error| format!("Windowsの自動起動へ登録できません: {error}"))
}

#[tauri::command]
fn get_auto_start_state() -> Result<bool, String> {
    if current_startup_backend()? == StartupBackend::Packaged {
        return get_packaged_auto_start_state();
    }
    let script_path = auto_start_script_path()?;
    auto_start_manager(&script_path)
        .is_enabled()
        .map_err(|error| format!("自動起動の状態を確認できません: {error}"))
}

#[tauri::command]
fn repair_auto_start() -> Result<(), String> {
    match current_startup_backend()? {
        StartupBackend::Packaged => install_packaged_auto_start(),
        StartupBackend::Portable => install_resilient_auto_start(),
    }
}

#[tauri::command]
fn set_auto_start_state(enabled: bool) -> Result<(), String> {
    if current_startup_backend()? == StartupBackend::Packaged {
        return if enabled {
            install_packaged_auto_start()
        } else {
            disable_packaged_auto_start()
        };
    }
    let script_path = auto_start_script_path()?;
    let manager = auto_start_manager(&script_path);
    if enabled {
        return install_resilient_auto_start();
    }
    if manager
        .is_enabled()
        .map_err(|error| format!("自動起動の状態を確認できません: {error}"))?
    {
        manager
            .disable()
            .map_err(|error| format!("Windowsの自動起動を解除できません: {error}"))?;
    }
    if script_path.exists() {
        fs::remove_file(&script_path)
            .map_err(|error| format!("自動起動用ファイルを削除できません: {error}"))?;
    }
    Ok(())
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

fn load_app_data_from_path(path: &Path) -> Result<AppData, String> {
    let mut data: AppData = read_json_with_recovery(path)?.unwrap_or_default();
    let mut should_write = !path.exists();
    if (1..CALENDAR_DATA_VERSION).contains(&data.version) {
        let migration_backup =
            path.with_file_name(format!("calendar-data.v{}.backup.json", data.version));
        if path.exists() && !migration_backup.exists() {
            fs::copy(path, &migration_backup)
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
        should_write = true;
    } else if data.version != CALENDAR_DATA_VERSION {
        return Err(format!("未対応の保存形式です（version: {}）", data.version));
    }
    for event in data.events.iter_mut() {
        should_write |= normalize_event_range(event);
        should_write |= normalize_event_reminders(event);
    }
    for deleted in data.deleted_events.iter_mut() {
        should_write |= normalize_event_range(&mut deleted.event);
        should_write |= normalize_event_reminders(&mut deleted.event);
    }
    should_write |= normalize_notification_settings(&mut data.settings.notifications);
    should_write |= normalize_ui_scale_percent(&mut data.settings);
    should_write |= normalize_google_default_sync_targets(&mut data.settings.google);
    if should_write {
        write_json_with_backup(path, &data)?;
    }
    Ok(data)
}

fn load_app_data_inner() -> Result<AppData, String> {
    load_app_data_from_path(&portable_data_dir()?.join("calendar-data.json"))
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
    if data.events.iter().any(|event| {
        event.id.trim().is_empty()
            || event.title.trim().is_empty()
            || event.end_date < event.date
            || (!event.all_day
                && (event.start_time.is_empty()
                    || event.end_time.is_empty()
                    || (event.end_date == event.date && event.end_time <= event.start_time)))
    }) {
        return Err("予定名、ID、または開始・終了日時が正しくない予定は保存できません".into());
    }
    if data.settings.google.accounts.len() > 3 {
        return Err("Googleアカウントは3件まで接続できます".into());
    }
    if !google_default_sync_targets_are_valid(&data.settings.google) {
        return Err("新しい予定のGoogle既定保存先が正しくありません".into());
    }
    if data.events.iter().any(|event| {
        event.reminders.popup_minutes.len() + event.reminders.email_minutes.len() > 5
            || event
                .reminders
                .popup_minutes
                .iter()
                .chain(event.reminders.email_minutes.iter())
                .any(|minutes| *minutes > 40_320)
    }) {
        return Err("通知は各方式5件まで、予定開始の4週間前までで設定してください".into());
    }
    if data.settings.notifications.volume > 100
        || !(3..=60).contains(&data.settings.notifications.sound_duration_seconds)
        || !NOTIFICATION_SOUND_IDS.contains(&data.settings.notifications.sound_id.as_str())
        || (data.settings.notifications.sound_id == "custom"
            && data.settings.notifications.custom_sound.is_none())
    {
        return Err("通知音の設定が正しくありません".into());
    }
    if !(80..=130).contains(&data.settings.ui_scale_percent)
        || data.settings.ui_scale_percent % 5 != 0
    {
        return Err("表示倍率は80～130%の5%刻みで設定してください".into());
    }
    write_json_with_backup(&portable_data_dir()?.join("calendar-data.json"), &data)
}

const MAX_CUSTOM_NOTIFICATION_SOUND_BYTES: usize = 15 * 1024 * 1024;

fn supported_notification_sound(
    file_name: &str,
    bytes: &[u8],
) -> Option<(&'static str, &'static str, &'static str)> {
    let extension = Path::new(file_name)
        .extension()?
        .to_string_lossy()
        .to_ascii_lowercase();
    let starts_with = |signature: &[u8]| bytes.starts_with(signature);
    match extension.as_str() {
        "wav" if starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WAVE") => {
            Some(("wav", "audio/wav", "audio"))
        }
        "mp3"
            if starts_with(b"ID3")
                || bytes
                    .get(0..2)
                    .is_some_and(|head| head[0] == 0xff && head[1] & 0xe0 == 0xe0) =>
        {
            Some(("mp3", "audio/mpeg", "audio"))
        }
        "m4a" | "mp4" if bytes.get(4..8) == Some(b"ftyp") => Some(("m4a", "audio/mp4", "audio")),
        "aac"
            if bytes
                .get(0..2)
                .is_some_and(|head| head[0] == 0xff && head[1] & 0xf6 == 0xf0) =>
        {
            Some(("aac", "audio/aac", "audio"))
        }
        "ogg" | "oga" | "opus" if starts_with(b"OggS") => Some(("ogg", "audio/ogg", "audio")),
        "flac" if starts_with(b"fLaC") => Some(("flac", "audio/flac", "audio")),
        "mid" | "midi" if starts_with(b"MThd") => Some(("mid", "audio/midi", "midi")),
        _ => None,
    }
}

#[tauri::command]
fn save_custom_notification_sound(
    file_name: String,
    bytes: Vec<u8>,
) -> Result<CustomNotificationSound, String> {
    if bytes.is_empty() || bytes.len() > MAX_CUSTOM_NOTIFICATION_SOUND_BYTES {
        return Err("通知音は15 MB以内のファイルを選択してください".into());
    }
    let (extension, mime_type, kind) = supported_notification_sound(&file_name, &bytes)
        .ok_or_else(|| "対応している音声ファイル（MP3 / M4A / AAC / WAV / OGG / Opus / FLAC / MIDI）を選択してください".to_string())?;
    let directory = portable_data_dir()?.join("notification-sounds");
    fs::create_dir_all(&directory)
        .map_err(|error| format!("通知音フォルダーを作成できません: {error}"))?;
    let stored_file_name = format!("custom.{extension}");
    let destination = directory.join(&stored_file_name);
    let temporary = directory.join(format!("{stored_file_name}.tmp"));
    let previous = directory.join(format!("{stored_file_name}.previous"));
    fs::write(&temporary, &bytes)
        .map_err(|error| format!("通知音を一時保存できません: {error}"))?;
    if previous.exists() {
        fs::remove_file(&previous)
            .map_err(|error| format!("以前の通知音バックアップを整理できません: {error}"))?;
    }
    if destination.exists() {
        fs::rename(&destination, &previous)
            .map_err(|error| format!("以前の通知音を置き換えられません: {error}"))?;
    }
    if let Err(error) = fs::rename(&temporary, &destination) {
        if previous.exists() {
            let _ = fs::rename(&previous, &destination);
        }
        let _ = fs::remove_file(&temporary);
        return Err(format!("通知音を保存できません: {error}"));
    }
    if previous.exists() {
        let _ = fs::remove_file(&previous);
    }
    for entry in fs::read_dir(&directory)
        .map_err(|error| format!("通知音フォルダーを確認できません: {error}"))?
        .flatten()
    {
        let path = entry.path();
        if path != destination
            && path
                .file_stem()
                .is_some_and(|stem| stem.to_string_lossy() == "custom")
        {
            let _ = fs::remove_file(path);
        }
    }
    let display_name = Path::new(&file_name)
        .file_name()
        .map(|name| name.to_string_lossy().chars().take(120).collect())
        .filter(|name: &String| !name.trim().is_empty())
        .unwrap_or_else(|| stored_file_name.clone());
    Ok(CustomNotificationSound {
        display_name,
        stored_file_name,
        mime_type: mime_type.into(),
        kind: kind.into(),
    })
}

#[tauri::command]
fn load_custom_notification_sound(stored_file_name: String) -> Result<Vec<u8>, String> {
    let file_name = Path::new(&stored_file_name)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "通知音のファイル名が正しくありません".to_string())?;
    if file_name != stored_file_name || !file_name.starts_with("custom.") {
        return Err("通知音のファイル名が正しくありません".into());
    }
    let path = portable_data_dir()?
        .join("notification-sounds")
        .join(file_name);
    let bytes = fs::read(&path).map_err(|error| format!("通知音を読み込めません: {error}"))?;
    supported_notification_sound(file_name, &bytes)
        .ok_or_else(|| "保存されている通知音の形式を確認できません".to_string())?;
    Ok(bytes)
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

#[tauri::command]
fn show_main_window_for_notification(app: tauri::AppHandle) {
    show_main_window(&app);
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
            get_auto_start_state,
            repair_auto_start,
            set_auto_start_state,
            set_sidebar_window_mode,
            set_window_display_mode,
            save_custom_notification_sound,
            load_custom_notification_sound,
            show_main_window_for_notification,
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
        assert_eq!(data.settings.ui_scale_percent, 100);
        assert!(data.settings.google.default_sync_targets.is_empty());
        assert_eq!(data.settings.notifications.sound_id, "gentle-chimes");
        assert_eq!(data.settings.notifications.sound_duration_seconds, 12);
        assert!(EventReminders::default().use_google_default);
    }

    #[test]
    fn notification_duration_is_clamped_to_supported_range() {
        let mut settings = NotificationSettings {
            sound_duration_seconds: 0,
            ..NotificationSettings::default()
        };
        assert!(normalize_notification_settings(&mut settings));
        assert_eq!(settings.sound_duration_seconds, 3);
        settings.sound_duration_seconds = 100;
        assert!(normalize_notification_settings(&mut settings));
        assert_eq!(settings.sound_duration_seconds, 60);
    }

    #[test]
    fn ui_scale_is_clamped_and_rounded_to_supported_steps() {
        let mut settings = AppData::default().settings;
        settings.ui_scale_percent = 40;
        assert!(normalize_ui_scale_percent(&mut settings));
        assert_eq!(settings.ui_scale_percent, 80);
        settings.ui_scale_percent = 127;
        assert!(normalize_ui_scale_percent(&mut settings));
        assert_eq!(settings.ui_scale_percent, 125);
        settings.ui_scale_percent = 200;
        assert!(normalize_ui_scale_percent(&mut settings));
        assert_eq!(settings.ui_scale_percent, 130);
    }

    #[test]
    fn version_four_file_migrates_with_backup_and_notification_defaults() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be available")
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "koyomado-v4-migration-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&directory).expect("test directory should be created");
        let path = directory.join("calendar-data.json");
        let mut legacy = serde_json::to_value(AppData::default()).expect("data should serialize");
        legacy["version"] = serde_json::json!(4);
        legacy["settings"]
            .as_object_mut()
            .expect("settings should be an object")
            .remove("notifications");
        fs::write(
            &path,
            serde_json::to_vec_pretty(&legacy).expect("legacy data should serialize"),
        )
        .expect("legacy file should be written");

        let migrated = load_app_data_from_path(&path).expect("version 4 should migrate");
        assert_eq!(migrated.version, CALENDAR_DATA_VERSION);
        assert_eq!(migrated.settings.notifications.volume, 35);
        assert_eq!(migrated.settings.notifications.sound_duration_seconds, 12);
        assert!(directory.join("calendar-data.v4.backup.json").exists());

        fs::remove_dir_all(&directory).expect("test directory should be removed");
    }

    #[test]
    fn notification_file_signatures_reject_renamed_or_unsupported_data() {
        assert_eq!(
            supported_notification_sound("tone.wav", b"RIFF\0\0\0\0WAVEdata"),
            Some(("wav", "audio/wav", "audio"))
        );
        assert_eq!(
            supported_notification_sound("song.mid", b"MThd\0\0\0\x06"),
            Some(("mid", "audio/midi", "midi"))
        );
        assert!(supported_notification_sound("renamed.mp3", b"not audio").is_none());
        assert!(supported_notification_sound("script.exe", b"MThd\0\0\0\x06").is_none());
    }

    #[test]
    fn version_three_google_settings_without_defaults_remain_compatible() {
        let google: GoogleIntegrationSettings = serde_json::from_value(serde_json::json!({
            "enabled": true,
            "client": null,
            "accounts": []
        }))
        .expect("version 3 Google settings should deserialize");

        assert!(google.default_sync_targets.is_empty());
    }

    #[test]
    fn version_three_file_migrates_with_events_google_settings_and_backup() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be available")
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "koyomado-v3-migration-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&directory).expect("test directory should be created");
        let path = directory.join("calendar-data.json");
        let migration_backup = directory.join("calendar-data.v3.backup.json");
        let legacy_json = serde_json::json!({
            "version": 3,
            "events": [{
                "id": "legacy-event",
                "title": "移行前の予定",
                "date": "2026-08-23",
                "allDay": true,
                "startTime": "",
                "endTime": "",
                "location": "会議室",
                "notes": "予定を保持する",
                "style": { "color": "#78a88f" },
                "syncTargets": ["active"],
                "createdAt": "2026-08-22T00:00:00.000Z",
                "updatedAt": "2026-08-22T00:00:00.000Z"
            }],
            "deletedEvents": [{
                "id": "deleted-event",
                "title": "削除済みの予定",
                "date": "2026-08-24",
                "allDay": true,
                "startTime": "",
                "endTime": "",
                "location": "",
                "notes": "削除情報を保持する",
                "style": { "color": "#86abc7" },
                "createdAt": "2026-08-22T01:00:00.000Z",
                "updatedAt": "2026-08-22T01:00:00.000Z",
                "deletedAt": "2026-08-22T02:00:00.000Z"
            }],
            "settings": {
                "theme": "moonlit-water",
                "sidebarCollapsed": true,
                "windowDisplayMode": "both",
                "google": {
                    "enabled": true,
                    "client": null,
                    "accounts": [{
                        "id": "active",
                        "email": "active@example.invalid",
                        "displayName": "Active",
                        "calendarId": "primary",
                        "calendarName": "Main",
                        "syncEnabled": true,
                        "syncToken": "sync-token-is-not-a-credential"
                    }]
                }
            }
        });
        fs::write(
            &path,
            serde_json::to_vec_pretty(&legacy_json).expect("legacy JSON should serialize"),
        )
        .expect("legacy file should be written");

        let migrated = load_app_data_from_path(&path).expect("version 3 file should migrate");
        assert_eq!(migrated.version, CALENDAR_DATA_VERSION);
        assert_eq!(migrated.events.len(), 1);
        assert_eq!(migrated.events[0].title, "移行前の予定");
        assert_eq!(migrated.events[0].end_date, "2026-08-23");
        assert_eq!(migrated.deleted_events.len(), 1);
        assert_eq!(migrated.deleted_events[0].event.title, "削除済みの予定");
        assert_eq!(migrated.settings.theme, "moonlit-water");
        assert!(migrated.settings.sidebar_collapsed);
        assert_eq!(migrated.settings.google.accounts.len(), 1);
        assert_eq!(
            migrated.settings.google.accounts[0].email,
            "active@example.invalid"
        );
        assert!(migrated.settings.google.default_sync_targets.is_empty());
        assert!(migration_backup.exists());
        let backup: AppData = serde_json::from_slice(
            &fs::read(&migration_backup).expect("migration backup should be readable"),
        )
        .expect("migration backup should remain valid JSON");
        assert_eq!(backup.version, 3);
        assert_eq!(backup.events[0].title, "移行前の予定");
        assert_eq!(backup.deleted_events[0].event.title, "削除済みの予定");

        fs::remove_dir_all(&directory).expect("test directory should be removed");
    }

    #[test]
    fn google_defaults_keep_only_unique_active_accounts() {
        let mut google: GoogleIntegrationSettings = serde_json::from_value(serde_json::json!({
            "enabled": true,
            "client": null,
            "accounts": [
                {
                    "id": "active",
                    "email": "active@example.invalid",
                    "displayName": "Active",
                    "calendarId": "primary",
                    "calendarName": "Main",
                    "syncEnabled": true
                },
                {
                    "id": "paused",
                    "email": "paused@example.invalid",
                    "displayName": "Paused",
                    "calendarId": "primary",
                    "calendarName": "Main",
                    "syncEnabled": false
                }
            ],
            "defaultSyncTargets": ["active", "active", "paused", "missing"]
        }))
        .expect("Google settings should deserialize");

        assert!(!google_default_sync_targets_are_valid(&google));
        assert!(normalize_google_default_sync_targets(&mut google));
        assert_eq!(google.default_sync_targets, vec!["active"]);
        assert!(google_default_sync_targets_are_valid(&google));
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
        assert!(event.end_date.is_empty());

        let mut normalized = event;
        assert!(normalize_event_range(&mut normalized));
        assert_eq!(normalized.end_date, "2026-07-22");
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
    fn packaged_runtime_profile_uses_package_local_state_and_startup_task() {
        let profile = runtime_profile(
            true,
            Path::new(r"C:\\Program Files\\WindowsApps\\Koyomado\\koyomado.exe"),
            Some(Path::new(
                r"C:\\Users\\Example\\AppData\\Local\\Packages\\Y-TEC.Koyomado_y7q84f7nwz24j\\LocalState",
            )),
        )
        .expect("packaged apps should use their package LocalState directory");

        assert_eq!(
            profile.data_dir,
            PathBuf::from(
                r"C:\\Users\\Example\\AppData\\Local\\Packages\\Y-TEC.Koyomado_y7q84f7nwz24j\\LocalState\\Koyomado\\data",
            )
        );
        assert_eq!(profile.startup_backend, StartupBackend::Packaged);
    }

    #[test]
    fn packaged_runtime_profile_rejects_unscoped_local_app_data() {
        let error = runtime_profile(
            true,
            Path::new(r"C:\\Program Files\\WindowsApps\\Koyomado\\koyomado.exe"),
            Some(Path::new(r"C:\\Users\\Example\\AppData\\Local")),
        )
        .expect_err("packaged apps must not write through virtualized AppData paths");

        assert!(error.contains("LocalState"));
    }

    #[test]
    fn packaged_runtime_profile_keeps_unpacked_data_adjacent_and_vbs_backend() {
        let profile = runtime_profile(
            false,
            Path::new(r"D:\\Portable\\Koyomado\\koyomado.exe"),
            Some(Path::new(r"C:\\Users\\Example\\AppData\\Local")),
        )
        .expect("portable apps should keep their data next to the executable");

        assert_eq!(
            profile.data_dir,
            PathBuf::from(r"D:\\Portable\\Koyomado\\data")
        );
        assert_eq!(profile.startup_backend, StartupBackend::Portable);
    }

    #[test]
    fn packaged_runtime_startup_state_reports_user_and_policy_blocks() {
        assert_eq!(
            packaged_startup_state_is_enabled(PackagedStartupState::Disabled),
            Ok(false)
        );
        assert_eq!(
            packaged_startup_state_is_enabled(PackagedStartupState::Enabled),
            Ok(true)
        );

        let user_error = packaged_startup_state_is_enabled(PackagedStartupState::DisabledByUser)
            .expect_err("a user-disabled task must not be reported as enabled");
        assert!(user_error.contains("ユーザー"));
        assert!(user_error.contains("スタートアップ"));

        let policy_error =
            packaged_startup_state_is_enabled(PackagedStartupState::DisabledByPolicy)
                .expect_err("a policy-disabled task must not be reported as enabled");
        assert!(policy_error.contains("ポリシー"));
    }

    #[test]
    fn auto_start_script_waits_for_portable_executable_and_avoids_duplicates() {
        let target = PathBuf::from(r"G:\マイドライブ\Koyomado\koyomado.exe");
        let script = build_auto_start_script(&target).expect("script should be generated");
        assert!(script.contains(r#"target = "G:\マイドライブ\Koyomado\koyomado.exe""#));
        assert!(script.contains("For retry = 1 To 150"));
        assert!(script.contains("WScript.Sleep 2000"));
        assert!(script.contains("alreadyRunning"));
        assert!(script.contains("fso.FileExists(target)"));
    }

    #[test]
    fn auto_start_script_is_encoded_as_utf16le_with_bom() {
        let encoded = encode_utf16le_with_bom("予定");
        assert_eq!(&encoded[..2], &[0xff, 0xfe]);
        assert_eq!(
            &encoded[2..],
            &[0x88, 0x4e, 0x9a, 0x5b],
            "Japanese paths must remain readable by Windows Script Host"
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
