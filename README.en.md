# Koyomado

In Settings, choose 「操作説明書を開く」 to open the bundled PDF in Windows' default viewer. No network connection or Google sign-in is required.

**Version 1.1.3 (2026-10-17)** adds this in-app manual button and centers the contact link on the Japanese manual cover.

[日本語](README.md) | English

Koyomado is a simple Windows calendar that can sit on your desktop like a widget. It is distributed as a Microsoft Store app for general use and as a portable folder that can be carried on USB storage or Google Drive.

**Version 1.1.2 (2026-10-17)** fixes duplicate active IDs after reconnecting Google and the resulting ZIP backup failure. Existing duplicates receive distinct internal IDs without deleting events, with the original file preserved first. Selectable folders, ZIP backup/restore, automatic backups and shared-data protection were introduced in 1.1.0. Store and Vector availability follows their review/update process. The application UI is in Japanese; the [English operation manual](docs/Koyomado-Manual.en.pdf) explains the Japanese button labels. A [Japanese manual](docs/Koyomado操作説明書.pdf), [English changelog](CHANGELOG.en.md) and [English privacy policy](PRIVACY.en.md) are also available.

- Official website: https://ytec.cloudfree.jp/forge/en/projects/koyomado/
- Microsoft Store (recommended): https://apps.microsoft.com/detail/9P6WFRBWG8X5
- Portable edition: https://github.com/ytec-forge-commits/ytec-calendar/releases/tag/v1.1.3
- Supported operating systems: Windows 10 / 11 (64-bit)
- License: Apache License 2.0

## Main features

- View, add, edit, and delete events in a monthly calendar
- Vacations, business trips, overnight timed events, and other multi-day events with start and end dates
- Copy, paste, and delete events from the right-click menu
- Drag to move an event, or hold Ctrl while dragging to copy it
- Daily, weekly, monthly, and yearly recurring events
- Birthdays and anniversaries fixed to the same date every year
- Edit or delete either one occurrence or an entire recurring series
- A pop-up list when you select a day that contains events
- Distinct colors for Saturdays, Sundays, and Japanese public holidays, with holiday names included for 1970 through 2050
- Stable compact display and list access even with five or more events on one day or more than ten events in a month
- Consistent LINE Seed JP typography, six event colors, and eight background themes
- A collapsible sidebar that remembers whether it is open or closed
- Window position and size saved separately for each monitor configuration
- Display scale slider from 80% to 130% in 5% increments
- Optional startup with Windows
- Three display modes: taskbar only, system tray only, or both
- Per-event pop-up reminders, five built-in notification sounds, and adjustable volume and playback duration
- Custom notification sounds in MP3, M4A, AAC, WAV, OGG, Opus, FLAC, or MIDI format
- Optional bidirectional Google Calendar synchronization with up to three accounts

Koyomado does not include printing, PDF export, advertising, analytics, or a proprietary Koyomado cloud server.

## Getting started

