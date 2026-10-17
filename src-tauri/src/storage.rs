use super::*;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StorageLocation {
    version: u32,
    data_directory: PathBuf,
}

const PENDING_COPY: &str = ".koyomado-migration-pending";

fn reject_pending_copy(directory: &Path) -> Result<(), String> {
    if directory.join(PENDING_COPY).exists() {
        return Err("このフォルダーにはコピー途中のデータがあります。元の保存先を保全し、別の空フォルダーでやり直してください".into());
    }
    Ok(())
}

fn location_directory(default: &Path, config: &Path) -> Result<PathBuf, String> {
    // A .tmp pointer is an uncommitted switch, never an active location.
    let parse = |path: &Path| -> Result<StorageLocation, String> {
        serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|_| "保存先の設定が壊れています".into())
    };
    let location = if config.exists() {
        Some(parse(config).or_else(|_| parse(&backup_path(config)))?)
    } else if backup_path(config).exists() {
        Some(parse(&backup_path(config))?)
    } else {
        None
    };
    match location {
        Some(location) if location.version == 1 && location.data_directory.is_absolute() => {
            Ok(location.data_directory)
        }
        Some(_) => Err("保存先の設定を読み込めません。元の設定を保護して停止しました".into()),
        None => Ok(default.to_path_buf()),
    }
}

fn resolve_directory(default: &Path, config: &Path) -> Result<PathBuf, String> {
    let directory = location_directory(default, config)?;
    reject_pending_copy(&directory)?;
    if directory == default {
        return Ok(directory);
    }
    let directory = fs::canonicalize(&directory)
        .map_err(|_| "指定した保存先を利用できません。ドライブの接続・同期状態を確認してください。標準の保存先へは自動で切り替えません".to_string())?;
    if !directory.is_dir()
        || ![
            "calendar-data.json",
            "calendar-data.backup.json",
            "calendar-data.json.tmp",
        ]
        .iter()
        .any(|name| directory.join(name).exists())
    {
        return Err("指定した保存先に予定データが見つかりません。空の予定の作成を停止しました。同期状態を確認してください".into());
    }
    Ok(directory)
}

fn paths_for(
    profile: &RuntimeProfile,
    local_app_data: Option<&Path>,
) -> Result<(PathBuf, PathBuf), String> {
    let config = if profile.startup_backend == StartupBackend::Packaged {
        profile
            .data_dir
            .parent()
            .ok_or("標準保存先を確認できません")?
            .join("storage-location.json")
    } else {
        // Portable overrides are PC-local; an absolute path must not travel
        // inside a shared portable folder to a PC with a different drive letter.
        let mut hash = 0xcbf29ce484222325u64;
        for byte in profile.data_dir.to_string_lossy().to_lowercase().bytes() {
            hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
        }
        local_app_data
            .ok_or("PC側の設定保存先を確認できません")?
            .join("Koyomado/storage-locations")
            .join(format!("{hash:016x}.json"))
    };
    Ok((profile.data_dir.clone(), config))
}

pub(super) fn current_paths() -> Result<(PathBuf, PathBuf), String> {
    let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    paths_for(&current_runtime_profile()?, local.as_deref())
}

pub(super) fn current_directory() -> Result<PathBuf, String> {
    let (default, config) = current_paths()?;
    resolve_directory(&default, &config)
}

