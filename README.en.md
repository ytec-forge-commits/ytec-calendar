# Koyomado

[日本語](README.md) | English

Koyomado is a simple portable calendar for Windows that can sit on your desktop like a widget. Events and settings are stored next to the executable, so you can move the entire folder between computers on a USB drive or through Google Drive.

- Official website: https://ytec.cloudfree.jp/ytb/koyomado/
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

1. Extract the distribution ZIP to the location where you intend to keep it.
2. Run `koyomado.exe`.
3. Open the gear menu in the upper-right corner and configure the display mode and Windows startup behavior.
4. Move and resize the window as desired. Koyomado will restore that placement the next time it starts with the same monitor configuration.

The default mode shows Koyomado only on the taskbar, and the close button exits the application. In “system tray only” or “both” mode, closing or minimizing hides the window; use “Exit” from the tray menu to quit completely.

When Windows startup is enabled, Koyomado waits up to five minutes after sign-in for its executable to become available. This improves reliability on computers where Google Drive starts slowly. Before moving the application folder, turn startup off, then turn it on again after the move.

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

Event data and synchronization settings are stored in JSON next to the executable, but Google refresh tokens are stored in Windows Credential Manager. When moving the application folder to another computer, reauthorize each Google account on that computer. See the [privacy policy](PRIVACY.md) for details.

## Portable data

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

When using Koyomado from Google Drive, do not run the same folder simultaneously on multiple computers. Koyomado cannot resolve file-level conflicts created by Google Drive itself.

## Development

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
Push-Location src-tauri
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
Pop-Location
npm run tauri:build
```

Integration tests with real Google accounts must use the user's own test OAuth client and synthetic events only. Never include an OAuth client JSON file or token in Git, logs, or Issues.

### Portable ZIP

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

GitHub Releases and the official website state whether each distribution is signed or unsigned. Verify the SHA-256 checksum before running an unsigned build. Code-signing operations follow the [code signing policy](CODE_SIGNING_POLICY.md).

Copyright 2026 Y-TEC. Licensed under the Apache License, Version 2.0.
