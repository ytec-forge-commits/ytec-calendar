"""Generate the English guide for Koyomado's existing Japanese interface."""
from __future__ import annotations

import argparse
import importlib.util
from pathlib import Path

from reportlab.lib.units import mm
from reportlab.platypus import BaseDocTemplate, Frame, Image, PageBreak, PageTemplate, Spacer

spec = importlib.util.spec_from_file_location("manual_ja", Path(__file__).with_name("generate-operation-manual.py"))
manual = importlib.util.module_from_spec(spec)
spec.loader.exec_module(manual)

OFFICIAL = "https://ytec.cloudfree.jp/forge/en/projects/koyomado/"
STORE = "https://apps.microsoft.com/detail/9P6WFRBWG8X5"
TOTAL_PAGES = 27

# The Japanese labels match the shipped interface; screenshots are not an English UI.
PAGES = [
    ("GETTING STARTED", "Start using Koyomado", "Choose the Microsoft Store edition for everyday use, or the portable edition when you need to carry its folder.", [
        ("Install and open", "Install the Store edition from the official listing, or extract the portable ZIP into a writable folder and open koyomado.exe. Windows 10/11 x64 and Microsoft Edge WebView2 are required."),
        ("Check the download", "Compare the ZIP's SHA-256 with SHA256SUMS.txt from the official release. The portable edition uses Y-TEC self-signing; it is not a publicly trusted certificate and does not guarantee removal of SmartScreen warnings. Do not install it into Trusted Root merely to dismiss a warning."),
        ("Set up your window", "Open the gear button for 「表示と起動の設定」 (display and startup settings). Choose taskbar/tray behavior, resize the window and move it to your preferred position."),
        ("Close or quit", "In taskbar-only mode, Close quits. In tray-only or both mode, Close/Minimize hides the window. Use 「終了」 (Quit) in the tray menu to stop the application."),
    ], [], [("Microsoft Store", STORE)]),
    ("SCREEN", "Read the calendar", "The interface is Japanese. This guide provides English instructions and keeps the actual Japanese control names where useful.", [
        ("Month and day", "Use the previous/next month controls and 「今日」 (Today). Click an empty date to add an event. Click a date that contains events to open its daily list, then choose the event to edit."),
        ("Sidebar and overflow", "The sidebar shows today's events and the next seven days. A multi-day event appears once in the seven-day list. Busy dates use abbreviated entries; open the daily list to see all events."),
        ("Colors", "Saturday, Sunday and Japanese holidays are distinguished by color. Built-in holiday data covers 1970-2050. Event colors and eight background themes are independent of holiday colors."),
    ], [("calendar-v1.png", "Month calendar and sidebar (Japanese interface).", 145)], []),
    ("SCHEDULE", "Add, edit and delete events", "Click an empty date or an Add control; click an event to edit it.", [
        ("Event fields", "Enter a title (up to 80 characters), start/end dates, optional location (100 characters), notes (1,000 characters) and one of six colors. 「終日」 means All day; turn it off to enter times."),
        ("Start and end", "Changing the start time resets the end time to one hour later. Set the start first, then adjust the end date/time. An event cannot end before it starts."),
        ("Save and delete", "Use 「保存」 (Save) to apply changes, or cancel to discard the editor's changes. Right-click an event for 「削除」 (Delete). Deleted records are retained internally as soft-deleted data, not a user-facing recycle bin."),
    ], [("period-editor-v1.png", "Event editor with start/end dates.", 78)], []),
    ("MULTI-DAY", "Trips and events across dates", "Use one event with a start and end date instead of separate duplicate entries.", [
        ("All-day periods", "For a three-day trip, enable 「終日」 and select its first and last date. The event appears on every included date."),
        ("Overnight events", "Turn off All day and set both dates and times. Copying, pasting or dragging keeps the duration. Recurring occurrences also keep the original duration."),
        ("Review a busy date", "Click the date to open the list. The calendar's shortened display does not remove events."),
    ], [("multi-day-v1.png", "One event spanning several calendar dates.", 145)], []),
    ("COPY AND MOVE", "Copy or move an event", "Use the event context menu or drag it to another date.", [
        ("Copy and paste", "Right-click the event and select 「内容をコピー」 (Copy contents). Right-click the destination date and choose Paste. The date changes while its duration is preserved."),
        ("Drag to move", "Drag an event to its new start date. For a recurring event, an ordinary drag moves only that occurrence."),
        ("Ctrl-drag to copy", "Hold Ctrl while dragging to create an independent copy. A recurring occurrence is copied as an independent event, not another linked occurrence."),
        ("Paste unavailable", "First copy an event using Koyomado's event menu. This internal event clipboard is not arbitrary text from the Windows clipboard."),
    ], [("calendar-v1.png", "Drag between dates or use context menus.", 140)], []),
    ("RECURRENCE", "Create repeating events", "Enable 「繰り返し」 (Repeat) in the event editor.", [
        ("Daily and weekly", "Choose an interval, such as every two days. Weekly repeats can use multiple weekdays."),
        ("Monthly and yearly", "Monthly repeats use the same date or a selected weekday/ordinal. Yearly repeats are suitable for birthdays and anniversaries. February 29 appears only in leap years."),
        ("End conditions", "Choose no end, an end date, or a number of occurrences. Review the start date, interval and end condition before saving."),
    ], [("recurrence-v1.png", "Repeat interval and end-condition controls.", 135)], []),
    ("RECURRENCE SCOPE", "Change one occurrence or a series", "Koyomado asks which scope to use when editing or deleting a recurring event.", [
        ("This event only", "「この予定のみ」 changes or removes only the selected occurrence. Other occurrences remain linked to their original series."),
        ("Whole series", "「繰り返し全体」 changes or removes the series. Check that this is really intended before confirming."),
        ("Daily list", "Select a date to open its list; click an entry to edit. Use this view when a calendar cell abbreviates several events."),
    ], [("agenda.png", "Daily agenda showing individual entries.", 105)], []),
    ("APPEARANCE", "Theme, size and position", "Use the gear button to open display and startup settings.", [
        ("Background and scale", "Choose one of eight themes. Display scale ranges from 80% to 130% in 5% steps, initially 100%. LINE Seed JP is the included interface font."),
        ("Sidebar", "The sidebar can be collapsed; Koyomado remembers this choice. The minimum width is about 806 px when expanded and 375 px when collapsed."),
        ("Window position", "The window's position and size are saved separately for recent monitor configurations (up to 12). Off-screen positions are corrected into a visible area. Move and close the window to save your preferred placement."),
    ], [("settings-v1.png", "Appearance settings.", 112)], []),
    ("WINDOW DISPLAY", "Taskbar, tray and startup", "Choose the display mode that matches how you use the calendar.", [
        ("Three modes", "Taskbar only is the default. Tray only and Both hide rather than quit when closed/minimized. Left-click the tray icon to show the window; right-click it for Show and Quit."),
        ("Windows startup", "The Store edition uses a Windows StartupTask. If Windows has disabled it, use Windows Settings > Apps > Startup. The portable edition waits up to five minutes after sign-in for its executable to become available."),
        ("Moving a portable folder", "Turn automatic startup off before moving the folder, then turn it on from the new location. Do not leave a startup entry pointing to the old folder."),
        ("Notifications need a running app", "Hiding in the tray keeps it running. Quitting stops local reminders and background application activity."),
    ], [], []),
    ("REMINDER", "Set event reminders", "Reminders run only while Koyomado is running; they are not a Windows scheduled-task service.", [
        ("Choose times", "Add up to five reminders per event, from the start time back to 28 days earlier. Presets include 10/30 minutes, 1/3/6/12 hours and 1 day; custom minutes, hours or days are also available."),
        ("All-day events", "Their start is midnight on the start date. A 30-minute reminder is therefore 23:30 on the previous date."),
        ("Google reminders", "New events initially save the chosen reminder times to Google. You can instead use the target calendar's default reminders; these are not the locally chosen times."),
        ("Popup and sleep", "Press 「OK（音を止める）」 to stop the sound. Reminders missed while the PC sleeps or the app is closed are not all replayed later; only a short recent window is considered."),
    ], [], []),
    ("NOTIFICATION SOUND", "Choose sounds and volume", "Open 「予定の通知音」 (event notification sound) in settings.", [
        ("Included sounds", "Choose 「やわらぎ」, 「深い雫」, 「小鈴」, 「朝露のピアノ」 or 「木漏れ日のカリンバ」, or no sound. The included five recordings are CC0; see the bundled sound notice."),
        ("Volume and duration", "Adjust the volume and preview the result. Automatic stop is initially 12 seconds and can be set from 3 to 60 seconds."),
        ("Your own file", "Import a file up to 15 MB: MP3, M4A, AAC, WAV, OGG, Opus, FLAC or MIDI. Supported playback also depends on Windows WebView2 codecs. MIDI uses the built-in gentle instrument and may sound different from its original synthesizer."),
        ("Where files go", "Custom audio is stored in the current edition's data/notification-sounds directory. Keep those files when backing up or moving your data."),
    ], [], []),
    ("DATA AND UPDATE", "Back up, update and move data", "Calendar data and OAuth client settings are not encrypted. Do not store passwords or confidential material in events.", [
        ("Two separate locations", "Portable: data beside koyomado.exe. Store: %LOCALAPPDATA%/Packages/Y-TEC.Koyomado_y7q84f7nwz24j/LocalState/Koyomado/data. These locations are not silently merged."),
        ("Back up first", "Quit the app and copy the entire data folder to another location. It contains calendar-data.json, its backup, window-state.json and optional notification-sounds. Do this before updates, uninstalling the Store edition or resetting the PC."),
        ("Update the portable edition", "Extract the new ZIP separately. Replace the executable and bundled documents, but retain your existing data folder; do not overwrite it with the empty distribution folder."),
        ("Move between editions", "Quit both editions, initialize and quit the destination edition once, then copy the data folder contents. Keep the original until events, settings and sounds have been checked. Do not run both against the same Google events: duplicate sync/reminders can result."),
        ("Old formats and accounts", "Versions 1-4 migrate to format 5 with versioned backups. Google refresh tokens stay in Windows Credential Manager, not this folder. Another PC requires reauthentication. On one PC, disconnecting an account in one edition can affect the other."),
    ], [], []),
    ("GOOGLE CALENDAR", "Optional Google synchronization", "Google integration is initially OFF. Koyomado does not use a shared Y-TEC OAuth client.", [
        ("Your own client", "Create a Desktop app OAuth client in your own Google Cloud project. Import its JSON into Koyomado. Do not use an API key, service account or Web application client."),
        ("Up to three accounts", "Connect at most three accounts, with one calendar per account. Enable or pause synchronization individually. New events can default to one, multiple, all, or no Google accounts."),
        ("Data exchanged", "Synchronization covers title, dates/times, all-day status, location, notes, recurrence and reminders. Both sides can edit. Simultaneous edits retain both versions and show a conflict."),
        ("Protect credentials", "Never send OAuth JSON, refresh tokens or real events to a public repository or support contact. Client settings are in the local unencrypted JSON; refresh tokens are stored using Windows Credential Manager."),
    ], [("google-settings-v1.png", "Optional integration settings; no real account is shown.", 110)], []),
    ("GOOGLE BEFORE START", "Personal-use production setting", "The Cloud screenshots are explanatory captures from August 2026. Audience and verification rules were checked against Google's official help on September 29, 2026.", [
        ("Personal use", "This procedure is for your own project and a small number of your own accounts. Google's personal-use exception allows fewer than 100 users without verification; it is not a blanket exception for a public multi-user OAuth service."),
        ("In production", "For routine use, change your project's Audience publishing status to In production. Testing authorizations for Calendar access, including refresh tokens, expire after seven days."),
        ("Warnings and caps", "In production does not verify the application. An unverified-app warning and a lifetime cap of 100 new users can still apply. Other token-expiry and administrator policies still apply."),
        ("Do not submit Y-TEC details", "You do not need Y-TEC's domain, website or privacy-policy URL for this personal-use procedure, and you do not submit an OAuth verification request. If Google's current requirements differ, review its help instead of inventing values."),
    ], [], [("Audience and limits", "https://support.google.com/cloud/answer/15549945?hl=en"), ("Verification exceptions", "https://support.google.com/cloud/answer/13464323?hl=en")]),
    ("GOOGLE CLOUD 1", "Create a project and enable the API", "Sign into Google Cloud using your own account. Names in the console can change; follow the current official documentation where they differ.", [
        ("Project", "Create a project you can identify, for example Koyomado, and select it in the project selector. Do not enable unrelated services or billing merely to follow this guide."),
        ("Calendar API", "Find Google Calendar API in the API Library and click Enable for that project."),
    ], [("oauth/01-google-project-create.png", "Create and select your project.", 145), ("oauth/02-enable-calendar-api.png", "Enable Google Calendar API.", 145)], [("Cloud Console", manual.GOOGLE_CONSOLE_URL)]),
    ("GOOGLE CLOUD 2", "Start Google Auth Platform", "Open Google Auth Platform for the selected project and choose Get started.", [
        ("App information", "Use a recognizable name such as Koyomado. Choose your own account for the support email; the guide hides personal information."),
    ], [("oauth/03-auth-platform-start.png", "Open the setup flow.", 145), ("oauth/04-oauth-app-info.png", "App name and your own support email.", 145)], []),
    ("GOOGLE CLOUD 3", "Audience and contact information", "For an ordinary personal Google account, choose External.", [
        ("Audience", "External permits your personal account to authorize this project. Internal is for an eligible Google Workspace organization, not ordinary personal Gmail accounts."),
        ("Contact", "Enter your own developer-contact email. The masked example does not contain an address to copy."),
    ], [("oauth/05-oauth-audience-external.png", "Select External.", 135), ("oauth/06-oauth-contact-email.png", "Enter your own contact information.", 135)], []),
    ("GOOGLE CLOUD 4", "Create a Desktop app OAuth client", "Review Google's User Data Policy before agreeing and completing the setup.", [
        ("Client type", "Create an OAuth client and select Desktop app (「デスクトップ アプリ」). A Web application, API key or service account does not work here."),
        ("Client name", "Choose a name you recognize. This client belongs to your own project and is not a Y-TEC-wide shared credential."),
    ], [("oauth/07-oauth-user-data-policy.png", "Review the policy and agree only if appropriate.", 135), ("oauth/08-create-desktop-client.png", "Select Desktop app.", 135)], [("Desktop credentials", manual.GOOGLE_CREDENTIALS_URL)]),
    ("GOOGLE CLOUD 5", "Save the client JSON", "Download the complete OAuth JSON when creating the client and store it privately.", [
        ("Protect the download", "The guide masks the ID and secret. Do not publish, attach or send your actual JSON to support. Back it up securely after importing."),
        ("If the JSON is lost", "Google may not show the complete secret again. Rotate the client secret and download a new JSON where available, or create a new Desktop client. Reconnect using the new client as needed."),
        ("Next", "Open Google Auth Platform > Audience to review the publishing status."),
    ], [("oauth/09-download-oauth-json.png", "Download JSON; credentials are hidden in this guide.", 145)], [("OAuth clients", manual.GOOGLE_AUTH_CLIENTS_URL)]),
    ("GOOGLE CLOUD 6", "Change the publishing status", "In Audience, select Publish app and confirm the change to In production.", [
        ("What this changes", "This avoids the Testing-specific seven-day authorization expiry for Calendar access. It is not an OAuth verification submission and does not remove all warnings or token-expiry causes."),
    ], [("oauth/11-publish-to-production.png", "Select Publish app.", 145), ("oauth/12-confirm-production.png", "Confirm the publishing-status change.", 145)], []),
    ("GOOGLE CLOUD 7", "Confirm In production", "Check the status before returning to Koyomado.", [
        ("Personal project only", "For this personal-use workflow, no verification application is submitted. If you make the OAuth project available to a larger audience, reassess Google's requirements."),
        ("The warning remains possible", "An unverified-app prompt is expected for unapproved sensitive scopes. Proceed only when the project and requested access are yours and match this guide."),
    ], [("oauth/13-production-complete.png", "The status is In production.", 145)], [("Audience", manual.GOOGLE_AUTH_AUDIENCE_URL)]),
    ("KOYOMADO CONNECT 1", "Import the JSON into Koyomado", "Keep the downloaded Desktop app JSON on your own PC.", [
        ("Enable integration", "Open the gear button, enable Google integration and choose 「JSONを選択」 (Select JSON)."),
        ("Choose and confirm", "Select the downloaded client JSON. On success the control changes to 「JSONを読み直す」 (Reload JSON), and a message confirms that the OAuth settings were loaded."),
        ("Import rejected", "Check the client type. API-key JSON, service-account JSON and Web application credentials are not supported."),
    ], [("oauth/14-koyomado-json-select.png", "Enable integration and select the JSON.", 105), ("oauth/15-koyomado-json-loaded.png", "Loaded settings, before connecting an account.", 105)], []),
    ("KOYOMADO CONNECT 2", "Authorize and confirm the account", "Choose 「アカウントを接続」 (Connect account); the default browser opens. Keep Koyomado running.", [
        ("Choose the correct account", "Check the selected Google account, project name and requested permissions. If you do not recognize them, cancel."),
        ("Unverified-app prompt", "Only for your own matching client, use Advanced and the continue option. Do not bypass warnings for an unknown client."),
        ("Permissions", "The app requests access to your calendar list and to view/edit calendar events, as well as account identification. Review and grant the required calendar access to use synchronization."),
        ("Finish within three minutes", "Koyomado's local authorization listener has a three-minute timeout. If it expires, start Connect account again. A browser success message alone is not enough: check the connected-account count in Koyomado."),
        ("Select the calendar", "After connection, choose the account's one target calendar and the default destination for new events."),
    ], [], []),
    ("GOOGLE SYNC", "Select destinations and synchronize", "Review each connected account before your first synchronization.", [
        ("Target calendar", "Choose 「同期するカレンダー」 (calendar to sync). After the first synchronization it is locked to prevent accidental relinking. To change it, disconnect and reconnect."),
        ("Account and event switches", "Enable 「このアカウントと同期」 for the account. In 「新しい予定の既定の保存先」 choose one/multiple/all accounts, or none for local only. An event's destinations can be adjusted individually; imported Google events retain their original source link."),
        ("When sync happens", "At startup/re-show, about every 60 seconds while the app is displayed, after event changes, or by 「今すぐ同期」 (Sync now). Check the last-sync time and any error."),
        ("Main and secondary PCs", "Use separate Koyomado folders and authenticate on each PC, with Google as the shared calendar. Do not run the same Google Drive folder on two PCs simultaneously: local JSON file conflicts are not resolved by Koyomado."),
        ("Try a synthetic event", "If needed, test one clearly marked, non-sensitive event with your own test account. Check both sides and remove it from both when finished."),
    ], [], []),
    ("GOOGLE TROUBLE", "Conflicts, disconnecting and errors", "The application preserves both edited versions when it detects conflicting changes.", [
        ("Conflict", "Review the two events and their conflict indication before keeping or removing a version. Do not assume one side silently won."),
        ("Disconnect", "「接続解除」 attempts to revoke the token, removes the stored Windows credential and sync links, and retains imported events as local events."),
        ("Reauthentication after seven days", "Check whether the project's Audience status is still Testing. Change to In production for personal routine use, then disconnect and reconnect to obtain new authorization."),
        ("No sync or callback", "Check the global integration switch, account switch and event destinations. Keep Koyomado open during browser authorization. Localhost blocking by a firewall/security product can prevent the callback."),
        ("Cost and policy", "Koyomado does not add billing settings or use Y-TEC's shared API client. You remain responsible for your Google Cloud account settings, service terms and any administrator restrictions."),
    ], [], [("Google User Data Policy", manual.GOOGLE_USER_DATA_URL)]),
    ("HELP", "Troubleshooting and support", "Check the displayed month, the taskbar/tray, and the data directory of the edition you are using.", [
        ("Missing events or window", "Open the correct month and daily list. Check hidden tray icons. If the saved position is off-screen, Koyomado brings it into view for the current monitor setup."),
        ("Reminders or sounds", "The app must be running. Check the event's reminder time, mute/sound choice and volume. Sleep-time or closed-app notifications are not replayed as a backlog."),
        ("Damaged data", "Koyomado tries the previous backup and preserves a damaged file with a corrupt suffix. If manual recovery is necessary, quit the app, retain the current files and restore your own backup to the correct edition's data folder."),
        ("Scope and license", "There is no printing, event PDF export, advertising, analytics or independent Koyomado cloud service. External application communication is the optional Google integration. See LICENSE.txt (Apache-2.0), PRIVACY.md and the third-party notices."),
        ("Contact safely", "Include the application/Windows versions, steps and non-sensitive error text. Do not send actual events, account addresses, OAuth JSON, tokens, passwords or personal information."),
    ], [], [("Public product page", OFFICIAL), ("Contact", manual.CONTACT_URL), ("Source", manual.SOURCE_URL)]),
]