1. For everyday use, install the [Microsoft Store edition](https://apps.microsoft.com/detail/9P6WFRBWG8X5). When portability is needed, extract the [portable edition](https://github.com/ytec-forge-commits/ytec-calendar/releases/tag/v1.1.3) ZIP where you intend to keep it.
2. Launch the Store edition from the Start menu, or run `koyomado.exe` for the portable edition.
3. Open the gear menu in the upper-right corner and configure the display mode and Windows startup behavior.
4. Move and resize the window as desired. Koyomado will restore that placement the next time it starts with the same monitor configuration.

The default mode shows Koyomado only on the taskbar, and the close button exits the application. In “system tray only” or “both” mode, closing or minimizing hides the window; use “Exit” from the tray menu to quit completely.

The Microsoft Store edition uses a Windows StartupTask. If Windows disables it, enable Koyomado under Settings > Apps > Startup. The portable edition continues to wait up to five minutes after sign-in for its executable to become available, which helps on computers where Google Drive starts slowly. Before moving the portable folder, turn startup off, then turn it on again after the move.

## Start and end times and multi-day events

Each event can have a start date and an end date. With “all day” enabled, a vacation or business trip from September 1 through September 3 appears on every day in that period. With “all day” disabled, you can also enter start and end times. Changing the start time automatically sets the end time to one hour later; you can then change the end date and time manually.

When a multi-day event is copied and pasted, moved by dragging, or copied with Ctrl+drag, its original duration is preserved from the new start date. Each occurrence of a recurring event also retains its duration.

## Recurring events and anniversaries

Enable recurrence in the event editor to configure an interval and end condition.

- Daily: every day, every two days, and so on
- Weekly: select one or more weekdays
- Monthly: the same date, or the same weekday occurrence within the month
- Yearly: suitable for birthdays and anniversaries
- End condition: never, on a specified date, or after a specified number of occurrences

When editing or deleting a recurring event from the calendar, you can choose either “this event only” or “the entire series.” Dragging one occurrence moves only that occurrence. Holding Ctrl while dragging creates an independent copy.

## Google Calendar integration

Google integration is off by default. Koyomado communicates with Google only after the user creates a Desktop OAuth client in their own Google Cloud project and imports the downloaded JSON file. It uses an OAuth 2.0 client, not an API key. Koyomado does not use a shared Y-TEC OAuth client, so API usage by other users is not aggregated under Y-TEC.

You can connect up to three Google accounts. For each account, select one calendar and enable or disable synchronization. Under “Default destinations for new events,” you can select one account, multiple accounts, all accounts, or local storage only. Those destinations are selected automatically for a new event, but they can be cleared or changed for each event.

Synchronization runs at startup, when the window is shown again, approximately every 60 seconds, immediately after an event changes, and when “Sync now” is selected. It synchronizes titles, start and end dates and times, all-day status, locations, notes, recurrence, and reminders in both directions. For a new Koyomado event, “Save the reminder times above to Google” is selected by default. You can instead use the Google calendar's default reminders for an individual event; in that case, Google uses the target calendar's defaults rather than the reminder times specified in Koyomado. If the same event is changed in both Koyomado and Google at the same time, Koyomado preserves both conflicting copies and notifies the user instead of silently overwriting either one.

The illustrated manual on the official website and in the distribution ZIP explains how to create and connect the Google OAuth client. For normal personal use, the user switches their own OAuth project to “In production” in Google Auth Platform. The normal personal-use procedure does not require OAuth verification, a public home page, a privacy-policy URL, or an authorized domain. Google may display an unverified-app warning during the first authorization; proceed only after confirming that it is the project you created yourself. An unverified project can have a lifetime limit of 100 new users, but this normally does not affect Koyomado's model because each user creates a separate project and connects no more than three accounts. Do not leave the project in “Testing” for regular use, because authorization normally expires after seven days. Koyomado cannot inspect or change the Google Cloud publishing status.

The JSON file containing the complete client secret can be downloaded when the OAuth client is created. After importing it into Koyomado, keep the original JSON private and back it up in a safe location. If it is lost, the simplest recovery is to rotate the client secret in Google Auth Platform and download a new JSON file. If rotation is unavailable, create a new Desktop client.

Event data and synchronization settings are stored in the data folder for the edition in use, while Google refresh tokens are stored in Windows Credential Manager. When moving the portable edition to another computer, reauthorize each Google account on that computer. See the [privacy policy](PRIVACY.md) for details.

## Data storage and distribution channels

| Edition | Data location | Updates | Signing |
| --- | --- | --- | --- |
| Microsoft Store | `%LOCALAPPDATA%\Packages\Y-TEC.Koyomado_y7q84f7nwz24j\LocalState\Koyomado\data` | Microsoft Store | Microsoft Store distribution signature |
| Portable | `data` next to `koyomado.exe` | Extract the new ZIP and preserve the existing `data` | Y-TEC self-signature plus published SHA-256 |

The editions use separate default data locations. For manual migration from published v1.0.0, exit both editions, launch and exit the Store edition once, then copy the contents of the old `data` folder to the new location. Keep the source as a backup until events, settings, and custom sounds have been verified. The corrected source also supports folder selection below. Concurrent use can duplicate Google synchronization and reminders. On the same PC they share Google refresh tokens in Windows Credential Manager, so disconnecting an account in one edition requires reauthorization in the other.

Store-edition data is held in the package `LocalState`. Store updates normally retain it, but uninstalling the app can remove it. Before uninstalling or resetting Windows, exit Koyomado and copy the Store `data` folder to a separate backup location. Portable data remains next to the executable unless it is overwritten during a ZIP update.

### Choosing a data folder (1.1.0)

Both editions can select a Google Drive or other locally accessible sync folder in Settings > Calendar data location. Select a Windows folder, not a Drive web URL.

1. Create a dedicated empty folder, or select the existing portable `data` folder containing `calendar-data.json`.
2. Use the folder picker or enter a path and inspect it. Review event counts and the proposed action before confirming; inspection does not change the active location.
3. For an empty folder, copy current data. Events, backups, history and custom sounds are copied and verified before the location changes. The source remains intact.
4. For an existing calendar, explicitly use its data. The two calendars are not merged and neither is overwritten.

No app restart is required. Idle windows notice a location change, including when event contents are identical. An open editor keeps its input and refuses to save old-location changes into the new folder. An unavailable folder never silently falls back to the default or initializes empty data. If startup loading fails, the app retries automatically up to three times at ten-second intervals. You can also retry manually or select another existing calendar on the error screen.

Window positions stay in the original default folder. Location selection is PC-local: Store uses `LocalState/Koyomado/storage-location.json`; portable uses installation-specific files under `%LOCALAPPDATA%/Koyomado/storage-locations/`. Select the shared folder separately on each PC. Google refresh tokens remain in that PC's Windows Credential Manager. To return, specify the default folder and review its existing data before confirming.

Copy, verification or pointer-commit failure retains the previous active location. Partial copies are not deleted automatically: preserve the source and retry in a different empty folder. Avoid editing on other PCs during switching and wait for Drive to finish syncing. This does not automatically resolve undelivered or offline Drive conflicts.

### Contents of the data folder

- `data/calendar-data.json`: events, recurrence, appearance, and Google integration settings
- `data/calendar-data.backup.json`: the previous saved state
- `data/calendar-data.v1.backup.json` / `calendar-data.v2.backup.json` / `calendar-data.v3.backup.json` / `calendar-data.v4.backup.json`: pre-migration data from older formats (created only during migration)
- `data/notification-sounds/`: user-selected notification sounds (only when configured)
- `data/window-state.json`: positions and sizes for up to the 12 most recently used monitor configurations
- `data/window-state.backup.json`: the previous window-position state
- `data/window-state.v1.backup.json`: window-position data from the older format (created only during migration)

Storage formats version 1, 2, 3, and 4 are migrated automatically to version 5 on first launch, and the pre-migration files are retained. Event deletion is a soft delete; deleted records remain in `deletedEvents`.

## Event reminders

The event editor can schedule pop-up reminders from the event start time up to 28 days in advance. Common reminder times—10 minutes, 30 minutes, 1 hour, 3 hours, 6 hours, 12 hours, and 1 day before—can be toggled with one click, and multiple presets can be selected. You can also enter a custom number of minutes, hours, or days before the event. Internally, reminders are stored and synchronized as Google Calendar-compatible minute values. All-day reminders are based on 00:00 on the start date. For example, Google's default 30-minute reminder for an all-day event appears at 23:30 on the previous day. Koyomado reminders work only while the application is running. Notification sounds stop automatically after the configured 3-to-60-second duration, which defaults to 12 seconds, or immediately when “OK (stop sound)” is selected in the notification pop-up.

The settings screen includes “Yawaragi,” “Deep Drop,” “Small Bell,” “Morning Dew Piano,” and “Sunlight Kalimba,” as well as silent mode and a user-supplied sound. Custom sounds can be up to 15 MB and use MP3, M4A, AAC, WAV, OGG, Opus, FLAC, or MIDI. MIDI files are played with Koyomado's built-in gentle instrument, so they may sound different from the original sound source. Actual format support also depends on the codecs available to Windows WebView2.

### Concurrent use and data protection (1.1.0)

You can run the same portable folder on multiple PCs. **The protection below requires version 1.1.0 or later. Use 1.1.0 or later on every PC sharing the folder.** Mixed use with older versions is not protected.

While idle, the app checks for saved changes about every two seconds and when brought back into view. Editing keeps your input intact. Before saving, changes to separate events or settings are combined with the latest data. Conflicting changes to the same event, recurring series, or setting stop the save and leave your draft in the editor. Copy or record the input before closing the editor; the latest data can then load without restarting the app.

Windows also serializes local processes accessing the same calendar file. Saving replaces the primary without deleting it first. A missing or corrupt primary can recover from a backup or complete temporary file. Evidence of unrecoverable existing data stops empty initialization.

Before and after calendar writes, separate JSON snapshots are retained in `data/calendar-history/`. They contain events, deleted events and settings: keep them private just like the primary. History is not deleted automatically and grows with use. To restore, exit the app on all PCs, copy the entire `data` folder to a separate backup, inspect a history JSON, then copy it to `calendar-data.json`. A newer timestamp alone does not establish which snapshot is correct across PCs. Keep the original primary, backups and history.

Changes that Google Drive has not yet delivered, including offline edits, cannot be detected locally. History supports recovery; it does not automatically resolve Drive conflicts. Check that Drive has finished syncing before editing and maintain a separate backup of important events.

### ZIP backup and restore (1.1.0)

Settings can export and restore a Koyomado backup ZIP containing current events, deleted events, appearance settings and custom sounds. OAuth client configuration, credential-manager tokens, PC-local paths, window position, automatic-backup settings and historical JSON snapshots are excluded. ZIPs are not encrypted and contain private event/account information. Keep them private and retain the original OAuth client JSON separately.

Choose a ZIP and an empty dedicated restore folder, inspect its date/counts, then confirm. Files are extracted and verified before switching locations without a restart. Neither the old calendar nor the ZIP is overwritten or merged. An incomplete restore is blocked; retry in another empty folder. Backup format v1 checks SHA-256, allowed paths and supported calendar versions. Limits: 256 MB total, 32 MB per file, 2048 entries.

Google integration is disabled after restore. Reimport the OAuth client and reauthorize. The first sync of each restored account downloads the complete remote calendar and reconciles by Google event ID or original Koyomado ID, without remote creates, updates or deletes. Linked events use Google's latest content. Unmatched restored events remain local with synchronization targets removed; old deletion records are not sent to Google. The original restore content remains in the ZIP. Subsequent syncs use normal behavior. Live Google API restore integration testing is still required.

#### Move between the self-signed and Store editions

Update both editions to 1.1.2 or later before migrating. In either direction, stop editing the source, export a backup from Settings, then restore it in the destination edition into a dedicated empty folder. **The application distribution ZIP is not a calendar backup.** Retain both the source and the backup until events, multi-day durations, recurrence and custom sounds are verified. Reconfigure Google authorization and automatic backups on the destination. Before uninstalling the Store edition, save the ZIP outside its LocalState. For old 1.0.0 data, preserve it before updating, or select its existing data folder using the location feature above.

Version 1.1.2 repairs existing duplicate active IDs when loading, assigning distinct internal IDs to later records without deleting or merging events. Google event IDs, content and update timestamps remain unchanged. The exact original file is preserved and verified as `calendar-data.before-id-repair.<SHA256>.json` in the same data folder, with another source check immediately before saving. Repair stops if preservation fails. This original copy is not automatically deleted or included in normal ZIP exports; keep it private because it contains the original settings and personal data. Identical-looking local copies are retained.

Automatic backup settings are PC-local: select a separate folder, daily/weekly (Monday boundary)/monthly (first-day boundary), and 1–100 retained generations (default 10). While running, the app checks approximately every 60 seconds, once per period; a missed period creates one backup at next launch. Only after a new ZIP is successfully written, reread and verified are older unchanged files recorded for this automatic series removed. Manual ZIPs and other series are excluded. Old folders, files that could not be safely pruned and unregistered ZIPs from an interrupted run may remain; check the error status. Calendar history JSON files are outside this retention policy.

## Development

Building the ZIP feature from source requires Rust 1.88 or newer, matching `zip 9.0.4`.

Development requires Node.js, npm, stable Rust, and the Windows development environment required by Tauri 2.

```powershell
npm ci
npm run tauri:dev
```

Use `npm run dev` to inspect only the web interface. The browser build stores synthetic data in localStorage; Windows-specific features such as Google integration, startup registration, and the system tray do not work there.

### Verification

```powershell
npm run lint
npm test
npm run build
npm run test:windows-signing-policy
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test-partner-center-submission.ps1 -SkipArtifactHashCheck
Push-Location src-tauri
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
Pop-Location
npm run tauri:build
```

Integration tests with real Google accounts must use the user's own test OAuth client and synthetic events only. Never include an OAuth client JSON file or token in Git, logs, or Issues.

### Microsoft Store MSIX

MSIX packaging uses MakeAppx from the Windows SDK. Use the exact Package Identity assigned by Partner Center.

```powershell
npm run tauri:build
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/package-msix.ps1 -SkipBuild -PackageVersion 1.1.3.0 -CreateUpload
npm run test:store-submission
```

Do not self-sign the Store submission package. Partner Center applies the Store signature. The command creates `release/koyomado-v<version>-store-x64.msixupload`.

### Self-signed portable ZIP

The self-signed certificate is created as a non-exportable private key in `CurrentUser\My`; no PFX or private key is placed in the workspace. The release script signs a staging copy of the executable and packages only that verified copy.

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/new-self-signed-code-signing-certificate.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/package-self-signed-direct.ps1 -CertificateThumbprint <shown-thumbprint>
```

This produces the portable ZIP, a public-key-only `.cer`, the manual, and `SHA256SUMS.txt`. The signature receives an RFC 3161 timestamp. A self-signature does not guarantee that SmartScreen warnings disappear. No certificate is automatically added to a user's Trusted Root store.

For development only, the following command creates an unsigned ZIP. Do not publish it as the signed release.

```powershell
npm run tauri:portable
```

This creates `release/koyomado-v<version>-windows-portable.zip`. When updating an existing installation, preserve the current `data` folder and replace only the executable and bundled documents.

## Security and publication

- [Privacy policy](PRIVACY.md)
- [Security policy](SECURITY.md)
- [Code signing policy](CODE_SIGNING_POLICY.md)
- [Contribution guide](CONTRIBUTING.md)
- [Third-party licenses](THIRD_PARTY_NOTICES.md)

GitHub Releases and the official website distinguish the Microsoft Store and self-signed portable editions and state their signing, update, and storage behavior. Verify the published SHA-256 checksums. Code-signing operations follow the [code signing policy](CODE_SIGNING_POLICY.md).

Copyright 2026 Y-TEC. Licensed under the Apache License, Version 2.0.
