# Koyomado Microsoft Store submission draft

Prepared: 2026-10-17 (JST), version 1.1.0 release candidate

This document is a pre-submission working note. It does not authorize package upload, certification submission, Git push, Release publication, or Forge publication.

## Partner Center identity

- Product: Koyomado
- Store ID: `9P6WFRBWG8X5`
- Submission: a new 1.1.0 update submission has not been created or inspected. The previous Submission 1 reference belongs to the historical 1.0 release.
- Package identity name: `Y-TEC.Koyomado`
- Publisher: `CN=F7BD381A-C29C-41A4-B039-8E9962198E21`
- Publisher display name: `Y-TEC`
- Package family name: `Y-TEC.Koyomado_y7q84f7nwz24j`

## Intended availability

- Markets: all available markets, including future markets
- Audience: public
- Discoverability: available and discoverable in Microsoft Store
- Release: as soon as certification and publishing complete
- Stop acquisition: never
- Base price: Free
- Free trial: none (not applicable to a free app)
- Device family: Windows 10/11 Desktop only
- Future device families: let Microsoft decide

Rationale: Koyomado is an Apache-2.0 open-source freeware application with no paid features or in-app purchases. Microsoft documents these as the standard defaults except for the required base-price choice.

## Properties

- Primary category: Productivity (`仕事効率化`)
- Secondary category: none
- Personal information: Yes
  - Koyomado can optionally access and transmit the connected user's Google account address and Google Calendar event data when the user enables synchronization.
  - Koyomado does not transmit schedules or OAuth credentials to Y-TEC.
- Privacy policy: `https://github.com/ytec-forge-commits/ytec-calendar/blob/main/PRIVACY.md`
- Website: `https://ytec.cloudfree.jp/forge/projects/koyomado/`
- Support: `https://ytec.cloudfree.jp/forge/contact/`
- Purchases outside Microsoft Store commerce: No
- Accessibility-tested declaration: leave unchecked unless a complete accessibility conformance test is separately recorded
- Removable/alternate drive installation: use package/Partner Center validation result; do not promise portability for the Store-managed installation
- OneDrive automatic backup: leave at the Partner Center/package-derived default unless certification identifies a conflict
- Pen/ink: No
- Generative AI: No
- Special hardware requirements: none

## Age rating questionnaire

- Use the IARC questionnaire (no existing certificate/GRID)
- App type: all other app types
- Existing rating authority / physical-media rating: No
- Expected content answers: no violence, fear, sexuality, gambling, controlled substances, profanity, user-generated public content, or unrestricted web browsing

The questionnaire must be answered according to the exact wording shown by IARC. Do not infer answers for any newly introduced question.

## Store package

- Upload file: `release/koyomado-v1.1.0-store-x64.msixupload`
- SHA-256: `75a46e62457de0e207d02574b3fdc918556bf1ac2255bb91463af82f7d3262c6`
- Package architecture: x64
- Version: 1.1.0.0
- Device family selection: Windows 10/11 Desktop

The upload is an external action and requires an explicit confirmation immediately before selecting the file in Partner Center.

## Japanese Store listing

### Description

Koyomadoは、Windowsデスクトップへウィジェットのように置いて使える、落ち着いたデザインのカレンダーです。

月間カレンダーと直近7日間の予定を一画面で確認し、予定の追加・編集・コピー・ドラッグ移動をすばやく行えます。土曜・日曜・日本の祝日を色分けし、祝日の名前も表示します。

毎日・毎週・毎月・毎年の繰り返し予定、誕生日や記念日、複数日にまたがる予定、指定時刻のポップアップ通知に対応しています。通知音は内蔵音から選べるほか、自分の音声ファイルも使用できます。

Google Calendarとの同期は任意です。利用者自身が用意したGoogle OAuthクライアントを使い、最大3アカウントと双方向同期できます。連携しない場合は、予定を端末内だけで管理できます。

背景テーマは8種類、表示倍率は80～130%から選択できます。ウィンドウ位置はディスプレイ構成ごとに保存され、タスクバー・タスクトレイの表示方法も設定できます。

KoyomadoはApache License 2.0で公開しているオープンソースソフトウェアです。

### What's new in version 1.1.0

Store版でもGoogle Drive等の同期フォルダーを予定の保存先に指定できます。アプリ内のZIPバックアップ・空フォルダーへの復元、毎日・毎週・毎月の自動バックアップと1～100世代の保持設定を追加しました。ZIPでStore版とポータブル版の間を移行できます。複数起動時の変更検知・競合保存保護と日英マニュアルも更新しています。Google連携は復元後に再設定が必要で、最初の同期は受信・照合のみを行います。

### Feature list

- 月間カレンダーと直近7日間の予定を一画面で確認
- 土曜・日曜・日本の祝日を色分けし、祝日名を表示
- 予定の追加・編集・コピー・削除・ドラッグ移動
- 毎日・毎週・毎月・毎年の繰り返し予定と記念日
- 開始日と終了日を指定できる複数日予定
- ポップアップ通知と、選べる通知音・音量・再生時間
- 任意で最大3つのGoogle Calendarと双方向同期
- 8種類の背景テーマと80～130%の表示倍率
- タスクバー・タスクトレイ・両方から表示方法を選択
- ディスプレイ構成ごとのウィンドウ位置保存
- ローカル中心のデータ保存とApache-2.0のオープンソース
- Store版を含む予定データ保存先の選択
- ZIPバックアップ・復元と日・週・月の自動バックアップ、1～100世代の保持

### Search terms