def decorate_page(canvas, doc):
    if doc.page <= 1:
        return
    canvas.saveState()
    canvas.setStrokeColor(manual.LINE)
    canvas.line(manual.MARGIN_X, 12 * mm, manual.PAGE_W - manual.MARGIN_X, 12 * mm)
    canvas.setFont("KoyomadoRegular", 7.2)
    canvas.setFillColor(manual.MUTED)
    canvas.drawString(manual.MARGIN_X, 7.8 * mm, f"Koyomado User Guide v{manual.VERSION}")
    canvas.drawRightString(manual.PAGE_W - manual.MARGIN_X, 7.8 * mm, f"{doc.page} / {TOTAL_PAGES}")
    canvas.restoreState()


def generate(output):
    manual.register_fonts()
    styles = manual.build_styles()
    for style in styles.values():
        style.wordWrap = None
    styles["h1"].fontSize = 19
    styles["h1"].leading = 25
    logo = Image(str(manual.ROOT / "src/assets/koyomado-logo.png"), 22 * mm, 22 * mm)
    logo.hAlign = "CENTER"
    story = [logo, Spacer(1, 4 * mm), manual.p("Koyomado", styles["cover_title"]),
             manual.p(f"User Guide v{manual.VERSION}<br/>Revised September 29, 2026", styles["cover_subtitle"]),
             manual.screenshot("calendar-v1.png", 145 * mm), Spacer(1, 5 * mm),
             manual.p("Windows 10/11 x64. The application interface is Japanese; this guide explains its controls in English. Store and portable editions have separate data locations. Google integration is optional and initially off.", styles["body"]),
             manual.url_link("Public product page", OFFICIAL, styles), manual.url_link("Source", manual.SOURCE_URL, styles),
             manual.url_link("Contact", manual.CONTACT_URL, styles),
             manual.p("For republication, the source, distribution and documents were reviewed, and the executable and both guides were newly generated. Application features and storage format remain unchanged from v1.0.0.", styles["muted"]), PageBreak()]
    for index, (kicker, title, lead, sections, images, links) in enumerate(PAGES):
        manual.page_title(story, styles, kicker, title, lead)
        for heading, body in sections:
            story.extend([manual.p(heading, styles["h2"]), manual.p(body, styles["body"])])
        for filename, caption, width in images:
            story.extend([Spacer(1, 2 * mm), manual.screenshot_figure(filename, caption, styles, width * mm)])
        for label, url in links:
            story.append(manual.url_link(label, url, styles))
        if index < len(PAGES) - 1:
            story.append(PageBreak())
    output.parent.mkdir(parents=True, exist_ok=True)
    doc = BaseDocTemplate(str(output), pagesize=manual.A4, leftMargin=manual.MARGIN_X,
                          rightMargin=manual.MARGIN_X, topMargin=manual.MARGIN_TOP,
                          bottomMargin=manual.MARGIN_BOTTOM, title=f"Koyomado User Guide v{manual.VERSION}",
                          author="Y-TEC", subject="English instructions for the Japanese Koyomado interface",
                          creator="Koyomado manual generator")
    frame = Frame(doc.leftMargin, doc.bottomMargin, doc.width, doc.height, id="content",
                  leftPadding=0, rightPadding=0, topPadding=0, bottomPadding=0)
    doc.addPageTemplates([PageTemplate(id="main", frames=[frame], onPage=decorate_page)])
    doc.build(story)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=manual.ROOT / "docs/Koyomado.en.pdf")
    args = parser.parse_args()
    generate(args.output.resolve())
    print("English guide generated.")