pub(super) fn with_current<T>(
    operation: impl FnOnce(&Path) -> Result<T, String>,
) -> Result<T, String> {
    let (default, config) = current_paths()?;
    let _location_guard = CalendarFileLock::acquire(&config)?;
    let directory = resolve_directory(&default, &config)?;
    let _data_guard = CalendarFileLock::acquire(&directory.join("calendar-data.json"))?;
    let directory =
        fs::canonicalize(directory).map_err(|e| format!("保存先を確認できません: {e}"))?;
    operation(&directory)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CalendarSnapshot {
    pub data: AppData,
    pub directory: String,
}

pub(super) fn snapshot(directory: &Path) -> Result<CalendarSnapshot, String> {
    Ok(CalendarSnapshot {
        data: load_app_data_from_path(&directory.join("calendar-data.json"))?,
        directory: directory.display().to_string(),
    })
}

type Signature = Vec<(String, Vec<u8>)>;
fn signature(directory: &Path) -> Result<Signature, String> {
    [
        "calendar-data.json",
        "calendar-data.backup.json",
        "calendar-data.json.tmp",
    ]
    .iter()
    .filter(|name| directory.join(name).exists())
    .map(|name| {
        Ok((
            name.to_string(),
            fs::read(directory.join(name)).map_err(|_| "予定データを読み込めません".to_string())?,
        ))
    })
    .collect()
}

fn preview_data(bytes: &Signature) -> Result<AppData, String> {
    for (_, content) in bytes {
        if let Ok(data) = serde_json::from_slice::<AppData>(content) {
            if !(1..=CALENDAR_DATA_VERSION).contains(&data.version) {
                return Err("この予定データの保存形式には対応していません".into());
            }
            return Ok(data);
        }
    }
    Err(
        "利用できる予定データが見つかりません。既存ポータブル版はdataフォルダーを指定してください"
            .into(),
    )
}

fn empty_target(directory: &Path) -> Result<bool, String> {
    for entry in fs::read_dir(directory).map_err(|_| "フォルダーの内容を確認できません")?
    {
        if entry.map_err(|e| e.to_string())?.file_name() != "calendar-data.lock" {
            return Ok(false);
        }
    }
    Ok(true)
}

#[derive(Clone)]
struct StoragePlan {
    token: String,
    default: PathBuf,
    config: PathBuf,
    source: PathBuf,
    source_signature: Option<Signature>,
    target: PathBuf,
    target_signature: Signature,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct StoragePreview {
    token: String,
    source_directory: String,
    target_directory: String,
    source_event_count: Option<usize>,
    target_event_count: Option<usize>,
}

impl StoragePlan {
    fn preview(&self) -> StoragePreview {
        StoragePreview {
            token: self.token.clone(),
            source_directory: self.source.display().to_string(),
            target_directory: self.target.display().to_string(),
            source_event_count: self
                .source_signature
                .as_ref()
                .and_then(|bytes| preview_data(bytes).ok())
                .map(|data| data.events.len()),
            target_event_count: preview_data(&self.target_signature)
                .ok()
                .map(|data| data.events.len()),
        }
    }
}

fn build_plan(default: &Path, config: &Path, target: &Path) -> Result<StoragePlan, String> {
    if !target.is_absolute() {
        return Err("保存先はドライブ名から始まる絶対パスで指定してください".into());
    }
    let _config_guard = CalendarFileLock::acquire(config)?;
    let target = fs::canonicalize(target).map_err(|_| "指定したフォルダーが見つかりません。先に空のフォルダーを作成するか、既存のdataフォルダーを指定してください")?;
    if !target.is_dir() {
        return Err("フォルダーを指定してください".into());
    }
    reject_pending_copy(&target)?;
    let source = location_directory(default, config)?;
    let source = fs::canonicalize(&source).unwrap_or(source);
    if target == source {
        return Err("現在の保存先と同じフォルダーです".into());
    }
    if target.starts_with(&source) || source.starts_with(&target) {
        return Err("現在の保存先の親・子フォルダーは指定できません".into());
    }
    let source_guard = if source.is_dir() {
        CalendarFileLock::acquire(&source.join("calendar-data.json")).ok()
    } else {
        None
    };
    let source_signature = if source_guard.is_some() && reject_pending_copy(&source).is_ok() {
        signature(&source)
            .ok()
            .filter(|bytes| preview_data(bytes).is_ok())
    } else {
        None
    };
    let target_signature = signature(&target)?;
    if !empty_target(&target)? {
        preview_data(&target_signature)?;
    }
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    Ok(StoragePlan {
        token: format!("{}-{stamp}", std::process::id()),
        default: default.to_path_buf(),
        config: config.to_path_buf(),
        source,
        source_signature,
        target,
        target_signature,
    })
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) enum SwitchMode {
    CopyCurrent,
    UseExisting,
}

fn collect_files(root: &Path, relative: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in
        fs::read_dir(root.join(relative)).map_err(|e| format!("移行元を確認できません: {e}"))?
    {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name();
        let name_text = name.to_string_lossy();
        if relative.as_os_str().is_empty()
            && !(matches!(
                name_text.as_ref(),
                "calendar-data.json"
                    | "calendar-data.backup.json"
                    | "calendar-data.json.tmp"
                    | "calendar-history"
                    | "notification-sounds"
                    | "google-restore-state.json"
            ) || (name_text.starts_with("calendar-data.v")
                && name_text.ends_with(".backup.json"))
                || (name_text.starts_with("calendar-data.corrupt-")
                    && name_text.ends_with(".json")))
        {
            continue;
        }
        if name_text == "calendar-data.lock" {
            continue;
        }
        let kind = entry.file_type().map_err(|e| e.to_string())?;
        if kind.is_symlink() {
            return Err(
                "移行元にリンクが含まれています。リンク先のデータを個別に確認してください".into(),
            );
        }
        let path = relative.join(name);
        if kind.is_dir() {
            collect_files(root, &path, files)?;
        } else if kind.is_file() {
            files.push(path);
        } else {
            return Err("移行できない種類のファイルがあります".into());
        }
    }
    files.sort();
    Ok(())
}

fn copy_verified(source: &Path, target: &Path) -> Result<(), String> {
    let mut files = Vec::new();
    collect_files(source, Path::new(""), &mut files)?;
    for relative in &files {
        let destination = target.join(relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let bytes = fs::read(source.join(relative)).map_err(|e| e.to_string())?;
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&destination)
            .map_err(|_| {
                "移行先にファイルが追加されたか、書込みできません。上書きせず停止しました"
                    .to_string()
            })?;
        file.write_all(&bytes)
            .and_then(|_| file.sync_all())
            .map_err(|e| format!("コピーを完了できません: {e}"))?;
        drop(file);
        if fs::read(&destination).map_err(|e| e.to_string())? != bytes {
            return Err("コピー先の照合に失敗しました".into());
        }
    }
    let mut after = Vec::new();
    collect_files(source, Path::new(""), &mut after)?;
    if after != files {
        return Err("コピー中に移行元が変更されました。保存先を切り替えず停止しました".into());
    }
    for relative in &files {
        if fs::read(source.join(relative)).map_err(|e| e.to_string())?
            != fs::read(target.join(relative)).map_err(|e| e.to_string())?
        {
            return Err("コピー中に移行元が変更されました。保存先を切り替えず停止しました".into());
        }
    }
    Ok(())
}

fn apply_plan(plan: &StoragePlan, mode: SwitchMode) -> Result<CalendarSnapshot, String> {
    let _config_guard = CalendarFileLock::acquire(&plan.config)?;
    let source = location_directory(&plan.default, &plan.config)?;
    let source = fs::canonicalize(&source).unwrap_or(source);
    if source != plan.source || fs::canonicalize(&plan.target).ok().as_ref() != Some(&plan.target) {
        return Err("確認後に保存先が変わりました。もう一度フォルダーを確認してください".into());
    }
    // A stable order prevents deadlock when two app instances switch in
    // opposite directions. Never create a disconnected source directory.
    let mut paths = vec![plan.target.join("calendar-data.json")];
    if plan.source_signature.is_some() {
        paths.push(source.join("calendar-data.json"));
    }
    paths.sort();
    paths.dedup();
    let _guards = paths
        .iter()
        .map(|path| CalendarFileLock::acquire(path))
        .collect::<Result<Vec<_>, _>>()?;
    let source_signature = if plan.source_signature.is_some() {
        Some(signature(&source)?)
    } else {
        None
    };
    if plan.source_signature.is_some() && source_signature != plan.source_signature {
        return Err("確認後に元の予定が更新されました。元データを変更せず停止しました。もう一度確認してください".into());
    }
    if signature(&plan.target)? != plan.target_signature {
        return Err("確認後に選択先の予定が更新されました。上書きせず停止しました".into());
    }
    match mode {
        SwitchMode::CopyCurrent => {
            if plan.source_signature.is_none() {
                return Err("元の予定データを利用できないためコピーできません".into());
            }
            if !empty_target(&plan.target)? {
                return Err(
                    "コピー先は空のフォルダーを指定してください。既存データは上書きしません".into(),
                );
            }
            let mut marker = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(plan.target.join(PENDING_COPY))
                .map_err(|e| format!("コピー開始を記録できません: {e}"))?;
            marker
                .write_all(b"Koyomado verified migration v1")
                .and_then(|_| marker.sync_all())
                .map_err(|e| e.to_string())?;
            drop(marker);
            copy_verified(&source, &plan.target)?;
        }
        SwitchMode::UseExisting => {
            preview_data(&plan.target_signature)?;
        }
    }
    // Load/migrate successfully before committing the PC-local pointer.
    let result = snapshot(&plan.target)?;
    if matches!(mode, SwitchMode::CopyCurrent) {
        if signature(&source)?
            != plan
                .source_signature
                .clone()
                .ok_or("元のデータを確認できません")?
        {
            return Err("コピー中に元の予定が更新されました。保存先の切替を停止しました".into());
        }
        fs::rename(
            plan.target.join(PENDING_COPY),
            plan.target
                .join(format!(".koyomado-migration-complete-{}", plan.token)),
        )
        .map_err(|e| format!("コピー完了を記録できません: {e}"))?;
    }
    let location = StorageLocation {
        version: 1,
        data_directory: plan.target.clone(),
    };
    let valid_primary = fs::read(&plan.config)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<StorageLocation>(&bytes).ok())
        .is_some();
    write_json_file(&plan.config, &location, valid_primary)?;
    Ok(result)
}

static PENDING_PLAN: Mutex<Option<StoragePlan>> = Mutex::new(None);

pub(super) fn empty_restore_target(
    default: &Path,
    config: &Path,
    target: &Path,
) -> Result<(PathBuf, PathBuf), String> {
    let plan = build_plan(default, config, target)?;
    if !empty_target(&plan.target)? {
        return Err("復元先には空のフォルダーを指定してください".into());
    }
    Ok((plan.target, plan.source))
}
pub(super) fn activate_restored(
    default: &Path,
    config: &Path,
    target: &Path,
    source: &Path,
    expected: &AppData,
) -> Result<CalendarSnapshot, String> {
    let plan = build_plan(default, config, target)?;
    if plan.source != source {
        return Err(
            "復元中に保存先が変わりました。復元したデータは残し、切替を停止しました".into(),
        );
    }
    if serde_json::to_value(preview_data(&plan.target_signature)?).map_err(|e| e.to_string())?
        != serde_json::to_value(expected).map_err(|e| e.to_string())?
    {
        return Err(
            "復元直後に保存先の内容が変更されました。データを保全し、切替を停止しました".into(),
        );
    }
    apply_plan(&plan, SwitchMode::UseExisting)
}

#[cfg(windows)]
fn pick_path(owner: isize, mode: u8) -> Result<Option<String>, String> {
    use std::os::windows::ffi::OsStringExt;
    use windows::core::Interface;
    use windows::Win32::{
        Foundation::HWND,
        System::Com::{
            CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_INPROC_SERVER,
            COINIT_APARTMENTTHREADED,
        },
        UI::Shell::{
            Common::COMDLG_FILTERSPEC, FileOpenDialog, FileSaveDialog, IFileDialog,
            IFileOpenDialog, IFileSaveDialog, FOS_DONTADDTORECENT, FOS_FILEMUSTEXIST,
            FOS_FORCEFILESYSTEM, FOS_PATHMUSTEXIST, FOS_PICKFOLDERS, SIGDN_FILESYSPATH,
        },
    };
    struct Apartment;
    impl Drop for Apartment {
        fn drop(&mut self) {
            unsafe { CoUninitialize() };
        }
    }
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED)
            .ok()
            .map_err(|e| e.to_string())?;
        let _apartment = Apartment;
        let dialog: IFileDialog = if mode == 2 {
            let save: IFileSaveDialog =
                CoCreateInstance(&FileSaveDialog, None, CLSCTX_INPROC_SERVER)
                    .map_err(|e| e.to_string())?;
            save.cast().map_err(|e| e.to_string())?
        } else {
            let open: IFileOpenDialog =
                CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)
                    .map_err(|e| e.to_string())?;
            open.cast().map_err(|e| e.to_string())?
        };
        let options = dialog.GetOptions().map_err(|e| e.to_string())?
            | FOS_FORCEFILESYSTEM
            | FOS_PATHMUSTEXIST
            | FOS_DONTADDTORECENT;
        dialog
            .SetOptions(if mode == 0 {
                options | FOS_PICKFOLDERS
            } else if mode == 1 {
                options | FOS_FILEMUSTEXIST
            } else {
                options
            })
            .map_err(|e| e.to_string())?;
        if mode == 0 {
            dialog
                .SetTitle(windows::core::w!(
                    "フォルダーを選択（予定データの移行・復元先は専用の空フォルダー）"
                ))
                .map_err(|e| e.to_string())?;
        } else {
            dialog
                .SetFileTypes(&[COMDLG_FILTERSPEC {
                    pszName: windows::core::w!("Koyomadoバックアップ (*.zip)"),
                    pszSpec: windows::core::w!("*.zip"),
                }])
                .map_err(|e| e.to_string())?;
            dialog
                .SetDefaultExtension(windows::core::w!("zip"))
                .map_err(|e| e.to_string())?;
            if mode == 2 {
                dialog
                    .SetFileName(windows::core::w!("koyomado-backup.zip"))
                    .map_err(|e| e.to_string())?;
            }
        }
        if let Err(error) = dialog.Show(Some(HWND(owner as *mut std::ffi::c_void))) {
            if error.code().0 as u32 == 0x800704c7 {
                return Ok(None);
            }
            return Err(error.to_string());
        }
        let path = dialog
            .GetResult()
            .and_then(|item| item.GetDisplayName(SIGDN_FILESYSPATH))
            .map_err(|e| e.to_string())?;
        let result = std::ffi::OsString::from_wide(path.as_wide())
            .to_string_lossy()
            .into_owned();
        CoTaskMemFree(Some(path.0.cast()));
        Ok(Some(result))
    }
}

