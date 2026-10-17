use super::*;

fn test_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../output/native-smoke-fixes-20261017/test-data")
        .join(format!("{name}-{}-{unique}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    dir.join("calendar-data.json")
}

fn event(id: &str) -> CalendarEvent {
    serde_json::from_value(serde_json::json!({
        "id": id, "title": id, "date":"2026-10-17", "endDate":"2026-10-17",
        "allDay":true, "startTime":"", "endTime":"", "location":"", "notes":"",
        "style":{"color":"#78a88f"}, "createdAt":"2026-10-17T00:00:00Z", "updatedAt":"2026-10-17T00:00:00Z"
    })).unwrap()
}

#[test]
fn missing_primary_recovers_committed_backup() {
    let path = test_path("missing-primary");
    let mut data = AppData::default();
    data.events.push(event("protected"));
    fs::write(backup_path(&path), serde_json::to_vec(&data).unwrap()).unwrap();
    let recovered = load_app_data_from_path(&path).unwrap();
    assert_eq!(recovered.events.len(), 1);
    assert_eq!(recovered.events[0].id, "protected");
}

#[test]
fn interrupted_first_save_recovers_complete_temp_file() {
    let path = test_path("temp-only");
    let mut data = AppData::default();
    data.events.push(event("first-save"));
    fs::write(
        path.with_extension("json.tmp"),
        serde_json::to_vec(&data).unwrap(),
    )
    .unwrap();
    assert_eq!(load_app_data_from_path(&path).unwrap().events.len(), 1);
}

#[test]
fn invalid_recovery_candidate_does_not_initialize_an_empty_calendar() {
    let path = test_path("invalid-backup");
    fs::write(backup_path(&path), b"broken").unwrap();
    assert!(load_app_data_from_path(&path).is_err());
    assert!(!path.exists());
}

#[test]
fn recovery_keeps_the_uncommitted_temp_candidate_as_history() {
    let path = test_path("backup-and-temp");
    let mut committed = AppData::default();
    committed.events.push(event("committed"));
    let mut pending = committed.clone();
    pending.events.push(event("pending"));
    fs::write(backup_path(&path), serde_json::to_vec(&committed).unwrap()).unwrap();
    fs::write(
        path.with_extension("json.tmp"),
        serde_json::to_vec(&pending).unwrap(),
    )
    .unwrap();
    assert_eq!(load_app_data_from_path(&path).unwrap().events.len(), 1);
    let history: Vec<AppData> = fs::read_dir(path.with_file_name("calendar-history"))
        .unwrap()
        .map(|entry| serde_json::from_slice(&fs::read(entry.unwrap().path()).unwrap()).unwrap())
        .collect();
    assert!(history
        .iter()
        .any(|data| data.events.iter().any(|e| e.id == "pending")));
}

#[test]
fn stale_native_save_is_rejected_and_primary_is_unchanged() {
    let path = test_path("native-cas");
    let base = AppData::default();
    write_json_with_backup(&path, &base).unwrap();
    let mut office = base.clone();
    office.events.push(event("office"));
    save_app_data_checked(&path, office, &base).unwrap();
    let mut home = base.clone();
    home.events.push(event("home"));
    assert!(save_app_data_checked(&path, home, &base)
        .unwrap_err()
        .starts_with("DATA_CHANGED:"));
    let saved = load_app_data_from_path(&path).unwrap();
    assert_eq!(saved.events.len(), 1);
    assert_eq!(saved.events[0].id, "office");
    let history: Vec<AppData> = fs::read_dir(path.with_file_name("calendar-history"))
        .unwrap()
        .map(|entry| serde_json::from_slice(&fs::read(entry.unwrap().path()).unwrap()).unwrap())
        .collect();
    assert!(history
        .iter()
        .any(|data| data.events.iter().any(|e| e.id == "home")));
    assert!(history
        .iter()
        .any(|data| data.events.iter().any(|e| e.id == "office")));
}

#[test]
fn recovering_corrupt_primary_does_not_overwrite_good_backup() {
    let path = test_path("preserve-backup");
    let mut data = AppData::default();
    data.events.push(event("backup"));
    let bytes = serde_json::to_vec_pretty(&data).unwrap();
    fs::write(backup_path(&path), &bytes).unwrap();
    fs::write(&path, b"broken").unwrap();
    assert_eq!(load_app_data_from_path(&path).unwrap().events.len(), 1);
    assert_eq!(fs::read(backup_path(&path)).unwrap(), bytes);
}

#[test]
fn non_object_json_primary_can_recover_without_panicking() {
    let path = test_path("non-object-primary");
    let mut data = AppData::default();
    data.events.push(event("protected"));
    fs::write(backup_path(&path), serde_json::to_vec(&data).unwrap()).unwrap();
    fs::write(&path, b"42").unwrap();
    assert_eq!(
        load_app_data_from_path(&path).unwrap().events[0].id,
        "protected"
    );
}

#[test]
fn history_only_is_not_mistaken_for_a_new_installation() {
    let path = test_path("history-only");
    retain_calendar_history(&path, &serde_json::to_vec(&AppData::default()).unwrap()).unwrap();
    assert!(load_app_data_from_path(&path).is_err());
    assert!(!path.exists());
}

#[test]
fn sync_bookkeeping_alone_does_not_grow_history_every_minute() {
    let path = test_path("sync-bookkeeping");
    let mut data = AppData::default();
    data.settings.google.accounts.push(
        serde_json::from_value(serde_json::json!({
            "id":"synthetic-account", "email":"test@example.invalid"
        }))
        .unwrap(),
    );
    write_json_with_backup(&path, &data).unwrap();
    let count = fs::read_dir(path.with_file_name("calendar-history"))
        .unwrap()
        .count();
    data.settings.google.accounts[0].last_sync_at = "2026-10-17T02:00:00Z".into();
    write_json_with_backup(&path, &data).unwrap();
    assert_eq!(
        fs::read_dir(path.with_file_name("calendar-history"))
            .unwrap()
            .count(),
        count
    );
    data.events.push(event("real-content-change"));
    write_json_with_backup(&path, &data).unwrap();
    assert!(
        fs::read_dir(path.with_file_name("calendar-history"))
            .unwrap()
            .count()
            > count
    );
}

#[cfg(windows)]
#[test]
fn failed_windows_replacement_preserves_primary_and_both_history_versions() {
    use std::os::windows::fs::OpenOptionsExt;
    let path = test_path("replacement-failure");
    let mut previous = AppData::default();
    previous.events.push(event("previous"));
    write_json_with_backup(&path, &previous).unwrap();
    let _held = OpenOptions::new()
        .read(true)
        .share_mode(3)
        .open(&path)
        .unwrap();
    let mut next = previous.clone();
    next.events.push(event("next"));
    assert!(write_json_with_backup(&path, &next).is_err());
    assert_eq!(load_app_data_from_path(&path).unwrap().events.len(), 1);
    let history: Vec<AppData> = fs::read_dir(path.with_file_name("calendar-history"))
        .unwrap()
        .map(|entry| serde_json::from_slice(&fs::read(entry.unwrap().path()).unwrap()).unwrap())
        .collect();
    assert!(history.iter().any(|data| data.events.len() == 2));
    assert!(history.iter().any(|data| data.events.len() == 1));
}

#[cfg(windows)]
#[test]
fn file_lock_serializes_two_independent_processes() {
    if let Some(path) = std::env::var_os("KOYOMADO_SYNTHETIC_LOCK_TEST") {
        let path = PathBuf::from(path);
        fs::write(path.with_extension("ready"), b"ready").unwrap();
        let _guard = CalendarFileLock::acquire(&path).unwrap();
        fs::write(path.with_extension("entered"), b"entered").unwrap();
        return;
    }
    let path = test_path("two-process-lock");
    let guard = CalendarFileLock::acquire(&path).unwrap();
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "data_safety_tests::file_lock_serializes_two_independent_processes",
        ])
        .env("KOYOMADO_SYNTHETIC_LOCK_TEST", &path)
        .stdout(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let started = Instant::now();
    while !path.with_extension("ready").exists() && started.elapsed() < Duration::from_secs(5) {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(path.with_extension("ready").exists());
    std::thread::sleep(Duration::from_millis(200));
    assert!(
        !path.with_extension("entered").exists(),
        "second process entered a locked transaction"
    );
    drop(guard);
    assert!(child.wait().unwrap().success());
    assert!(path.with_extension("entered").exists());
}
