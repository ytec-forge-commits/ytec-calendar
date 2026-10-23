use super::*;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::Read;

type Files = BTreeMap<String, Vec<u8>>;

// Only the uniquely named temporary file created by this export is owned here.
// Keep failures from accumulating another partial ZIP on every timer retry.
struct ExportTemporary(PathBuf);
impl Drop for ExportTemporary {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

const MAX_TOTAL: u64 = 256 * 1024 * 1024;
const MAX_FILE: u64 = 32 * 1024 * 1024;
const MAX_ENTRIES: usize = 2048;
const RESTORE_STATE: &str = "google-restore-state.json";

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    format: String,
    version: u32,
    created_at: String,
    files: BTreeMap<String, Entry>,
}
#[derive(Serialize, Deserialize)]
struct Entry {
    size: u64,
    sha256: String,
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn safe_name(name: &str) -> bool {
    let parts: Vec<_> = name.split('/').collect();
    let valid = |part: &&str| {
        !part.is_empty()
            && *part != "."
            && *part != ".."
            && !part.ends_with([' ', '.'])
            && !part
                .chars()
                .any(|c| c.is_control() || "\\:<>\"|?*".contains(c))
            && !matches!(
                part.split('.')
                    .next()
                    .unwrap_or("")
                    .to_ascii_uppercase()
                    .as_str(),
                "CON"
                    | "PRN"
                    | "AUX"
                    | "NUL"
                    | "COM1"
                    | "COM2"
                    | "COM3"
                    | "COM4"
                    | "COM5"
                    | "COM6"
                    | "COM7"
                    | "COM8"
                    | "COM9"
                    | "LPT1"
                    | "LPT2"
                    | "LPT3"
                    | "LPT4"
                    | "LPT5"
                    | "LPT6"
                    | "LPT7"
                    | "LPT8"
                    | "LPT9"
            )
    };
    parts.iter().all(valid)
        && (name == "calendar-data.json"
            || name == "manifest.json"
            || (parts.len() == 2 && parts[0] == "notification-sounds"))
}
fn portable_data(mut data: AppData) -> AppData {
    data.settings.google.client = None;
    for account in &mut data.settings.google.accounts {
        account.sync_token.clear();
    }
    data
}
fn collect_snapshot(source: &Path) -> Result<Files, String> {
    if fs::metadata(source.join("calendar-data.json"))
        .map_err(|e| e.to_string())?
        .len()
        > MAX_FILE
    {
        return Err("予定データが32MBを超えています".into());
    }
    let data = load_app_data_from_path(&source.join("calendar-data.json"))?;
    let before = fs::read(source.join("calendar-data.json")).map_err(|e| e.to_string())?;
    let mut files = Files::new();
    files.insert(
        "calendar-data.json".into(),
        serde_json::to_vec_pretty(&portable_data(data)).map_err(|e| e.to_string())?,
    );
    let sounds = source.join("notification-sounds");
    if sounds.exists() {
        if fs::symlink_metadata(&sounds)
            .map_err(|e| e.to_string())?
            .file_type()
            .is_symlink()
        {
            return Err("通知音フォルダーのリンクはバックアップできません".into());
        }
        for entry in fs::read_dir(&sounds).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            if !entry.file_type().map_err(|e| e.to_string())?.is_file() {
                return Err("通知音フォルダーには通常ファイルだけを保存してください".into());
            }
            let name = format!(
                "notification-sounds/{}",
                entry.file_name().to_string_lossy()
            );
            if !safe_name(&name) || entry.metadata().map_err(|e| e.to_string())?.len() > MAX_FILE {
                return Err("バックアップできない通知音ファイルです".into());
            }
            files.insert(name, fs::read(entry.path()).map_err(|e| e.to_string())?);
        }
    }
    if fs::read(source.join("calendar-data.json")).map_err(|e| e.to_string())? != before {
        return Err("バックアップ中に予定が変更されました。もう一度お試しください".into());
    }
    if files.len() >= MAX_ENTRIES
        || files.values().any(|b| b.len() as u64 > MAX_FILE)
        || files.values().map(|b| b.len() as u64).sum::<u64>() > MAX_TOTAL
    {
        return Err("バックアップの容量制限を超えています（合計256MB、1ファイル32MB）".into());
    }
    Ok(files)
}
fn write_archive(files: &Files, destination: &Path) -> Result<(), String> {
    if !destination.is_absolute()
        || destination
            .extension()
            .is_none_or(|ext| !ext.eq_ignore_ascii_case("zip"))
    {
        return Err("ZIPの絶対パスを指定してください".into());
    }
    if destination.exists() {
        return Err("同名ファイルがあるため上書きせず停止しました".into());
    }
    let manifest = Manifest {
        format: "koyomado-backup".into(),
        version: 1,
        created_at: chrono::Utc::now().to_rfc3339(),
        files: files
            .iter()
            .map(|(name, bytes)| {
                (
                    name.clone(),
                    Entry {
                        size: bytes.len() as u64,
                        sha256: digest(bytes),
                    },
                )
            })
            .collect(),
    };
    let tmp = destination.with_extension(format!(
        "zip.{}.tmp",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    let file = OpenOptions::new()
        .create_new(true)
        .read(true)
        .write(true)
        .open(&tmp)
        .map_err(|e| format!("ZIPの保存先を利用できません: {e}"))?;
    let _temporary = ExportTemporary(tmp.clone());
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    for (name, bytes) in files {
        zip.start_file(name, options).map_err(|e| e.to_string())?;
        zip.write_all(bytes).map_err(|e| e.to_string())?;
    }
    zip.start_file("manifest.json", options)
        .map_err(|e| e.to_string())?;
    zip.write_all(&serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    zip.finish()
        .map_err(|e| e.to_string())?
        .sync_all()
        .map_err(|e| e.to_string())?;
    if read_archive(&tmp)? != *files {
        return Err("ZIPの照合に失敗しました。元データを保持しています".into());
    }
    // Windows rename fails if the destination appeared after our existence check.
    fs::rename(&tmp, destination).map_err(|e| format!("ZIPの確定に失敗しました: {e}"))
}
fn export_to(source: &Path, destination: &Path) -> Result<(), String> {
    write_archive(&collect_snapshot(source)?, destination)
}
fn read_archive(path: &Path) -> Result<Files, String> {
    let file = File::open(path).map_err(|e| format!("ZIPを開けません: {e}"))?;
    if file.metadata().map_err(|e| e.to_string())?.len() > MAX_TOTAL {
        return Err("ZIPが大きすぎます（256MBまで）".into());
    }
    let mut zip = zip::ZipArchive::new(file).map_err(|_| "KoyomadoのZIPを読み込めません")?;
    if zip.len() > MAX_ENTRIES {
        return Err("ZIPのファイル数が多すぎます".into());
    }
    let mut files = Files::new();
    let mut names = HashSet::new();
    let mut total = 0u64;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).map_err(|_| "ZIPの内容を読み込めません")?;
        let name = entry
            .name()
            .map_err(|_| "ZIPのファイル名を読み込めません")?
            .into_owned();
        if !safe_name(&name)
            || entry.enclosed_name().is_none()
            || entry.is_symlink()
            || entry.is_dir()
            || entry.encrypted()
            || !names.insert(name.to_lowercase())
        {
            return Err("ZIPに不正なパス、重複名、または非対応ファイルが含まれています".into());
        }
        total = total
            .checked_add(entry.size())
            .ok_or("ZIPの容量を確認できません")?;
        if entry.size() > MAX_FILE || total > MAX_TOTAL {
            return Err("ZIPの展開容量が制限を超えています".into());
        }
        let mut bytes = Vec::new();
        entry
            .by_ref()
            .take(MAX_FILE + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| "ZIPが破損しています")?;
        if bytes.len() as u64 > MAX_FILE || bytes.len() as u64 != entry.size() {
            return Err("ZIP内のサイズが一致しません".into());
        }
        files.insert(name, bytes);
    }
    let manifest: Manifest = serde_json::from_slice(
        &files
            .remove("manifest.json")
            .ok_or("Koyomadoのバックアップ情報がありません")?,
    )
    .map_err(|_| "バックアップ情報が壊れています")?;
    if manifest.format != "koyomado-backup"
        || manifest.version != 1
        || manifest.files.len() != files.len()
    {
        return Err("非対応のバックアップ形式です".into());
    }
    for (name, bytes) in &files {
        let expected = manifest
            .files
            .get(name)
            .ok_or("ZIPのファイル一覧が一致しません")?;
        if expected.size != bytes.len() as u64 || expected.sha256 != digest(bytes) {
            return Err("ZIPの整合性確認に失敗しました".into());
        }
    }
    let data: AppData = serde_json::from_slice(
        files
            .get("calendar-data.json")
            .ok_or("予定データがありません")?,
    )
    .map_err(|_| "予定データが壊れています")?;
    if !(1..=CALENDAR_DATA_VERSION).contains(&data.version) {
        return Err("予定データの版に対応していません".into());
    }
    let mut ids = HashSet::new();
    if data
        .events
        .iter()
        .any(|e| e.id.is_empty() || !ids.insert(e.id.clone()))
    {
        return Err("現在の予定IDが空、または重複しています".into());
    }
    // A Google event restored after deletion keeps its stable ID. Each deletion
    // is a separate historical record and may share an ID with a live event.
    if data.deleted_events.iter().any(|d| d.event.id.is_empty()) {
        return Err("削除履歴の予定IDが空です".into());
    }
    if let Some(sound) = &data.settings.notifications.custom_sound {
        if !files.contains_key(&format!("notification-sounds/{}", sound.stored_file_name)) {
            return Err("設定された通知音がZIPにありません".into());
        }
    }
    Ok(files)
}

#[derive(Serialize, Deserialize, Default)]
pub(super) struct RestoreState {
    version: u32,
    pending_accounts: Vec<String>,
}
pub(super) fn restore_state(directory: &Path) -> Result<RestoreState, String> {
    let path = directory.join(RESTORE_STATE);
    if !path.exists() {
        return Ok(RestoreState {
            version: 1,
            ..Default::default()
        });
    }
    let state: RestoreState =
        serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|_| "復元後のGoogle確認状態を読み込めません。同期を停止しました")?;
    if state.version != 1 {
        return Err("復元後のGoogle確認状態の版に対応していません".into());
    }
    Ok(state)
}
impl RestoreState {
    pub(super) fn pending(&self, account: &str) -> bool {
        self.pending_accounts.iter().any(|id| id == account)
    }
    pub(super) fn complete(&mut self, account: &str) {
        self.pending_accounts.retain(|id| id != account);
    }
    pub(super) fn save(&self, directory: &Path) -> Result<(), String> {
        write_json_file(&directory.join(RESTORE_STATE), self, true)
    }
}
fn restore_to(archive: &Path, target: &Path) -> Result<AppData, String> {
    let files = read_archive(archive)?;
    if !target.is_dir()
        || fs::read_dir(target)
            .map_err(|e| e.to_string())?
            .any(|entry| {
                entry
                    .map(|e| e.file_name() != "calendar-data.lock")
                    .unwrap_or(true)
            })
    {
        return Err("復元先は空のフォルダーを選択してください。既存データは上書きしません".into());
    }
    let pending = target.join(".koyomado-migration-pending");
    let mut marker = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&pending)
        .map_err(|e| e.to_string())?;
    marker
        .write_all(b"Koyomado restore v1")
        .and_then(|_| marker.sync_all())
        .map_err(|e| e.to_string())?;
    drop(marker);
    for (name, bytes) in files {
        let path = target.join(name);
        fs::create_dir_all(path.parent().ok_or("復元先を確認できません")?)
            .map_err(|e| e.to_string())?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|e| e.to_string())?;
        file.write_all(&bytes)
            .and_then(|_| file.sync_all())
            .map_err(|e| e.to_string())?;
        drop(file);
        if fs::read(&path).map_err(|e| e.to_string())? != bytes {
            return Err("復元したファイルの照合に失敗しました".into());
        }
    }
    let mut data = load_app_data_from_path(&target.join("calendar-data.json"))?;
    data = portable_data(data);
    data.settings.google.enabled = false;
    data.settings.google.default_sync_targets.clear();
    for account in &mut data.settings.google.accounts {
        account.sync_enabled = false;
        account.needs_reauth = true;
        account.last_error = "ZIP復元後は再認証し、最初にGoogle側を受信・照合します".into();
    }
    let state = RestoreState {
        version: 1,
        pending_accounts: data
            .settings
            .google
            .accounts
            .iter()
            .map(|a| a.id.clone())
            .collect(),
    };
    write_json_file(&target.join(RESTORE_STATE), &state, false)?;
    write_json_with_backup(&target.join("calendar-data.json"), &data)?;
    let result = load_app_data_from_path(&target.join("calendar-data.json"))?;
    fs::rename(
        &pending,
        target.join(format!(
            ".koyomado-migration-complete-restore-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        )),
    )
    .map_err(|e| e.to_string())?;
    Ok(result)
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OwnedBackup {
    name: String,
    sha256: String,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AutoBackup {
    version: u32,
    enabled: bool,
    directory: String,
    frequency: String,
    generations: usize,
    series: String,
    last_period: String,
    last_source: String,
    last_success: String,
    last_error: String,
    owned: Vec<OwnedBackup>,
}
impl Default for AutoBackup {
    fn default() -> Self {
        Self {
            version: 1,
            enabled: false,
            directory: String::new(),
            frequency: "daily".into(),
            generations: 10,
            series: String::new(),
            last_period: String::new(),
            last_source: String::new(),
            last_success: String::new(),
            last_error: String::new(),
            owned: vec![],
        }
    }
}
fn period(frequency: &str, date: chrono::NaiveDate) -> Result<String, String> {
    use chrono::Datelike;
    match frequency {
        "daily" => Ok(date.format("%Y-%m-%d").to_string()),
        "weekly" => Ok((date
            - chrono::Duration::days(i64::from(date.weekday().num_days_from_monday())))
        .format("%Y-%m-%d")
        .to_string()),
        "monthly" => Ok(date.format("%Y-%m").to_string()),
        _ => Err("バックアップの間隔が正しくありません".into()),
    }
}
fn run_due(
    source: &Path,
    state: &mut AutoBackup,
    date: chrono::NaiveDate,
) -> Result<Option<String>, String> {
    if !state.enabled {
        return Ok(None);
    }
    if !(1..=100).contains(&state.generations) {
        return Err("残す世代数は1～100で指定してください".into());
    }
    let period = period(&state.frequency, date)?;
    let source = fs::canonicalize(source).map_err(|e| e.to_string())?;
    let source_id = source.display().to_string();
    if state.last_period == period && state.last_source == source_id {
        return Ok(None);
    }
    let directory = fs::canonicalize(&state.directory)
        .map_err(|_| "自動バックアップ先を利用できません。接続状態を確認してください")?;
    if !directory.is_dir() || directory.starts_with(&source) || source.starts_with(&directory) {
        return Err("バックアップ先は予定データとは別のフォルダーを指定してください".into());
    }
    if state.series.is_empty() {
        state.series = format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
    }
    let prefix = format!("koyomado-auto-{}-", state.series);
    let name = format!(
        "{prefix}{}.zip",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    let path = directory.join(&name);
    export_to(&source, &path)?;
    let bytes = fs::read(&path).map_err(|e| e.to_string())?;
    state.owned.push(OwnedBackup {
        name,
        sha256: digest(&bytes),
    });
    state.last_period = period;
    state.last_source = source_id;
    state.last_success = chrono::Local::now().to_rfc3339();
    state.last_error.clear();
    while state.owned.len() > state.generations {
        let oldest = &state.owned[0];
        let old_path = directory.join(&oldest.name);
        // Only files in this PC's recorded automatic series, with identical
        // bytes, are eligible. Never follow links or touch manually named ZIPs.
        let allowed = oldest.name.starts_with(&prefix)
            && oldest.name.ends_with(".zip")
            && Path::new(&oldest.name).components().count() == 1;
        let metadata = fs::symlink_metadata(&old_path);
        if metadata
            .as_ref()
            .is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound)
        {
            state.owned.remove(0);
            continue;
        }
        let verified = allowed
            && metadata
                .is_ok_and(|m| m.is_file() && !m.file_type().is_symlink() && m.len() <= MAX_TOTAL)
            && fs::read(&old_path).is_ok_and(|b| digest(&b) == oldest.sha256);
        if !verified {
            state.last_error="新しいZIPは保存済みです。内容などが変わった古いファイルは削除せず、自動管理から外して残しました。".into();
            state.owned.remove(0);
            continue;
        }
        if fs::remove_file(&old_path).is_err() {
            state.last_error =
                "新しいZIPは保存済みです。古い世代を安全に整理できなかったため残しました。".into();
            break;
        }
        state.owned.remove(0);
    }
    Ok(Some(path.display().to_string()))
}

fn auto_path() -> Result<PathBuf, String> {
    let (_, config) = storage::current_paths()?;
    Ok(config.with_file_name(format!(
        "{}-backup-settings.json",
        config
            .file_stem()
            .ok_or("設定先を確認できません")?
            .to_string_lossy()
    )))
}
fn read_auto(path: &Path) -> Result<AutoBackup, String> {
    let state: AutoBackup = read_json_with_recovery(path)?.unwrap_or_default();
    if state.version != 1 || !(1..=100).contains(&state.generations) || state.owned.len() > 4096 {
        return Err("自動バックアップの設定を読み込めません".into());
    }
    Ok(state)
}
#[tauri::command]
pub(super) fn get_auto_backup() -> Result<AutoBackup, String> {
    let path = auto_path()?;
    let _lock = CalendarFileLock::acquire(&path)?;
    read_auto(&path)
}
#[tauri::command]
pub(super) fn configure_auto_backup(
    enabled: bool,
    directory: String,
    frequency: String,
    generations: usize,
) -> Result<AutoBackup, String> {
    period(&frequency, chrono::Local::now().date_naive())?;
    if !(1..=100).contains(&generations) {
        return Err("残す世代数は1～100で指定してください".into());
    }
    let _guard = APP_DATA_LOCK
        .lock()
        .map_err(|_| "バックアップ設定を開始できません")?;
    let path = auto_path()?;
    let _lock = CalendarFileLock::acquire(&path)?;
    let mut state = read_auto(&path)?;
    let directory = if enabled {
        let target = fs::canonicalize(&directory)
            .map_err(|_| "先にバックアップ用フォルダーを作成してください")?;
        let source = storage::current_directory()?;
        let source = fs::canonicalize(source).map_err(|e| e.to_string())?;
        if !target.is_dir() || target.starts_with(&source) || source.starts_with(&target) {
            return Err("バックアップ先は予定データとは別のフォルダーを指定してください".into());
        }
        target.display().to_string()
    } else {
        directory
    };
    if state.directory != directory {
        state.owned.clear();
        state.series.clear();
        state.last_period.clear();
    }
    if state.frequency != frequency {
        state.last_period.clear();
    }
    state.enabled = enabled;
    state.directory = directory;
    state.frequency = frequency;
    state.generations = generations;
    write_json_file(&path, &state, path.exists())?;
    Ok(state)
}
#[tauri::command]
pub(super) async fn run_due_backup() -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let _guard = APP_DATA_LOCK
            .lock()
            .map_err(|_| "バックアップを開始できません")?;
        let path = auto_path()?;
        let _lock = CalendarFileLock::acquire(&path)?;
        let mut state = read_auto(&path)?;
        if !state.enabled {
            return Ok(None);
        }
        let result = storage::with_current(|source| {
            run_due(source, &mut state, chrono::Local::now().date_naive())
        });
        if let Err(error) = &result {
            state.last_error = error.clone();
        }
        write_json_file(&path, &state, path.exists())?;
        result
    })
    .await
    .map_err(|e| e.to_string())?
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct BackupPreview {
    token: String,
    created_at: String,
    event_count: usize,
    deleted_event_count: usize,
    sound_count: usize,
    google_account_count: usize,
    target_directory: String,
}
struct RestorePlan {
    token: String,
    archive: PathBuf,
    checksum: String,
    target: PathBuf,
    source: PathBuf,
}
static RESTORE_PLAN: Mutex<Option<RestorePlan>> = Mutex::new(None);
fn checked_archive_hash(path: &Path) -> Result<String, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    if file.metadata().map_err(|e| e.to_string())?.len() > MAX_TOTAL {
        return Err("ZIPが大きすぎます".into());
    }
    let mut bytes = Vec::new();
    file.take(MAX_TOTAL + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_TOTAL {
        return Err("ZIPが大きすぎます".into());
    }
    Ok(digest(&bytes))
}
#[tauri::command]
pub(super) async fn export_backup(destination: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = APP_DATA_LOCK
            .lock()
            .map_err(|_| "バックアップを開始できません")?;
        storage::with_current(|source| export_to(source, Path::new(&destination)))?;
        Ok(destination)
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub(super) async fn preview_backup_restore(
    archive: String,
    target: String,
) -> Result<BackupPreview, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = APP_DATA_LOCK
            .lock()
            .map_err(|_| "ZIP確認を開始できません")?;
        let archive = PathBuf::from(archive);
        let checksum = checked_archive_hash(&archive)?;
        let files = read_archive(&archive)?;
        let (default, config) = storage::current_paths()?;
        let (target, source) =
            storage::empty_restore_target(&default, &config, Path::new(&target))?;
        let data: AppData =
            serde_json::from_slice(&files["calendar-data.json"]).map_err(|e| e.to_string())?;
        let token = format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        let manifest: Manifest = read_manifest(&archive)?;
        let preview = BackupPreview {
            token: token.clone(),
            created_at: manifest.created_at,
            event_count: data.events.len(),
            deleted_event_count: data.deleted_events.len(),
            sound_count: files.len() - 1,
            google_account_count: data.settings.google.accounts.len(),
            target_directory: target.display().to_string(),
        };
        *RESTORE_PLAN
            .lock()
            .map_err(|_| "復元内容を保持できません")? = Some(RestorePlan {
            token,
            archive,
            checksum,
            target,
            source,
        });
        Ok(preview)
    })
    .await
    .map_err(|e| e.to_string())?
}
fn read_manifest(path: &Path) -> Result<Manifest, String> {
    let mut zip = zip::ZipArchive::new(File::open(path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    let mut bytes = Vec::new();
    zip.by_name("manifest.json")
        .map_err(|e| e.to_string())?
        .take(MAX_FILE + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}
#[tauri::command]
pub(super) async fn apply_backup_restore(
    token: String,
) -> Result<storage::CalendarSnapshot, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = APP_DATA_LOCK.lock().map_err(|_| "復元を開始できません")?;
        let plan = RESTORE_PLAN
            .lock()
            .map_err(|_| "復元内容を参照できません")?
            .take()
            .ok_or("先にZIPを確認してください")?;
        if token != plan.token || checked_archive_hash(&plan.archive)? != plan.checksum {
            return Err("ZIPまたは確認内容が変わりました。もう一度確認してください".into());
        }
        let (default, config) = storage::current_paths()?;
        let (target, source) = storage::empty_restore_target(&default, &config, &plan.target)?;
        if target != plan.target || source != plan.source {
            return Err("確認後に保存先が変わりました。もう一度確認してください".into());
        }
        let restored = {
            let _lock = CalendarFileLock::acquire(&target.join("calendar-data.json"))?;
            restore_to(&plan.archive, &target)?
        };
        if checked_archive_hash(&plan.archive)? != plan.checksum {
            return Err("復元中にZIPが変更されたため保存先の切替を停止しました".into());
        }
        storage::activate_restored(&default, &config, &target, &source, &restored)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    fn area(name: &str) -> PathBuf {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../output/native-smoke-backup-20261017/tests")
            .join(format!(
                "{name}-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
        fs::create_dir_all(&path).unwrap();
        path
    }
    fn seed(root: &Path) {
        fs::create_dir_all(root.join("notification-sounds")).unwrap();
        let mut data = AppData::default();
        data.settings.theme = "forest-breath".into();
        data.settings.google.client = Some(GoogleOAuthClient {
            client_id: "synthetic-client".into(),
            client_secret: "synthetic-secret".into(),
            project_id: "synthetic-project".into(),
        });
        data.settings.google.enabled = true;
        data.settings.google.accounts.push(serde_json::from_value(serde_json::json!({"id":"synthetic-account","email":"synthetic@example.invalid","calendarId":"synthetic-calendar","syncEnabled":true,"syncToken":"synthetic-sync-token"})).unwrap());
        fs::write(
            root.join("calendar-data.json"),
            serde_json::to_vec(&data).unwrap(),
        )
        .unwrap();
        fs::write(
            root.join("notification-sounds/test.wav"),
            b"synthetic-audio",
        )
        .unwrap();
        fs::write(root.join("window-state.json"), b"must-not-travel").unwrap();
    }
    #[test]
    fn backup_retains_live_event_and_multiple_deletion_records_with_the_same_id() {
        let root = area("reused-google-id");
        let source = root.join("source");
        seed(&source);
        let path = source.join("calendar-data.json");
        let mut data = load_app_data_from_path(&path).unwrap();
        let event: CalendarEvent = serde_json::from_value(serde_json::json!({
            "id":"google-synthetic-reused", "title":"synthetic restored event",
            "date":"2026-10-17", "endDate":"2026-10-17", "allDay":true,
            "startTime":"", "endTime":"", "location":"", "notes":"",
            "style":{"color":"#83a9c2"}, "createdAt":"2026-10-01T00:00:00Z",
            "updatedAt":"2026-10-17T00:00:00Z"
        }))
        .unwrap();
        data.events.push(event.clone());
        data.deleted_events.push(DeletedCalendarEvent {
            event: event.clone(),
            deleted_at: "2026-10-02T00:00:00Z".into(),
        });
        data.deleted_events.push(DeletedCalendarEvent {
            event,
            deleted_at: "2026-10-03T00:00:00Z".into(),
        });
        fs::write(&path, serde_json::to_vec(&data).unwrap()).unwrap();
        let original = fs::read(&path).unwrap();
        let archive = root.join("backup.zip");
        export_to(&source, &archive).unwrap();
        let target = root.join("restored");
        fs::create_dir(&target).unwrap();
        let restored = restore_to(&archive, &target).unwrap();
        assert_eq!(restored.events.len(), 1);
        assert_eq!(restored.deleted_events.len(), 2);
        assert_eq!(restored.events[0].id, "google-synthetic-reused");
        assert!(restored
            .deleted_events
            .iter()
            .all(|item| item.event.id == restored.events[0].id));
        assert_eq!(
            restored.deleted_events[0].deleted_at,
            "2026-10-02T00:00:00Z"
        );
        assert_eq!(
            restored.deleted_events[1].deleted_at,
            "2026-10-03T00:00:00Z"
        );
        assert_eq!(fs::read(&path).unwrap(), original);
    }

    #[test]
    fn backup_recovers_duplicate_live_ids_and_keeps_all_records_and_original_bytes() {
        let root = area("duplicate-live-id");
        let source = root.join("source");
        seed(&source);
        let path = source.join("calendar-data.json");
        let mut data = load_app_data_from_path(&path).unwrap();
        let event: CalendarEvent = serde_json::from_value(serde_json::json!({
            "id":"duplicate", "title":"synthetic", "date":"2026-10-17", "endDate":"2026-10-17",
            "allDay":true, "startTime":"", "endTime":"", "location":"", "notes":"",
            "style":{"color":"#83a9c2"},
            "createdAt":"2026-10-01T00:00:00Z", "updatedAt":"2026-10-17T00:00:00Z"
        }))
        .unwrap();
        data.events.extend([event.clone(), event]);
        fs::write(&path, serde_json::to_vec(&data).unwrap()).unwrap();
        let original = fs::read(&path).unwrap();
        let archive = root.join("backup.zip");
        export_to(&source, &archive).unwrap();
        let target = root.join("restored");
        fs::create_dir(&target).unwrap();
        let restored = restore_to(&archive, &target).unwrap();
        assert_eq!(restored.events.len(), 2);
        assert_ne!(restored.events[0].id, restored.events[1].id);
        let preserved: Vec<_> = fs::read_dir(&source)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with("calendar-data.before-id-repair.")
            })
            .collect();
        assert_eq!(preserved.len(), 1);
        assert_eq!(fs::read(preserved[0].path()).unwrap(), original);
    }

    #[test]
    fn empty_live_and_history_ids_are_rejected_without_overwriting_the_source() {
        let root = area("empty-id");
        let source = root.join("source");
        seed(&source);
        let path = source.join("calendar-data.json");
        let mut data = load_app_data_from_path(&path).unwrap();
        let mut empty: CalendarEvent = serde_json::from_value(serde_json::json!({
            "id":"", "title":"synthetic", "date":"2026-10-17", "endDate":"2026-10-17",
            "allDay":true, "startTime":"", "endTime":"", "location":"", "notes":"",
            "style":{"color":"#83a9c2"},
            "createdAt":"2026-10-01T00:00:00Z", "updatedAt":"2026-10-17T00:00:00Z"
        }))
        .unwrap();
        let archive = root.join("backup.zip");
        empty.id.clear();
        for deleted in [false, true] {
            data.events.clear();
            data.deleted_events.clear();
            if deleted {
                data.deleted_events.push(DeletedCalendarEvent {
                    event: empty.clone(),
                    deleted_at: "2026-10-17T00:00:00Z".into(),
                });
            } else {
                data.events.push(empty.clone());
            }
            fs::write(&path, serde_json::to_vec(&data).unwrap()).unwrap();
            let before = fs::read(&path).unwrap();
            assert!(export_to(&source, &archive).is_err());
            assert!(!archive.exists());
            assert_eq!(fs::read(&path).unwrap(), before);
        }
    }

    #[test]
    fn export_and_restore_keep_settings_sound_and_source_without_oauth_secret() {
        let root = area("roundtrip");
        let source = root.join("source");
        seed(&source);
        let before = fs::read(source.join("calendar-data.json")).unwrap();
        let archive = root.join("backup.zip");
        export_to(&source, &archive).unwrap();
        let files = read_archive(&archive).unwrap();
        let json = String::from_utf8(files["calendar-data.json"].clone()).unwrap();
        assert!(!json.contains("synthetic-secret"));
        assert!(!json.contains("synthetic-client"));
        assert!(!json.contains("synthetic-sync-token"));
        assert!(!files.contains_key("window-state.json"));
        let target = root.join("restored");
        fs::create_dir(&target).unwrap();
        let data = restore_to(&archive, &target).unwrap();
        assert_eq!(data.settings.theme, "forest-breath");
        assert!(!data.settings.google.enabled);
        assert!(!data.settings.google.accounts[0].sync_enabled);
        assert!(data.settings.google.accounts[0].needs_reauth);
        assert!(restore_state(&target).unwrap().pending("synthetic-account"));
        assert_eq!(
            fs::read(target.join("notification-sounds/test.wav")).unwrap(),
            b"synthetic-audio"
        );
        assert_eq!(fs::read(source.join("calendar-data.json")).unwrap(), before);
    }
    #[test]
    fn existing_archive_and_restore_folder_are_never_overwritten() {
        let root = area("no-overwrite");
        let source = root.join("source");
        seed(&source);
        let archive = root.join("backup.zip");
        export_to(&source, &archive).unwrap();
        let original = fs::read(&archive).unwrap();
        assert!(export_to(&source, &archive).is_err());
        assert_eq!(fs::read(&archive).unwrap(), original);
        assert!(restore_to(&archive, &source).is_err());
        assert!(fs::read_to_string(source.join("calendar-data.json"))
            .unwrap()
            .contains("synthetic-client"));
    }
    #[test]
    fn invalid_archive_does_not_create_calendar_or_pending_marker() {
        let root = area("bad-zip");
        let target = root.join("target");
        fs::create_dir(&target).unwrap();
        let archive = root.join("bad.zip");
        fs::write(&archive, b"broken zip").unwrap();
        assert!(restore_to(&archive, &target).is_err());
        assert_eq!(fs::read_dir(&target).unwrap().count(), 0);
    }
    #[test]
    fn verification_failure_does_not_accumulate_partial_zip_files() {
        let root = area("failed-temporary");
        let mut files = Files::new();
        files.insert("calendar-data.json".into(), b"invalid calendar".to_vec());
        assert!(write_archive(&files, &root.join("failed.zip")).is_err());
        assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
    }
    #[test]
    fn weekly_and_monthly_periods_follow_local_calendar_boundaries() {
        let date = |s: &str| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap();
        assert_eq!(period("weekly", date("2026-10-18")).unwrap(), "2026-10-12");
        assert_eq!(period("weekly", date("2026-10-19")).unwrap(), "2026-10-19");
        assert_eq!(period("monthly", date("2026-12-31")).unwrap(), "2026-12");
        assert_eq!(period("monthly", date("2027-01-01")).unwrap(), "2027-01");
        assert!(period("invalid", date("2026-10-17")).is_err());
    }
    #[test]
    fn automatic_backup_runs_once_per_period_and_prunes_only_owned_verified_files() {
        let root = area("auto");
        let source = root.join("source");
        seed(&source);
        let directory = root.join("backups");
        fs::create_dir(&directory).unwrap();
        fs::write(directory.join("manual.zip"), b"untouched").unwrap();
        let mut state = AutoBackup {
            enabled: true,
            directory: directory.display().to_string(),
            generations: 2,
            ..Default::default()
        };
        let date = |day| chrono::NaiveDate::from_ymd_opt(2026, 10, day).unwrap();
        let first = run_due(&source, &mut state, date(17)).unwrap().unwrap();
        assert!(run_due(&source, &mut state, date(17)).unwrap().is_none());
        run_due(&source, &mut state, date(18)).unwrap().unwrap();
        run_due(&source, &mut state, date(19)).unwrap().unwrap();
        assert!(!Path::new(&first).exists());
        assert_eq!(state.owned.len(), 2);
        assert_eq!(
            fs::read(directory.join("manual.zip")).unwrap(),
            b"untouched"
        );
    }
    #[test]
    fn failed_new_backup_does_not_prune_previous_generation() {
        let root = area("auto-failure");
        let source = root.join("source");
        seed(&source);
        let directory = root.join("backups");
        fs::create_dir(&directory).unwrap();
        let mut state = AutoBackup {
            enabled: true,
            directory: directory.display().to_string(),
            generations: 1,
            ..Default::default()
        };
        let date = |day| chrono::NaiveDate::from_ymd_opt(2026, 10, day).unwrap();
        let first = run_due(&source, &mut state, date(17)).unwrap().unwrap();
        fs::write(source.join("calendar-data.json"), b"corrupt").unwrap();
        assert!(run_due(&source, &mut state, date(18)).is_err());
        assert!(Path::new(&first).exists());
        assert_eq!(state.owned.len(), 1);
    }
    #[test]
    fn modified_automatic_zip_is_preserved_instead_of_pruned() {
        let root = area("auto-modified");
        let source = root.join("source");
        seed(&source);
        let directory = root.join("backups");
        fs::create_dir(&directory).unwrap();
        let mut state = AutoBackup {
            enabled: true,
            directory: directory.display().to_string(),
            generations: 1,
            ..Default::default()
        };
        let date = |day| chrono::NaiveDate::from_ymd_opt(2026, 10, day).unwrap();
        let first = run_due(&source, &mut state, date(17)).unwrap().unwrap();
        fs::write(&first, b"a file replaced by user").unwrap();
        run_due(&source, &mut state, date(18)).unwrap().unwrap();
        assert_eq!(fs::read(&first).unwrap(), b"a file replaced by user");
        assert!(!state.last_error.is_empty());
        assert_eq!(
            state.owned.len(),
            1,
            "changed file must not block retention of later verified backups"
        );
    }
    #[test]
    fn unsafe_path_and_manifest_checksum_mismatch_are_rejected_before_restore() {
        let root = area("zip-validation");
        for (label, name) in [
            ("parent", "../calendar-data.json"),
            ("absolute", "C:/calendar-data.json"),
            ("stream", "notification-sounds/a.wav:stream"),
            ("device", "notification-sounds/CON.wav"),
        ] {
            let path = root.join(format!("{label}.zip"));
            let mut zip = zip::ZipWriter::new(File::create(&path).unwrap());
            zip.start_file(name, zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(b"synthetic").unwrap();
            zip.finish().unwrap();
            assert!(
                read_archive(&path).is_err(),
                "unsafe name accepted: {label}"
            );
        }
        let source = root.join("source");
        seed(&source);
        let path = root.join("checksum.zip");
        export_to(&source, &path).unwrap();
        let files = read_archive(&path).unwrap();
        let mut zip = zip::ZipWriter::new(File::create(root.join("changed.zip")).unwrap());
        let options = zip::write::SimpleFileOptions::default();
        let manifest = Manifest {
            format: "koyomado-backup".into(),
            version: 1,
            created_at: "2026-10-17T00:00:00Z".into(),
            files: files
                .iter()
                .map(|(n, b)| {
                    (
                        n.clone(),
                        Entry {
                            size: b.len() as u64,
                            sha256: "wrong".into(),
                        },
                    )
                })
                .collect(),
        };
        for (name, bytes) in files {
            zip.start_file(name, options).unwrap();
            zip.write_all(&bytes).unwrap();
        }
        zip.start_file("manifest.json", options).unwrap();
        zip.write_all(&serde_json::to_vec(&manifest).unwrap())
            .unwrap();
        zip.finish().unwrap();
        let target = root.join("target");
        fs::create_dir(&target).unwrap();
        assert!(restore_to(&root.join("changed.zip"), &target).is_err());
        assert_eq!(fs::read_dir(&target).unwrap().count(), 0);
    }
}
