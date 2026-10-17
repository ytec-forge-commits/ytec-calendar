# Koyomado changelog

## 1.1.2 - 2026-10-17

- Prevent Google reconnection from importing a retained local event with a duplicate internal ID. Reconcile unchanged copies within the same recurrence family; preserve changed copies with distinct IDs.
- Repair existing duplicate active IDs without removing records or changing their content or Google event IDs. Preserve and verify the exact original file as `calendar-data.before-id-repair.<SHA256>.json` first; stop if preservation fails or the source changes.
- Prioritize the Google account/calendar/event link over old internal-ID hints during post-restore reconciliation, preventing a detached local copy from matching the wrong event.
- Keep calendar format v5, ZIP format v1, empty-ID and unsafe-archive rejection, deletion history and concurrent shared-folder launches.

## 1.1.1 - 2026-10-17

- Update rustls to 0.23.45 for the TLS 1.3 handshake validation fix used by Google synchronization.
- Fix ZIP backup/restore validation rejecting legitimate shared IDs between active events and deletion history, including repeated deletions of the same event.
- Preserve event IDs and every history record; continue rejecting duplicate active IDs, empty IDs and invalid archives.
- Show manual backup, restore and automatic backup results beside their respective controls and scroll them into view.
- Keep calendar format v5, ZIP format v1 and compatibility with existing data and 1.1.0 archives.

## 1.1.0 (2026-10-17)

- Choose a locally accessible calendar data folder in both Store and self-signed portable editions, including folders mounted by Google Drive.
- Copy current data into an empty folder and verify it before switching, or explicitly select existing data. Switching does not require restarting the app.
- Export and restore calendar backup ZIPs containing events, deleted events, settings and custom sounds. The same format supports migration in either direction between editions.
- Configure daily, weekly or monthly automatic backups and retain 1-100 generations. Prune only unchanged ZIPs owned by the same automatic series, after a new ZIP is saved and verified.
- Disable Google integration after restore. After reauthorization, the first sync only downloads and reconciles IDs; it sends no remote creates, updates or deletions.
- Continue supporting concurrent launches against a shared folder, with live change detection, draft-preserving conflict handling, history and atomic file replacement.
- Reject unsafe, damaged or unsupported ZIPs and nonempty restore folders. Preserve the original calendar and archive.
- Retain calendar format v5 and backup format v1. Read calendar formats v1-v4 with pre-migration backups.
- Update the Japanese manual and add an English manual, changelog and privacy policy.

## 1.0.0 (2026-09-02)

First stable version. Added Microsoft Store MSIX distribution with package-local data storage and Windows StartupTask support alongside the self-signed portable edition. Store availability was announced on 2026-09-04.

For the complete earlier release history, see [the Japanese changelog](CHANGELOG.md).