`カレンダー,予定,スケジュール,デスクトップ,Google Calendar,リマインダー,祝日`

## English Store listing

The application UI is Japanese. Japanese and English manuals are included. Do not claim English UI support. The English listing must clearly state that the interface is currently Japanese.

### Description

Koyomado is a calm, desktop-friendly calendar for Windows 10 and Windows 11. It keeps a monthly calendar and the next seven days of events in one focused window, with clear color treatment for weekends and Japanese public holidays.

Create, edit, copy, delete, and drag events between dates. Koyomado supports multi-day events, daily/weekly/monthly/yearly recurrence, birthdays and anniversaries, and configurable popup reminders with built-in or custom notification sounds.

Optional Google Calendar integration provides two-way synchronization with up to three accounts using an OAuth desktop client created by the user. Without Google integration, events remain local to the device.

Choose from eight background themes, adjust the interface scale from 80% to 130%, and select taskbar, system-tray, or combined window behavior. Window positions are saved for each display configuration.

Koyomado is open-source software released under the Apache License 2.0. The application interface is Japanese; Japanese and English operation manuals are included.

### What's new in version 1.1.0

Choose a calendar-data folder, including a Drive-synchronized folder, in either edition. Save and restore ZIP backups, migrate between Store and portable editions, and schedule daily, weekly or monthly automatic backups with 1-100 retained generations. Concurrent-use change detection and save-conflict protection have been improved. Google integration requires setup after a restore; its first sync only receives and reconciles Google data. Japanese and English manuals are included.

### Feature list

- Monthly calendar and upcoming seven-day agenda
- Japanese public holidays and weekend color coding
- Add, edit, copy, delete, and drag events
- Recurring, anniversary, and multi-day events
- Popup reminders with configurable sounds and volume
- Optional two-way Google Calendar sync for up to three accounts
- Eight calming themes and 80–130% interface scaling
- Taskbar and system-tray display options
- Per-monitor-layout window position memory
- Local-first, Apache-2.0 open-source software
- User-selected calendar-data folders in both editions
- ZIP backup/restore and daily, weekly or monthly automatic backup with configurable retention

## Selected Store screenshots

All selected screenshots are PNG, 1440 x 900, generated from the frozen 1.0.0 candidate in a browser preview with synthetic schedule data only. They satisfy Microsoft's desktop minimum of 1366 x 768.

1. `docs/release/final-strict/store-assets/koyomado-store-calendar-ja.png`
   - Caption: 月間カレンダーと直近7日間の予定を、一つの落ち着いた画面で確認できます。
2. `docs/release/final-strict/store-assets/koyomado-store-event-editor-ja.png`
   - Caption: 複数日・繰り返し・通知時刻などを、分かりやすい編集画面から設定できます。
3. `docs/release/final-strict/store-assets/koyomado-store-day-agenda-ja.png`
   - Caption: 予定がある日を選ぶと、その日の予定をポップアップで確認できます。
4. `docs/release/final-strict/store-assets/koyomado-store-moon-theme-ja.png`
   - Caption: 8種類の背景テーマから、デスクトップに合う落ち着いた配色を選べます。
5. `docs/release/final-strict/store-assets/koyomado-store-settings-ja.png`
   - Caption: 表示倍率、タスクバー・トレイ動作、自動起動、通知音をまとめて調整できます。

The settings screenshot contains a web-preview-only note for the disabled autostart control. Prefer the first four screenshots for initial submission; use the fifth only if its context is acceptable.

## Certification notes draft

Koyomado is a Windows desktop calendar. Core calendar functionality works without an account or network connection. Google Calendar integration is optional and disabled by default. If a tester enables it, the application asks the tester to select a Google OAuth desktop-client JSON created in the tester's own Google Cloud project; no shared Y-TEC OAuth credential is embedded.

The Store package defaults to its package LocalState; the portable ZIP defaults to an adjacent data folder. Either edition can use a user-selected calendar-data folder. Restore only writes into an empty folder and preserves the source. ZIPs omit OAuth clients and credentials, disable Google sync on restore, and require reconfiguration. The first Google sync after restore only receives and reconciles IDs, with no remote additions, updates or deletions. Automatic backup prunes only unchanged owned archives after a verified new backup. No in-app purchase, advertising, analytics, telemetry, or Y-TEC account is used.

The application can register itself for Windows sign-in startup only after the user explicitly enables that setting. Calendar popup reminders work while Koyomado is running.

Restricted capability justification for Submission options: Koyomado is a packaged Win32/Tauri desktop application and requires `runFullTrust` only to launch its primary desktop executable. It uses full trust for calendar-file access in package `LocalState` or a user-selected folder, ZIP backup/restore and owned automatic-backup retention in user-selected folders, Windows Credential Manager access for optional Google OAuth refresh tokens, system-tray integration, window placement, user-selected notification audio, and the user-controlled StartupTask. It does not install a service or driver, modify protected system files, elevate to administrator, inject into other processes, or execute downloaded code.

## Protected final actions

The historical 1.0 final-strict review limitation is retained in prior Git history; it is not a 1.1.0 review result. For this 1.1.0 candidate, consult the dated release-check report and exact artifact hashes. The installed synthetic test package has a separate identity and is not the Partner Center artifact. This draft does not authorize any of the following actions:

- upload the `.msixupload` file;
- save completed Partner Center sections if they include externally visible declarations;
- submit for certification;
- commit or push the release candidate;
- create a GitHub Release;
- publish the Forge/WordPress update.

Partner Center's final **Submit for certification** action always requires immediate user confirmation.
