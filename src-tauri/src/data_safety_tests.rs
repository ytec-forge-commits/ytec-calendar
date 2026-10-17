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
fn duplicate_active_ids_are_recovered_without_discarding_records_or_the_original_file() {
    let path = test_path("duplicate-active-id-recovery");
    let first = event("google-detached");
    let mut edited = first.clone();
    edited.notes = "preserve this distinct local copy".into();
    let mut data = AppData {
        events: vec![first.clone(), first.clone(), edited],
        ..Default::default()
    };
    data.events.push(event("google-detached-recovered-1"));
    let original = serde_json::to_vec(&data).unwrap();
    fs::write(&path, &original).unwrap();
    let recovered = load_app_data_from_path(&path).unwrap();
    assert_eq!(recovered.events.len(), 4);
    assert_eq!(
        recovered
            .events
            .iter()
            .map(|e| &e.id)
            .collect::<HashSet<_>>()
            .len(),
        4
    );
    assert_eq!(
        serde_json::to_value(&recovered.events[0]).unwrap(),
        serde_json::to_value(&first).unwrap()
    );
    for (before, after) in data.events.iter().zip(&recovered.events) {
        let mut restored = after.clone();
        restored.id = before.id.clone();
        assert_eq!(
            serde_json::to_value(&restored).unwrap(),
            serde_json::to_value(before).unwrap(),
            "only local IDs may change"
        );
    }
    let preserved: Vec<_> = fs::read_dir(path.parent().unwrap())
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
    let saved = fs::read(&path).unwrap();
    assert_eq!(
        serde_json::to_value(load_app_data_from_path(&path).unwrap()).unwrap(),
        serde_json::to_value(&recovered).unwrap()
    );
    assert_eq!(
        fs::read(&path).unwrap(),
        saved,
        "reloading must be idempotent"
    );
}

#[test]
fn duplicate_recurring_parent_recovery_uses_google_identity_and_preserves_local_references() {
    let path = test_path("duplicate-parent-recovery");
    let mut detached = event("parent");
    detached.recurrence = Some(default_yearly_recurrence());
    let mut linked = detached.clone();
    linked.google_links = vec![serde_json::from_value(serde_json::json!({
        "accountId":"synthetic-account", "calendarId":"primary", "eventId":"remote-parent",
        "localUpdatedAt": linked.updated_at
    }))
    .unwrap()];
    let mut child = event("linked-child");
    child.recurrence_exception = Some(RecurrenceException {
        master_id: "parent".into(),
        original_date: "2026-10-17".into(),
    });
    child.google_links = vec![serde_json::from_value(serde_json::json!({
        "accountId":"synthetic-account", "calendarId":"primary", "eventId":"remote-child",
        "recurringEventId":"remote-parent", "localUpdatedAt":child.updated_at
    }))
    .unwrap()];
    let mut local_child = event("local-child");
    local_child.recurrence_exception = child.recurrence_exception.clone();
    let data = AppData {
        events: vec![detached, linked, child, local_child],
        ..Default::default()
    };
    fs::write(&path, serde_json::to_vec(&data).unwrap()).unwrap();
    let recovered = load_app_data_from_path(&path).unwrap();
    assert_eq!(recovered.events.len(), 4);
    assert_eq!(
        recovered.events[2]
            .recurrence_exception
            .as_ref()
            .unwrap()
            .master_id,
        recovered.events[1].id
    );
    assert_eq!(
        recovered.events[3]
            .recurrence_exception
            .as_ref()
            .unwrap()
            .master_id,
        "parent"
    );
    assert_eq!(
        serde_json::to_value(&recovered.events[1].google_links).unwrap(),
        serde_json::to_value(&data.events[1].google_links).unwrap()
    );
    assert_eq!(recovered.events[1].updated_at, data.events[1].updated_at);
}

#[test]
fn failed_original_id_backup_blocks_repair_without_touching_the_primary() {
    use sha2::{Digest, Sha256};
    let path = test_path("id-backup-failure");
    let duplicate = event("duplicate");
    let data = AppData {
        events: vec![duplicate.clone(), duplicate],
        ..Default::default()
    };
    let original = serde_json::to_vec(&data).unwrap();
    fs::write(&path, &original).unwrap();
    let hash = format!("{:x}", Sha256::digest(&original));
    let backup = path.with_file_name(format!("calendar-data.before-id-repair.{hash}.json"));
    fs::write(&backup, b"incomplete previous backup").unwrap();
    assert!(load_app_data_from_path(&path).is_err());
    assert_eq!(fs::read(&path).unwrap(), original);
    assert_eq!(fs::read(&backup).unwrap(), b"incomplete previous backup");
}

#[test]
fn id_repair_refuses_a_sync_replacement_of_the_parsed_source() {
    let path = test_path("id-repair-source-replaced");
    let duplicate = event("duplicate");
    let parsed = AppData {
        events: vec![duplicate.clone(), duplicate],
        ..Default::default()
    };
    let expected = serde_json::to_vec(&parsed).unwrap();
    fs::write(&path, &expected).unwrap();
    let replacement = serde_json::to_vec(&AppData {
        events: vec![event("new-remote-record")],
        ..Default::default()
    })
    .unwrap();
    // A Drive client can replace the primary without taking our .lock file.
    fs::write(&path, &replacement).unwrap();
    assert!(
        event_ids::preserve_original(&path, &expected).is_err(),
        "repair must reject a different source snapshot"
    );
    assert_eq!(fs::read(&path).unwrap(), replacement);
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