#[tauri::command]
pub(super) async fn choose_storage_directory(
    window: tauri::WebviewWindow,
) -> Result<Option<String>, String> {
    #[cfg(windows)]
    {
        let owner = window.hwnd().map_err(|e| e.to_string())?.0 as isize;
        tauri::async_runtime::spawn_blocking(move || {
            std::thread::spawn(move || pick_path(owner, 0))
                .join()
                .map_err(|_| "フォルダー選択を終了できませんでした".to_string())?
        })
        .await
        .map_err(|e| e.to_string())?
    }
    #[cfg(not(windows))]
    {
        let _ = window;
        Err("Windowsで利用してください".into())
    }
}

#[tauri::command]
pub(super) async fn choose_backup_file(
    window: tauri::WebviewWindow,
    save: bool,
) -> Result<Option<String>, String> {
    #[cfg(windows)]
    {
        let owner = window.hwnd().map_err(|e| e.to_string())?.0 as isize;
        tauri::async_runtime::spawn_blocking(move || {
            std::thread::spawn(move || pick_path(owner, if save { 2 } else { 1 }))
                .join()
                .map_err(|_| "ZIP選択を完了できません")?
        })
        .await
        .map_err(|e| e.to_string())?
    }
    #[cfg(not(windows))]
    {
        let _ = (window, save);
        Err("Windowsで利用してください".into())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct StorageInfo {
    directory: String,
    default_directory: String,
    available: bool,
}

#[tauri::command]
pub(super) fn get_storage_info() -> Result<StorageInfo, String> {
    let (default, config) = current_paths()?;
    let directory = location_directory(&default, &config)?;
    Ok(StorageInfo {
        directory: directory.display().to_string(),
        default_directory: default.display().to_string(),
        available: resolve_directory(&default, &config).is_ok(),
    })
}

#[tauri::command]
pub(super) async fn preview_storage_directory(directory: String) -> Result<StoragePreview, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = APP_DATA_LOCK
            .lock()
            .map_err(|_| "保存先の確認を開始できません")?;
        let (default, config) = current_paths()?;
        let plan = build_plan(&default, &config, Path::new(&directory))?;
        let preview = plan.preview();
        *PENDING_PLAN
            .lock()
            .map_err(|_| "確認内容を保持できません")? = Some(plan);
        Ok(preview)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub(super) async fn apply_storage_directory(
    token: String,
    mode: SwitchMode,
) -> Result<CalendarSnapshot, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = APP_DATA_LOCK
            .lock()
            .map_err(|_| "保存先の切替を開始できません")?;
        let plan = PENDING_PLAN
            .lock()
            .map_err(|_| "確認内容を参照できません")?
            .take()
            .ok_or("もう一度保存先を確認してください")?;
        if plan.token != token {
            return Err("確認内容が変わりました。もう一度保存先を確認してください".into());
        }
        apply_plan(&plan, mode)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    fn area(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../output/native-smoke-storage-20261017/tests")
            .join(format!("{name}-{}-{stamp}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        path
    }
    fn seed(directory: &Path) {
        fs::create_dir_all(directory).unwrap();
        fs::write(
            directory.join("calendar-data.json"),
            serde_json::to_vec(&AppData::default()).unwrap(),
        )
        .unwrap();
    }
    fn point(config: &Path, directory: &Path) {
        fs::write(
            config,
            serde_json::to_vec(&StorageLocation {
                version: 1,
                data_directory: directory.to_path_buf(),
            })
            .unwrap(),
        )
        .unwrap();
    }
    #[test]
    fn chosen_folder_overrides_store_default() {
        let root = area("override");
        let default = root.join("LocalState/Koyomado/data");
        let target = root.join("Drive/予定");
        seed(&target);
        let config = root.join("storage-location.json");
        point(&config, &target);
        assert_eq!(
            resolve_directory(&default, &config).unwrap(),
            fs::canonicalize(&target).unwrap()
        );
    }
    #[test]
    fn unavailable_folder_does_not_fall_back_or_create_empty_data() {
        let root = area("unavailable");
        let default = root.join("local");
        seed(&default);
        let target = root.join("missing-drive");
        let config = root.join("storage-location.json");
        point(&config, &target);
        assert!(resolve_directory(&default, &config).is_err());
        assert!(!target.exists());
    }
    #[test]
    fn missing_files_in_a_selected_folder_do_not_initialize_empty_data() {
        let root = area("not-hydrated");
        let target = root.join("Drive");
        fs::create_dir(&target).unwrap();
        let config = root.join("storage-location.json");
        point(&config, &target);
        assert!(resolve_directory(&root.join("local"), &config).is_err());
        assert!(!target.join("calendar-data.json").exists());
    }
    #[test]
    fn copy_switch_keeps_source_and_carries_backups_history_and_sound() {
        let root = area("copy");
        let source = root.join("LocalState/Koyomado/data");
        seed(&source);
        fs::write(source.join("calendar-data.backup.json"), b"backup-evidence").unwrap();
        fs::create_dir(source.join("calendar-history")).unwrap();
        fs::write(
            source.join("calendar-history/old.json"),
            b"history-evidence",
        )
        .unwrap();
        fs::create_dir(source.join("notification-sounds")).unwrap();
        fs::write(
            source.join("notification-sounds/custom.wav"),
            b"sound-evidence",
        )
        .unwrap();
        fs::write(source.join("window-state.json"), b"local-window").unwrap();
        let target = root.join("Drive/予定");
        fs::create_dir_all(&target).unwrap();
        let config = root.join("storage-location.json");
        let plan = build_plan(&source, &config, &target).unwrap();
        let result = apply_plan(&plan, SwitchMode::CopyCurrent);
        assert!(result.is_ok());
        assert!(source.join("calendar-data.json").exists());
        assert_eq!(
            fs::read(target.join("calendar-data.json")).unwrap(),
            fs::read(source.join("calendar-data.json")).unwrap()
        );
        assert_eq!(
            fs::read(target.join("calendar-history/old.json")).unwrap(),
            b"history-evidence"
        );
        assert_eq!(
            fs::read(target.join("notification-sounds/custom.wav")).unwrap(),
            b"sound-evidence"
        );
        assert!(!target.join("window-state.json").exists());
        assert_eq!(
            resolve_directory(&source, &config).unwrap(),
            fs::canonicalize(target).unwrap()
        );
    }
    #[test]
    fn existing_data_is_used_without_overwriting_either_calendar() {
        let root = area("existing");
        let source = root.join("local");
        seed(&source);
        let target = root.join("Drive/data");
        seed(&target);
        let original = fs::read(target.join("calendar-data.json")).unwrap();
        let config = root.join("storage-location.json");
        let plan = build_plan(&source, &config, &target).unwrap();
        assert!(apply_plan(&plan, SwitchMode::UseExisting).is_ok());
        assert_eq!(
            fs::read(target.join("calendar-data.json")).unwrap(),
            original
        );
        assert!(source.join("calendar-data.json").exists());
    }
    #[test]
    fn source_change_after_confirmation_stops_switch() {
        let root = area("stale-source");
        let source = root.join("local");
        seed(&source);
        let target = root.join("Drive");
        fs::create_dir(&target).unwrap();
        let config = root.join("storage-location.json");
        let plan = build_plan(&source, &config, &target).unwrap();
        let mut data = AppData::default();
        data.settings.sidebar_collapsed = true;
        fs::write(
            source.join("calendar-data.json"),
            serde_json::to_vec(&data).unwrap(),
        )
        .unwrap();
        assert!(apply_plan(&plan, SwitchMode::CopyCurrent).is_err());
        assert!(!config.exists());
        assert!(!target.join("calendar-data.json").exists());
    }
    #[test]
    fn target_change_after_confirmation_stops_switch_without_overwrite() {
        let root = area("stale-target");
        let source = root.join("local");
        seed(&source);
        let target = root.join("Drive");
        fs::create_dir(&target).unwrap();
        let config = root.join("storage-location.json");
        let plan = build_plan(&source, &config, &target).unwrap();
        seed(&target);
        let bytes = fs::read(target.join("calendar-data.json")).unwrap();
        assert!(apply_plan(&plan, SwitchMode::CopyCurrent).is_err());
        assert!(!config.exists());
        assert_eq!(fs::read(target.join("calendar-data.json")).unwrap(), bytes);
    }
    #[test]
    fn copying_over_existing_data_is_refused() {
        let root = area("no-overwrite");
        let source = root.join("local");
        seed(&source);
        let target = root.join("Drive");
        seed(&target);
        let config = root.join("storage-location.json");
        let plan = build_plan(&source, &config, &target).unwrap();
        assert!(apply_plan(&plan, SwitchMode::CopyCurrent).is_err());
        assert!(!config.exists());
    }
    #[test]
    fn externally_changed_restore_is_not_activated() {
        let root = area("restore-changed");
        let source = root.join("source");
        seed(&source);
        let target = root.join("restored");
        seed(&target);
        let config = root.join("location.json");
        let expected = AppData::default();
        let mut changed = expected.clone();
        changed.settings.theme = "forest-breath".into();
        fs::write(
            target.join("calendar-data.json"),
            serde_json::to_vec(&changed).unwrap(),
        )
        .unwrap();
        assert!(activate_restored(
            &source,
            &config,
            &target,
            &fs::canonicalize(&source).unwrap(),
            &expected
        )
        .is_err());
        assert!(!config.exists());
    }
    #[test]
    fn incomplete_copy_is_not_offered_as_an_existing_calendar() {
        let root = area("partial-copy");
        let source = root.join("local");
        seed(&source);
        let target = root.join("Drive");
        seed(&target);
        let config = root.join("storage-location.json");
        fs::write(target.join(".koyomado-migration-pending"), b"pending").unwrap();
        assert!(build_plan(&source, &config, &target).is_err());
        point(&config, &target);
        assert!(resolve_directory(&source, &config).is_err());
    }
    #[test]
    fn unavailable_source_can_switch_to_existing_data_without_recreating_source() {
        let root = area("recover-location");
        let source = root.join("disconnected");
        let target = root.join("available");
        seed(&target);
        let config = root.join("storage-location.json");
        point(&config, &source);
        let plan = build_plan(&root.join("default"), &config, &target).unwrap();
        assert!(plan.source_signature.is_none());
        assert!(apply_plan(&plan, SwitchMode::UseExisting).is_ok());
        assert!(!source.exists());
    }
    #[test]
    fn returning_to_default_uses_its_existing_data_and_keeps_external_folder() {
        let root = area("return-default");
        let default = root.join("local");
        seed(&default);
        let external = root.join("Drive");
        seed(&external);
        let config = root.join("storage-location.json");
        point(&config, &external);
        let plan = build_plan(&default, &config, &default).unwrap();
        assert!(apply_plan(&plan, SwitchMode::UseExisting).is_ok());
        assert_eq!(
            resolve_directory(&default, &config).unwrap(),
            fs::canonicalize(&default).unwrap()
        );
        assert!(external.join("calendar-data.json").exists());
    }
    #[test]
    fn unsupported_or_unrelated_target_is_not_modified() {
        let root = area("invalid-target");
        let source = root.join("local");
        seed(&source);
        let target = root.join("Drive");
        seed(&target);
        let config = root.join("storage-location.json");
        let data = AppData {
            version: 999,
            ..AppData::default()
        };
        let bytes = serde_json::to_vec(&data).unwrap();
        fs::write(target.join("calendar-data.json"), &bytes).unwrap();
        assert!(build_plan(&source, &config, &target).is_err());
        assert_eq!(fs::read(target.join("calendar-data.json")).unwrap(), bytes);
        assert!(!config.exists());
    }
    #[test]
    fn relative_same_and_nested_targets_are_rejected() {
        let root = area("paths");
        let source = root.join("local");
        seed(&source);
        let config = root.join("storage-location.json");
        fs::create_dir(source.join("child")).unwrap();
        assert!(build_plan(&source, &config, Path::new("relative")).is_err());
        assert!(build_plan(&source, &config, &source).is_err());
        assert!(build_plan(&source, &config, &source.join("child")).is_err());
    }
    #[test]
    fn uncommitted_location_temp_file_is_not_activated() {
        let root = area("uncommitted-pointer");
        let default = root.join("local");
        seed(&default);
        let target = root.join("Drive");
        seed(&target);
        let config = root.join("storage-location.json");
        point(&config.with_extension("json.tmp"), &target);
        assert_eq!(resolve_directory(&default, &config).unwrap(), default);
    }
    #[test]
    fn packaged_location_setting_stays_under_localstate() {
        let root = area("package-profile");
        let local = root.join("LocalState");
        let profile =
            runtime_profile(true, Path::new("C:/WindowsApps/Koyomado.exe"), Some(&local)).unwrap();
        let (default, config) = paths_for(&profile, None).unwrap();
        assert_eq!(default, local.join("Koyomado/data"));
        assert_eq!(config, local.join("Koyomado/storage-location.json"));
    }
    #[cfg(windows)]
    #[test]
    fn failed_pointer_commit_keeps_the_previous_active_folder() {
        use std::os::windows::fs::OpenOptionsExt;
        let root = area("pointer-failure");
        let source = root.join("local");
        seed(&source);
        let target = root.join("Drive");
        fs::create_dir(&target).unwrap();
        let config = root.join("storage-location.json");
        point(&config, &source);
        let plan = build_plan(&source, &config, &target).unwrap();
        let _held = OpenOptions::new()
            .read(true)
            .share_mode(3)
            .open(&config)
            .unwrap();
        assert!(apply_plan(&plan, SwitchMode::CopyCurrent).is_err());
        assert_eq!(resolve_directory(&source, &config).unwrap(), source);
        assert!(source.join("calendar-data.json").exists());
    }
    #[cfg(windows)]
    #[test]
    fn failed_file_copy_is_marked_incomplete_and_cannot_be_activated() {
        use std::os::windows::fs::OpenOptionsExt;
        let root = area("copy-io-failure");
        let source = root.join("local");
        seed(&source);
        fs::create_dir(source.join("notification-sounds")).unwrap();
        let sound = source.join("notification-sounds/custom.wav");
        fs::write(&sound, b"synthetic").unwrap();
        let target = root.join("Drive");
        fs::create_dir(&target).unwrap();
        let config = root.join("storage-location.json");
        let plan = build_plan(&source, &config, &target).unwrap();
        let _held = OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(sound)
            .unwrap();
        assert!(apply_plan(&plan, SwitchMode::CopyCurrent).is_err());
        assert!(target.join(PENDING_COPY).exists());
        assert!(!config.exists());
        assert!(build_plan(&source, &config, &target).is_err());
        assert!(source.join("calendar-data.json").exists());
    }
}
