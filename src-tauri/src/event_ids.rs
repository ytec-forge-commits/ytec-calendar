use super::*;
use std::collections::HashMap;

// Keep every existing record. Reserve all original IDs before allocating any
// replacement, so a later record cannot lose its ID to an earlier repair.
pub(super) fn unused_id(preferred: &str, occupied: &mut HashSet<String>) -> String {
    if occupied.insert(preferred.to_string()) {
        return preferred.to_string();
    }
    for suffix in occupied.len() as u64.. {
        let candidate = format!("{preferred}-recovered-{suffix}");
        if occupied.insert(candidate.clone()) {
            return candidate;
        }
    }
    unreachable!()
}

pub(super) fn repair_duplicates(data: &mut AppData) -> bool {
    let mut occupied: HashSet<_> = data.events.iter().map(|e| e.id.clone()).collect();
    occupied.extend(data.deleted_events.iter().map(|e| e.event.id.clone()));
    occupied.extend(
        data.events
            .iter()
            .filter_map(|e| e.recurrence_exception.as_ref().map(|r| r.master_id.clone())),
    );
    let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
    for (index, event) in data.events.iter().enumerate() {
        groups.entry(event.id.clone()).or_default().push(index);
    }
    let mut changed = false;
    // Vector order defines the existing first-match behavior for local
    // recurrence references. Keep that first ID; only rename later copies.
    for index in 0..data.events.len() {
        let id = data.events[index].id.clone();
        if !id.is_empty() && groups[&id][0] != index {
            data.events[index].id = unused_id(&id, &mut occupied);
            changed = true;
        }
    }
    if !changed {
        return false;
    }
    for index in 0..data.events.len() {
        let Some(exception) = &data.events[index].recurrence_exception else {
            continue;
        };
        let Some(parents) = groups.get(&exception.master_id).filter(|g| g.len() > 1) else {
            continue;
        };
        // Google links can disambiguate the parent of a linked exception.
        // Without that evidence, preserve the old first-match relationship.
        let candidates: Vec<_> = parents
            .iter()
            .filter(|&&parent| {
                data.events[parent].recurrence.is_some()
                    && data.events[index].google_links.iter().any(|child| {
                        child
                            .recurring_event_id
                            .as_ref()
                            .is_some_and(|remote_parent| {
                                data.events[parent].google_links.iter().any(|link| {
                                    link.account_id == child.account_id
                                        && link.calendar_id == child.calendar_id
                                        && &link.event_id == remote_parent
                                })
                            })
                    })
            })
            .copied()
            .collect();
        if let [parent] = candidates.as_slice() {
            let master_id = data.events[*parent].id.clone();
            data.events[index]
                .recurrence_exception
                .as_mut()
                .unwrap()
                .master_id = master_id;
        }
    }
    true
}

pub(super) fn preserve_original(path: &Path, expected: &[u8]) -> Result<(), String> {
    use sha2::{Digest, Sha256};
    let bytes = fs::read(path).map_err(|_| "ID修復前の元データを読み込めません")?;
    if bytes != expected {
        return Err(
            "DATA_CHANGED: ID修復中に予定が変更されました。元データを保護して停止しました".into(),
        );
    }
    let hash = format!("{:x}", Sha256::digest(&bytes));
    let backup = path.with_file_name(format!("calendar-data.before-id-repair.{hash}.json"));
    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&backup)
    {
        Ok(mut file) => file
            .write_all(&bytes)
            .and_then(|_| file.sync_all())
            .map_err(|_| "ID修復前の退避を完了できません。元データは変更していません")?,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(_) => return Err("ID修復前の退避を作成できません。元データは変更していません".into()),
    }
    if fs::read(&backup).map_err(|_| "ID修復前の退避を照合できません")? != bytes
        || fs::read(path).map_err(|_| "ID修復前の元データを再確認できません")? != bytes
    {
        return Err("ID修復前の退避が一致しません。元データは変更していません".into());
    }
    Ok(())
}
