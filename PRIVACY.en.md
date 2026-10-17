# Koyomado privacy policy

Updated: 2026-10-17, for the 1.1.0 release. [Japanese](PRIVACY.md).

Koyomado has no advertising, analytics or proprietary calendar server. Google Calendar integration is optional and initially disabled.

## Locally stored information

Calendar files contain events, deleted events, appearance and notification settings, Google client configuration and account/link information. The Store edition defaults to package LocalState; the portable edition defaults to `data` next to the executable. If you explicitly select another folder, calendar data, backups, history and custom sounds use that folder. Source files remain after a verified copy and switch. Window placement stays in the default folder and location settings are PC-local. Google refresh tokens stay in Windows Credential Manager and are not moved with the folder.

The calendar JSON, history and ZIPs are not encrypted. Protect them from unauthorized access and do not use event notes to store passwords or other secrets.

## Optional Google integration

You supply your own Desktop OAuth client. When enabled, Koyomado accesses the selected accounts' calendar lists and events, including titles, dates, times, locations, notes, recurrence and reminders, to provide synchronization. Up to three accounts are supported. Refresh tokens are stored in Windows Credential Manager, not in calendar JSON or backup ZIPs. OAuth client JSON must be kept private and backed up separately.

Google-related requests use Google OAuth and Calendar services. Authentication temporarily uses a local loopback callback. Calendar contents and credentials are not sent to Y-TEC. When Google integration is off, Koyomado does not contact those services.

If you select a Google Drive or other synchronized folder, the service's client or Windows may synchronize files independently of Koyomado's Google Calendar switch. Check the folder's sharing permissions. Koyomado does not connect to a Drive API.

## Backup and restore

Manual and automatic backup ZIPs contain current/deleted events, settings, custom sounds and Google account/link identifiers. They exclude the OAuth client, credential-manager tokens, PC-local paths, window placement, automatic-backup configuration and past history snapshots. ZIPs are not encrypted; keep their destinations private.

Automatic-backup location, interval, retained generation count, results and file ownership catalog are stored on the PC. Only unchanged ZIPs owned by that automatic series are pruned, after a new ZIP is saved and verified. Manual ZIPs and other series are retained. History JSON is not limited by the ZIP retention count.

Restore writes into an empty folder and preserves the old calendar and ZIP. Google integration is disabled. After client import and reauthorization, the first sync downloads and reconciles without sending remote additions, updates or deletions. Linked events use Google's latest content, unmatched restored events remain local, and old deletions are not sent. The original backup remains available for inspection.

## Disconnecting and removing data

Disconnecting a Google account attempts token revocation, removes its local credentials and sync links, and keeps imported events locally. Editions on the same PC share the account's credential-manager token, so disconnecting in one can require reauthorization in the other.

Before deleting calendar data or uninstalling the Store edition, save a backup outside the application's LocalState and verify it. Uninstalling can remove LocalState. A selected external folder, ZIPs, history and PC-local settings may remain; inspect every location you used. Exit Koyomado on all PCs before intentional manual cleanup. Deleting local data does not automatically delete Google Calendar events.

## Contact and vulnerability reports

Use the official project's support channels without sending real event contents, OAuth client JSON or tokens. See [SECURITY.md](SECURITY.md) for private vulnerability reporting guidance.
