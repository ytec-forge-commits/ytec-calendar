use crate::credentials;
use chrono::{Datelike, Utc};
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    EndpointNotSet, EndpointSet, PkceCodeChallenge, RedirectUrl, RefreshToken, Scope,
    TokenResponse, TokenUrl,
};
use reqwest::blocking::Client;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    net::TcpListener,
    ptr,
    sync::Mutex,
    thread,
    time::{Duration, Instant},
};
use url::Url;
use windows_sys::Win32::UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOWNORMAL};

use super::{
    default_true, load_app_data_inner, portable_data_dir, write_json_with_backup, AppData,
    CalendarEvent, DeletedCalendarEvent, EventOrigin, EventRecurrence, EventReminders, EventStyle,
    GoogleAccount, GoogleEventLink, GoogleOAuthClient, RecurrenceEnd, RecurrenceException,
    SyncConflict, APP_DATA_LOCK,
};

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_CALENDAR_API: &str = "https://www.googleapis.com/calendar/v3";
const GOOGLE_USER_INFO_URL: &str = "https://www.googleapis.com/oauth2/v2/userinfo";
const GOOGLE_REVOKE_URL: &str = "https://oauth2.googleapis.com/revoke";
const OAUTH_TIMEOUT: Duration = Duration::from_secs(180);
static GOOGLE_SYNC_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GoogleCalendarOption {
    id: String,
    name: String,
    primary: bool,
    access_role: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GoogleConnectionResult {
    account: GoogleAccount,
    calendars: Vec<GoogleCalendarOption>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CredentialStatus {
    account_id: String,
    available: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DisconnectResult {
    revoked: bool,
    message: String,
}

#[derive(Debug, Deserialize)]
struct UserInfoResponse {
    id: String,
    email: String,
    #[serde(default)]
    name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CalendarListResponse {
    #[serde(default)]
    items: Vec<CalendarListEntry>,
    next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CalendarListEntry {
    id: String,
    summary: String,
    #[serde(default)]
    primary: bool,
    #[serde(default)]
    access_role: String,
    #[serde(default)]
    deleted: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct GoogleEventDateTime {
    date: Option<String>,
    date_time: Option<String>,
    time_zone: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct GoogleApiEvent {
    id: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    etag: String,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    location: String,
    #[serde(default)]
    start: GoogleEventDateTime,
    #[serde(default)]
    end: GoogleEventDateTime,
    #[serde(default)]
    recurrence: Vec<String>,
    #[serde(default)]
    reminders: GoogleEventReminders,
    recurring_event_id: Option<String>,
    original_start_time: Option<GoogleEventDateTime>,
    #[serde(default)]
    updated: String,
    #[serde(default)]
    extended_properties: GoogleExtendedProperties,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoogleEventReminderOverride {
    method: String,
    minutes: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoogleEventReminders {
    #[serde(default = "default_true")]
    use_default: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    overrides: Vec<GoogleEventReminderOverride>,
}

impl Default for GoogleEventReminders {
    fn default() -> Self {
        Self {
            use_default: true,
            overrides: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
struct GoogleExtendedProperties {
    #[serde(default)]
    private: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EventsListResponse {
    #[serde(default)]
    items: Vec<GoogleApiEvent>,
    next_page_token: Option<String>,
    next_sync_token: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GoogleEventDateTimeMutation {
    #[serde(skip_serializing_if = "Option::is_none")]
    date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    date_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    time_zone: Option<String>,
}

#[derive(Debug, Serialize)]
struct ExtendedPropertiesMutation {
    private: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GoogleEventMutation {
    summary: String,
    description: String,
    location: String,
    start: GoogleEventDateTimeMutation,
    end: GoogleEventDateTimeMutation,
    #[serde(skip_serializing_if = "Option::is_none")]
    recurrence: Option<Vec<String>>,
    reminders: GoogleEventReminders,
    extended_properties: ExtendedPropertiesMutation,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GoogleSyncSummary {
    accounts_synced: usize,
    pulled: usize,
    pushed: usize,
    deleted: usize,
    conflicts: usize,
    warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GoogleSyncResult {
    data: AppData,
    summary: GoogleSyncSummary,
}

enum EventsPageError {
    Gone,
    Other(String),
}

type GoogleOauthClient =
    BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>;

fn oauth_client(
    config: &GoogleOAuthClient,
    redirect_url: Option<String>,
) -> Result<GoogleOauthClient, String> {
    if config.client_id.trim().is_empty() {
        return Err("OAuthクライアントIDが空です".into());
    }
    let client_secret = (!config.client_secret.trim().is_empty())
        .then(|| ClientSecret::new(config.client_secret.clone()));
    let client = BasicClient::new(ClientId::new(config.client_id.clone()))
        .set_auth_uri(
            AuthUrl::new(GOOGLE_AUTH_URL.into())
                .map_err(|error| format!("Google認証URLを準備できません: {error}"))?,
        )
        .set_token_uri(
            TokenUrl::new(GOOGLE_TOKEN_URL.into())
                .map_err(|error| format!("GoogleトークンURLを準備できません: {error}"))?,
        );
    let client = match client_secret {
        Some(secret) => client.set_client_secret(secret),
        None => client,
    };
    match redirect_url {
        Some(redirect) => RedirectUrl::new(redirect)
            .map(|url| client.set_redirect_uri(url))
            .map_err(|error| format!("Google認証の戻り先URLを準備できません: {error}")),
        None => Ok(client),
    }
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn open_browser(url: &str) -> Result<(), String> {
    let target = wide(url);
    let result = unsafe {
        ShellExecuteW(
            ptr::null_mut(),
            ptr::null(),
            target.as_ptr(),
            ptr::null(),
            ptr::null(),
            SW_SHOWNORMAL,
        )
    } as isize;
    if result <= 32 {
        return Err(format!(
            "既定のブラウザーを開けませんでした（エラー {result}）"
        ));
    }
    Ok(())
}

fn oauth_success_html() -> &'static str {
    "<!doctype html><html lang=\"ja\"><meta charset=\"utf-8\"><title>Koyomado</title><body style=\"font-family:system-ui;padding:40px;background:#eef5f1;color:#23352e\"><h1>Koyomadoと接続しました</h1><p>このタブを閉じて、Koyomadoへ戻ってください。</p></body></html>"
}

fn oauth_error_html() -> &'static str {
    "<!doctype html><html lang=\"ja\"><meta charset=\"utf-8\"><title>Koyomado</title><body style=\"font-family:system-ui;padding:40px;background:#fff1f1;color:#613333\"><h1>接続を完了できませんでした</h1><p>Koyomadoへ戻って、もう一度お試しください。</p></body></html>"
}

fn wait_for_authorization(listener: TcpListener, expected_state: &str) -> Result<String, String> {
    listener
        .set_nonblocking(true)
        .map_err(|error| format!("Google認証の待受を開始できません: {error}"))?;
    let started = Instant::now();
    while started.elapsed() < OAUTH_TIMEOUT {
        match listener.accept() {
            Ok((mut stream, _)) => {
                stream.set_read_timeout(Some(Duration::from_secs(10))).ok();
                let mut buffer = [0_u8; 16_384];
                let size = stream
                    .read(&mut buffer)
                    .map_err(|error| format!("Google認証結果を読み込めません: {error}"))?;
                let request = String::from_utf8_lossy(&buffer[..size]);
                let target = request
                    .lines()
                    .next()
                    .and_then(|line| line.split_whitespace().nth(1))
                    .ok_or_else(|| "Google認証結果の形式が正しくありません".to_string())?;
                let callback = Url::parse(&format!("http://127.0.0.1{target}"))
                    .map_err(|error| format!("Google認証結果を解析できません: {error}"))?;
                let parameter = |name: &str| {
                    callback
                        .query_pairs()
                        .find(|(key, _)| key == name)
                        .map(|(_, value)| value.into_owned())
                };
                let state = parameter("state");
                let code = parameter("code");
                let error = parameter("error");
                let success = state.as_deref() == Some(expected_state) && code.is_some();
                let html = if success {
                    oauth_success_html()
                } else {
                    oauth_error_html()
                };
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    html.len(),
                    html
                );
                let _ = stream.write_all(response.as_bytes());
                if let Some(error) = error {
                    return Err(format!(
                        "Google認証がキャンセルされたか拒否されました: {error}"
                    ));
                }
                if state.as_deref() != Some(expected_state) {
                    return Err(
                        "Google認証の安全確認に失敗しました。もう一度接続してください".into(),
                    );
                }
                return code.ok_or_else(|| "Googleから認証コードを受け取れませんでした".into());
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(100));
            }
            Err(error) => return Err(format!("Google認証の待受に失敗しました: {error}")),
        }
    }
    Err("Google認証が3分以内に完了しませんでした".into())
}

fn api_client() -> Result<Client, String> {
    Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(30))
        .user_agent("Koyomado/1.0")
        .build()
        .map_err(|error| format!("Google通信用クライアントを準備できません: {error}"))
}

fn parse_google_response<T: DeserializeOwned>(
    response: reqwest::blocking::Response,
) -> Result<T, String> {
    let status = response.status();
    if !status.is_success() {
        let body = response.text().unwrap_or_default();
        let message = serde_json::from_str::<serde_json::Value>(&body)
            .ok()
            .and_then(|value| {
                value
                    .pointer("/error/message")
                    .and_then(|item| item.as_str())
                    .map(str::to_string)
            })
            .unwrap_or_else(|| format!("HTTP {status}"));
        return Err(format!(
            "Google Calendar APIでエラーが発生しました: {message}"
        ));
    }
    response
        .json::<T>()
        .map_err(|error| format!("Google Calendar APIの応答を読み取れません: {error}"))
}

fn list_calendars_with_token(access_token: &str) -> Result<Vec<GoogleCalendarOption>, String> {
    let client = api_client()?;
    let mut page_token: Option<String> = None;
    let mut calendars = Vec::new();
    loop {
        let mut request = client
            .get(format!("{GOOGLE_CALENDAR_API}/users/me/calendarList"))
            .bearer_auth(access_token)
            .query(&[("maxResults", "250")]);
        if let Some(token) = page_token.as_deref() {
            request = request.query(&[("pageToken", token)]);
        }
        let page: CalendarListResponse = parse_google_response(
            request
                .send()
                .map_err(|error| format!("Googleのカレンダー一覧を取得できません: {error}"))?,
        )?;
        calendars.extend(page.items.into_iter().filter_map(|item| {
            (!item.deleted && matches!(item.access_role.as_str(), "owner" | "writer")).then_some(
                GoogleCalendarOption {
                    id: item.id,
                    name: item.summary,
                    primary: item.primary,
                    access_role: item.access_role,
                },
            )
        }));
        page_token = page.next_page_token;
        if page_token.is_none() {
            break;
        }
    }
    calendars.sort_by_key(|calendar| !calendar.primary);
    Ok(calendars)
}

fn refresh_access_token(config: &GoogleOAuthClient, account_id: &str) -> Result<String, String> {
    let refresh_token = credentials::read_refresh_token(account_id)?
        .ok_or_else(|| "このPCにはGoogleの認証情報がありません。再認証してください".to_string())?;
    let client = oauth_client(config, None)?;
    let http_client = api_client()?;
    let token = client
        .exchange_refresh_token(&RefreshToken::new(refresh_token))
        .request(&http_client)
        .map_err(|error| format!("Googleの認証を更新できません: {error}"))?;
    Ok(token.access_token().secret().to_string())
}

fn calendar_api_url(calendar_id: &str, segments: &[&str]) -> Result<Url, String> {
    let mut url = Url::parse(GOOGLE_CALENDAR_API)
        .map_err(|error| format!("Google Calendar APIのURLを準備できません: {error}"))?;
    {
        let mut path = url
            .path_segments_mut()
            .map_err(|_| "Google Calendar APIのURLを組み立てられません".to_string())?;
        path.push("calendars").push(calendar_id);
        for segment in segments {
            path.push(segment);
        }
    }
    Ok(url)
}

fn date_from_google(value: &GoogleEventDateTime) -> Option<String> {
    value.date.clone().or_else(|| {
        value
            .date_time
            .as_ref()
            .and_then(|item| item.get(0..10))
            .map(str::to_string)
    })
}

fn time_from_google(value: &GoogleEventDateTime) -> String {
    value
        .date_time
        .as_ref()
        .and_then(|item| item.get(11..16))
        .unwrap_or("")
        .to_string()
}

fn add_one_day(date: &str) -> Result<String, String> {
    let parsed = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|_| format!("日付の形式が正しくありません: {date}"))?;
    Ok((parsed + chrono::Duration::days(1))
        .format("%Y-%m-%d")
        .to_string())
}

fn compact_date(date: &str) -> String {
    date.replace('-', "")
}

fn weekday_code(index: u8) -> &'static str {
    match index {
        0 => "SU",
        1 => "MO",
        2 => "TU",
        3 => "WE",
        4 => "TH",
        5 => "FR",
        _ => "SA",
    }
}

fn recurrence_lines(event: &CalendarEvent) -> Result<Vec<String>, String> {
    let Some(recurrence) = event.recurrence.as_ref() else {
        return Ok(Vec::new());
    };
    match recurrence {
        EventRecurrence::Google {
            lines,
            excluded_dates,
            ..
        } => {
            let mut result = lines.clone();
            append_exdates(&mut result, event, excluded_dates);
            Ok(result)
        }
        EventRecurrence::Simple {
            frequency,
            interval,
            week_days,
            monthly_mode,
            end,
            excluded_dates,
        } => {
            let mut parts = vec![format!("FREQ={}", frequency.to_ascii_uppercase())];
            if *interval > 1 {
                parts.push(format!("INTERVAL={interval}"));
            }
            if frequency == "weekly" {
                let days = if week_days.is_empty() {
                    let date = chrono::NaiveDate::parse_from_str(&event.date, "%Y-%m-%d")
                        .map_err(|_| "繰り返し予定の開始日が正しくありません".to_string())?;
                    vec![date.weekday().num_days_from_sunday() as u8]
                } else {
                    week_days.clone()
                };
                parts.push(format!(
                    "BYDAY={}",
                    days.iter()
                        .map(|day| weekday_code(*day))
                        .collect::<Vec<_>>()
                        .join(",")
                ));
            }
            if frequency == "monthly" && monthly_mode == "weekday-of-month" {
                let date = chrono::NaiveDate::parse_from_str(&event.date, "%Y-%m-%d")
                    .map_err(|_| "繰り返し予定の開始日が正しくありません".to_string())?;
                let ordinal = ((date.day() - 1) / 7) + 1;
                parts.push(format!(
                    "BYDAY={}{}",
                    ordinal,
                    weekday_code(date.weekday().num_days_from_sunday() as u8)
                ));
            }
            match end {
                RecurrenceEnd::Never => {}
                RecurrenceEnd::Until { date } => {
                    let value = if event.all_day {
                        compact_date(date)
                    } else {
                        format!("{}T235959Z", compact_date(date))
                    };
                    parts.push(format!("UNTIL={value}"));
                }
                RecurrenceEnd::Count { count } => parts.push(format!("COUNT={}", count.max(&1))),
            }
            let mut result = vec![format!("RRULE:{}", parts.join(";"))];
            append_exdates(&mut result, event, excluded_dates);
            Ok(result)
        }
    }
}

fn append_exdates(lines: &mut Vec<String>, event: &CalendarEvent, excluded_dates: &[String]) {
    if excluded_dates.is_empty() {
        return;
    }
    if event.all_day {
        lines.push(format!(
            "EXDATE;VALUE=DATE:{}",
            excluded_dates
                .iter()
                .map(|date| compact_date(date))
                .collect::<Vec<_>>()
                .join(",")
        ));
    } else {
        let time = event.start_time.replace(':', "");
        lines.push(format!(
            "EXDATE;TZID=Asia/Tokyo:{}",
            excluded_dates
                .iter()
                .map(|date| format!("{}T{}00", compact_date(date), time))
                .collect::<Vec<_>>()
                .join(",")
        ));
    }
}

fn mutation_from_event(
    event: &CalendarEvent,
    include_recurrence: bool,
) -> Result<GoogleEventMutation, String> {
    let time_zone = match event.recurrence.as_ref() {
        Some(EventRecurrence::Google { time_zone, .. }) if !time_zone.is_empty() => {
            time_zone.clone()
        }
        _ => "Asia/Tokyo".into(),
    };
    let (start, end) = if event.all_day {
        (
            GoogleEventDateTimeMutation {
                date: Some(event.date.clone()),
                date_time: None,
                time_zone: None,
            },
            GoogleEventDateTimeMutation {
                date: Some(add_one_day(&event.end_date)?),
                date_time: None,
                time_zone: None,
            },
        )
    } else {
        (
            GoogleEventDateTimeMutation {
                date: None,
                date_time: Some(format!("{}T{}:00", event.date, event.start_time)),
                time_zone: Some(time_zone.clone()),
            },
            GoogleEventDateTimeMutation {
                date: None,
                date_time: Some(format!("{}T{}:00", event.end_date, event.end_time)),
                time_zone: Some(time_zone),
            },
        )
    };
    let mut private = std::collections::BTreeMap::new();
    private.insert("koyomadoId".into(), event.id.clone());
    private.insert("koyomadoDataVersion".into(), "5".into());
    Ok(GoogleEventMutation {
        summary: event.title.clone(),
        description: event.notes.clone(),
        location: event.location.clone(),
        start,
        end,
        recurrence: include_recurrence
            .then(|| recurrence_lines(event))
            .transpose()?,
        reminders: google_reminders_from_local(&event.reminders),
        extended_properties: ExtendedPropertiesMutation { private },
    })
}

fn google_reminders_from_local(reminders: &EventReminders) -> GoogleEventReminders {
    if reminders.use_google_default {
        return GoogleEventReminders::default();
    }
    let mut overrides = reminders
        .popup_minutes
        .iter()
        .map(|minutes| GoogleEventReminderOverride {
            method: "popup".into(),
            minutes: *minutes,
        })
        .chain(
            reminders
                .email_minutes
                .iter()
                .map(|minutes| GoogleEventReminderOverride {
                    method: "email".into(),
                    minutes: *minutes,
                }),
        )
        .collect::<Vec<_>>();
    overrides.sort_by(|left, right| {
        left.minutes
            .cmp(&right.minutes)
            .then_with(|| left.method.cmp(&right.method))
    });
    GoogleEventReminders {
        use_default: false,
        overrides,
    }
}

fn local_reminders_from_google(reminders: &GoogleEventReminders) -> EventReminders {
    let valid_overrides = reminders
        .overrides
        .iter()
        .filter(|reminder| {
            (reminder.method == "popup" || reminder.method == "email") && reminder.minutes <= 40_320
        })
        .take(5)
        .collect::<Vec<_>>();
    let mut popup_minutes = valid_overrides
        .iter()
        .filter(|reminder| reminder.method == "popup")
        .map(|reminder| reminder.minutes)
        .collect::<Vec<_>>();
    let mut email_minutes = valid_overrides
        .iter()
        .filter(|reminder| reminder.method == "email")
        .map(|reminder| reminder.minutes)
        .collect::<Vec<_>>();
    popup_minutes.sort_unstable();
    popup_minutes.dedup();
    email_minutes.sort_unstable();
    email_minutes.dedup();
    EventReminders {
        use_google_default: reminders.use_default,
        popup_minutes,
        email_minutes,
    }
}

fn stable_local_event_id(account_id: &str, event_id: &str) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in account_id
        .bytes()
        .chain(std::iter::once(0xff))
        .chain(event_id.bytes())
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("google-{hash:016x}")
}

fn remote_matches_local(remote: &GoogleApiEvent, local: &CalendarEvent) -> bool {
    let all_day = remote.start.date.is_some();
    let expected_end_date = if all_day {
        add_one_day(&local.end_date).ok()
    } else {
        Some(local.end_date.clone())
    };
    let mut remote_reminders = remote.reminders.clone();
    remote_reminders.overrides.sort_by(|left, right| {
        left.minutes
            .cmp(&right.minutes)
            .then_with(|| left.method.cmp(&right.method))
    });
    date_from_google(&remote.start).as_deref() == Some(local.date.as_str())
        && date_from_google(&remote.end) == expected_end_date
        && all_day == local.all_day
        && (all_day || time_from_google(&remote.start) == local.start_time)
        && (all_day || time_from_google(&remote.end) == local.end_time)
        && remote.summary == local.title
        && remote.description == local.notes
        && remote.location == local.location
        && remote_reminders == google_reminders_from_local(&local.reminders)
}

fn google_link(
    account: &GoogleAccount,
    event: &GoogleApiEvent,
    local_updated_at: &str,
) -> GoogleEventLink {
    GoogleEventLink {
        account_id: account.id.clone(),
        calendar_id: account.calendar_id.clone(),
        event_id: event.id.clone(),
        etag: event.etag.clone(),
        google_updated_at: event.updated.clone(),
        local_updated_at: local_updated_at.to_string(),
        recurring_event_id: event.recurring_event_id.clone(),
        original_start: event
            .original_start_time
            .as_ref()
            .and_then(date_from_google),
    }
}

fn local_event_from_google(
    account: &GoogleAccount,
    remote: &GoogleApiEvent,
    master_id: Option<String>,
) -> Result<CalendarEvent, String> {
    let date = date_from_google(&remote.start)
        .ok_or_else(|| format!("Google予定「{}」の開始日を確認できません", remote.summary))?;
    let all_day = remote.start.date.is_some();
    let remote_end_date = date_from_google(&remote.end)
        .ok_or_else(|| format!("Google予定「{}」の終了日を確認できません", remote.summary))?;
    let end_date = if all_day {
        shift_date(&remote_end_date, -1)?
    } else {
        remote_end_date
    };
    let end_date = if end_date < date {
        date.clone()
    } else {
        end_date
    };
    let start_time = if all_day {
        String::new()
    } else {
        time_from_google(&remote.start)
    };
    let end_time = if all_day {
        String::new()
    } else {
        time_from_google(&remote.end)
    };
    let recurrence = (!remote.recurrence.is_empty()).then(|| EventRecurrence::Google {
        lines: remote.recurrence.clone(),
        time_zone: remote
            .start
            .time_zone
            .clone()
            .unwrap_or_else(|| "Asia/Tokyo".into()),
        excluded_dates: Vec::new(),
    });
    let annual = remote
        .recurrence
        .iter()
        .any(|line| line.contains("FREQ=YEARLY"));
    let recurrence_exception = master_id.map(|master_id| RecurrenceException {
        master_id,
        original_date: remote
            .original_start_time
            .as_ref()
            .and_then(date_from_google)
            .unwrap_or_else(|| date.clone()),
    });
    let updated_at = if remote.updated.is_empty() {
        Utc::now().to_rfc3339()
    } else {
        remote.updated.clone()
    };
    Ok(CalendarEvent {
        id: stable_local_event_id(&account.id, &remote.id),
        title: if remote.summary.trim().is_empty() {
            "（無題の予定）".into()
        } else {
            remote.summary.clone()
        },
        date,
        end_date,
        annual,
        recurrence,
        recurrence_exception,
        all_day,
        start_time,
        end_time,
        location: remote.location.clone(),
        notes: remote.description.clone(),
        reminders: local_reminders_from_google(&remote.reminders),
        style: EventStyle {
            color: "#83a9c2".into(),
        },
        origin: EventOrigin::Google {
            account_id: account.id.clone(),
        },
        sync_targets: vec![account.id.clone()],
        google_links: vec![google_link(account, remote, &updated_at)],
        sync_conflict: None,
        created_at: updated_at.clone(),
        updated_at,
    })
}

fn get_events_page(
    access_token: &str,
    calendar_id: &str,
    sync_token: Option<&str>,
    page_token: Option<&str>,
) -> Result<EventsListResponse, EventsPageError> {
    let client = api_client().map_err(EventsPageError::Other)?;
    let url = calendar_api_url(calendar_id, &["events"]).map_err(EventsPageError::Other)?;
    let mut request = client.get(url).bearer_auth(access_token).query(&[
        ("maxResults", "2500"),
        ("showDeleted", "true"),
        ("singleEvents", "false"),
    ]);
    if let Some(token) = sync_token {
        request = request.query(&[("syncToken", token)]);
    }
    if let Some(token) = page_token {
        request = request.query(&[("pageToken", token)]);
    }
    let response = request
        .send()
        .map_err(|error| EventsPageError::Other(format!("Google予定を取得できません: {error}")))?;
    if response.status() == reqwest::StatusCode::GONE {
        return Err(EventsPageError::Gone);
    }
    parse_google_response(response).map_err(EventsPageError::Other)
}

fn list_changed_events(
    access_token: &str,
    calendar_id: &str,
    sync_token: &str,
) -> Result<(Vec<GoogleApiEvent>, String), String> {
    fn run(
        access_token: &str,
        calendar_id: &str,
        sync_token: Option<&str>,
    ) -> Result<(Vec<GoogleApiEvent>, String), EventsPageError> {
        let mut events = Vec::new();
        let mut page_token: Option<String> = None;
        loop {
            let page =
                get_events_page(access_token, calendar_id, sync_token, page_token.as_deref())?;
            events.extend(page.items);
            if let Some(next) = page.next_page_token {
                page_token = Some(next);
            } else {
                return Ok((events, page.next_sync_token.unwrap_or_default()));
            }
        }
    }

    match run(
        access_token,
        calendar_id,
        (!sync_token.is_empty()).then_some(sync_token),
    ) {
        Ok(result) => Ok(result),
        Err(EventsPageError::Gone) => {
            run(access_token, calendar_id, None).map_err(|error| match error {
                EventsPageError::Gone => "Googleの同期状態を再初期化できません".into(),
                EventsPageError::Other(message) => message,
            })
        }
        Err(EventsPageError::Other(message)) => Err(message),
    }
}

fn create_remote_event(
    access_token: &str,
    account: &GoogleAccount,
    event: &CalendarEvent,
) -> Result<GoogleApiEvent, String> {
    let url = calendar_api_url(&account.calendar_id, &["events"])?;
    let response = api_client()?
        .post(url)
        .bearer_auth(access_token)
        .json(&mutation_from_event(event, true)?)
        .send()
        .map_err(|error| format!("Googleへ予定を追加できません: {error}"))?;
    parse_google_response(response)
}

fn patch_remote_event(
    access_token: &str,
    account: &GoogleAccount,
    event_id: &str,
    event: &CalendarEvent,
    include_recurrence: bool,
) -> Result<GoogleApiEvent, String> {
    let url = calendar_api_url(&account.calendar_id, &["events", event_id])?;
    let response = api_client()?
        .patch(url)
        .bearer_auth(access_token)
        .json(&mutation_from_event(event, include_recurrence)?)
        .send()
        .map_err(|error| format!("Googleの予定を更新できません: {error}"))?;
    parse_google_response(response)
}

fn delete_remote_event(
    access_token: &str,
    account: &GoogleAccount,
    event_id: &str,
) -> Result<(), String> {
    let url = calendar_api_url(&account.calendar_id, &["events", event_id])?;
    let response = api_client()?
        .delete(url)
        .bearer_auth(access_token)
        .send()
        .map_err(|error| format!("Googleの予定を削除できません: {error}"))?;
    if response.status().is_success()
        || response.status() == reqwest::StatusCode::NOT_FOUND
        || response.status() == reqwest::StatusCode::GONE
    {
        return Ok(());
    }
    let status = response.status();
    let body = response.text().unwrap_or_default();
    let message = serde_json::from_str::<serde_json::Value>(&body)
        .ok()
        .and_then(|value| {
            value
                .pointer("/error/message")
                .and_then(|item| item.as_str())
                .map(str::to_string)
        })
        .unwrap_or_else(|| format!("HTTP {status}"));
    Err(format!("Googleの予定を削除できません: {message}"))
}

fn shift_date(date: &str, days: i64) -> Result<String, String> {
    let parsed = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|_| format!("日付の形式が正しくありません: {date}"))?;
    Ok((parsed + chrono::Duration::days(days))
        .format("%Y-%m-%d")
        .to_string())
}

fn find_remote_instance(
    access_token: &str,
    account: &GoogleAccount,
    recurring_event_id: &str,
    original_date: &str,
) -> Result<Option<GoogleApiEvent>, String> {
    let url = calendar_api_url(
        &account.calendar_id,
        &["events", recurring_event_id, "instances"],
    )?;
    let time_min = format!("{}T00:00:00Z", shift_date(original_date, -1)?);
    let time_max = format!("{}T23:59:59Z", shift_date(original_date, 1)?);
    let page: EventsListResponse = parse_google_response(
        api_client()?
            .get(url)
            .bearer_auth(access_token)
            .query(&[
                ("timeMin", time_min.as_str()),
                ("timeMax", time_max.as_str()),
                ("showDeleted", "true"),
                ("maxResults", "50"),
            ])
            .send()
            .map_err(|error| format!("Googleの繰り返し予定を確認できません: {error}"))?,
    )?;
    Ok(page.items.into_iter().find(|item| {
        item.original_start_time
            .as_ref()
            .and_then(date_from_google)
            .as_deref()
            == Some(original_date)
    }))
}

fn update_link(event: &mut CalendarEvent, account: &GoogleAccount, link: GoogleEventLink) {
    event
        .google_links
        .retain(|item| item.account_id != account.id || item.calendar_id != account.calendar_id);
    event.google_links.push(link);
}

fn remote_link_index(
    event: &CalendarEvent,
    account: &GoogleAccount,
    event_id: &str,
) -> Option<usize> {
    event.google_links.iter().position(|link| {
        link.account_id == account.id
            && link.calendar_id == account.calendar_id
            && link.event_id == event_id
    })
}

fn strip_account_link(event: &mut CalendarEvent, account_id: &str) {
    event
        .google_links
        .retain(|link| link.account_id != account_id);
    event.sync_targets.retain(|target| target != account_id);
    if matches!(&event.origin, EventOrigin::Google { account_id: origin } if origin == account_id) {
        event.origin = EventOrigin::Local;
    }
}

fn mark_conflict(event: &mut CalendarEvent, account: &GoogleAccount, reason: &str, message: &str) {
    event.sync_conflict = Some(SyncConflict {
        account_id: account.id.clone(),
        detected_at: Utc::now().to_rfc3339(),
        reason: reason.into(),
        message: message.into(),
    });
}

fn find_master_local_id(
    data: &AppData,
    account: &GoogleAccount,
    remote_master_id: &str,
) -> Option<String> {
    data.events.iter().find_map(|event| {
        event
            .google_links
            .iter()
            .any(|link| {
                link.account_id == account.id
                    && link.calendar_id == account.calendar_id
                    && link.event_id == remote_master_id
            })
            .then(|| event.id.clone())
    })
}

fn handle_remote_cancellation(
    data: &mut AppData,
    account: &GoogleAccount,
    remote: &GoogleApiEvent,
    summary: &mut GoogleSyncSummary,
) {
    let existing_index = data
        .events
        .iter()
        .position(|event| remote_link_index(event, account, &remote.id).is_some());
    if let Some(index) = existing_index {
        let link = data.events[index]
            .google_links
            .iter()
            .find(|link| link.account_id == account.id && link.event_id == remote.id)
            .cloned();
        let local_changed = link
            .as_ref()
            .is_some_and(|link| data.events[index].updated_at != link.local_updated_at);
        if local_changed {
            let retained_master_id = data.events[index].id.clone();
            strip_account_link(&mut data.events[index], &account.id);
            mark_conflict(
                &mut data.events[index],
                account,
                "deleted-on-google",
                "Google側で削除されましたが、Koyomado側にも未同期の変更があるためローカル予定として残しました。",
            );
            for exception in data.events.iter_mut().filter(|event| {
                event
                    .recurrence_exception
                    .as_ref()
                    .is_some_and(|item| item.master_id == retained_master_id)
            }) {
                strip_account_link(exception, &account.id);
            }
            summary.conflicts += 1;
            return;
        }
        let removed = data.events.remove(index);
        let master_id = removed.id.clone();
        if let Some(exception) = removed.recurrence_exception.as_ref() {
            if let Some(master) = data
                .events
                .iter_mut()
                .find(|event| event.id == exception.master_id)
            {
                match master.recurrence.as_mut() {
                    Some(EventRecurrence::Simple { excluded_dates, .. })
                    | Some(EventRecurrence::Google { excluded_dates, .. })
                        if !excluded_dates.contains(&exception.original_date) =>
                    {
                        excluded_dates.push(exception.original_date.clone());
                    }
                    Some(_) | None => {}
                }
            }
        }

        let mut removed_exceptions = Vec::new();
        if removed.recurrence.is_some() {
            let mut retained = Vec::with_capacity(data.events.len());
            for mut event in data.events.drain(..) {
                if event
                    .recurrence_exception
                    .as_ref()
                    .is_some_and(|item| item.master_id == master_id)
                {
                    let account_link = event.google_links.iter().find(|link| {
                        link.account_id == account.id && link.calendar_id == account.calendar_id
                    });
                    let exception_changed =
                        account_link.is_some_and(|link| event.updated_at != link.local_updated_at);
                    if exception_changed {
                        strip_account_link(&mut event, &account.id);
                        event.recurrence_exception = None;
                        mark_conflict(
                            &mut event,
                            account,
                            "deleted-on-google",
                            "Google側で繰り返し全体が削除されましたが、この回には未同期の変更があるためローカル予定として残しました。",
                        );
                        summary.conflicts += 1;
                        retained.push(event);
                    } else {
                        removed_exceptions.push(event);
                    }
                } else {
                    retained.push(event);
                }
            }
            data.events = retained;
        }
        let mut tombstone = removed;
        strip_account_link(&mut tombstone, &account.id);
        data.deleted_events.push(DeletedCalendarEvent {
            event: tombstone,
            deleted_at: Utc::now().to_rfc3339(),
        });
        let removed_exception_count = removed_exceptions.len();
        for mut exception in removed_exceptions {
            strip_account_link(&mut exception, &account.id);
            data.deleted_events.push(DeletedCalendarEvent {
                event: exception,
                deleted_at: Utc::now().to_rfc3339(),
            });
        }
        summary.deleted += 1 + removed_exception_count;
        return;
    }

    if let (Some(remote_master), Some(original_date)) = (
        remote.recurring_event_id.as_deref(),
        remote
            .original_start_time
            .as_ref()
            .and_then(date_from_google),
    ) {
        if let Some(master_id) = find_master_local_id(data, account, remote_master) {
            if let Some(master) = data.events.iter_mut().find(|event| event.id == master_id) {
                match master.recurrence.as_mut() {
                    Some(EventRecurrence::Simple { excluded_dates, .. })
                    | Some(EventRecurrence::Google { excluded_dates, .. })
                        if !excluded_dates.contains(&original_date) =>
                    {
                        excluded_dates.push(original_date);
                    }
                    Some(_) | None => {}
                }
            }
        }
    }
}

fn merge_remote_event(
    data: &mut AppData,
    account: &GoogleAccount,
    remote: &GoogleApiEvent,
    summary: &mut GoogleSyncSummary,
) -> Result<(), String> {
    if remote.status == "cancelled" {
        handle_remote_cancellation(data, account, remote, summary);
        return Ok(());
    }
    let existing_index = data
        .events
        .iter()
        .position(|event| remote_link_index(event, account, &remote.id).is_some());
    let master_id = remote
        .recurring_event_id
        .as_deref()
        .and_then(|master| find_master_local_id(data, account, master));

    if let Some(index) = existing_index {
        let link_index = remote_link_index(&data.events[index], account, &remote.id)
            .ok_or_else(|| "Google予定のリンク情報を確認できません".to_string())?;
        let previous_link = data.events[index].google_links[link_index].clone();
        let local_changed = data.events[index].updated_at != previous_link.local_updated_at;
        let remote_changed =
            remote.updated != previous_link.google_updated_at || remote.etag != previous_link.etag;
        if local_changed && remote_changed {
            let mut local = data.events[index].clone();
            let local_series_id = local.id.clone();
            strip_account_link(&mut local, &account.id);
            mark_conflict(
                &mut local,
                account,
                "both-edited",
                "KoyomadoとGoogleの両方で変更されたため、2件に分けて残しました。",
            );
            data.events[index] = local;
            for exception in data.events.iter_mut().filter(|event| {
                event
                    .recurrence_exception
                    .as_ref()
                    .is_some_and(|item| item.master_id == local_series_id)
            }) {
                strip_account_link(exception, &account.id);
            }

            let mut remote_copy = local_event_from_google(account, remote, master_id)?;
            remote_copy.id = format!(
                "{}-conflict-{}",
                remote_copy.id,
                Utc::now().timestamp_millis()
            );
            mark_conflict(
                &mut remote_copy,
                account,
                "both-edited",
                "Google側の内容です。不要な方を削除するか、必要な内容へ編集してください。",
            );
            data.events.push(remote_copy);
            summary.conflicts += 1;
        } else if remote_changed {
            let current = data.events[index].clone();
            let mut merged = local_event_from_google(account, remote, master_id)?;
            merged.id = current.id;
            merged.style = current.style;
            merged.origin = current.origin;
            merged.sync_targets = current.sync_targets;
            merged.created_at = current.created_at;
            merged.sync_conflict = current.sync_conflict;
            merged.google_links = current.google_links;
            let local_updated_at = merged.updated_at.clone();
            update_link(
                &mut merged,
                account,
                google_link(account, remote, &local_updated_at),
            );
            data.events[index] = merged;
            summary.pulled += 1;
        }
    } else {
        let koyomado_id = remote
            .extended_properties
            .private
            .get("koyomadoId")
            .filter(|id| !id.is_empty());
        let recovery_index = koyomado_id.and_then(|id| {
            data.events.iter().position(|event| {
                event.id == *id
                    && event.sync_targets.contains(&account.id)
                    && !event
                        .google_links
                        .iter()
                        .any(|link| link.account_id == account.id)
                    && remote_matches_local(remote, event)
            })
        });
        if let Some(index) = recovery_index {
            let local_updated_at = data.events[index].updated_at.clone();
            update_link(
                &mut data.events[index],
                account,
                google_link(account, remote, &local_updated_at),
            );
        } else {
            let imported = local_event_from_google(account, remote, master_id)?;
            data.events.push(imported);
            summary.pulled += 1;
        }
    }
    Ok(())
}

fn pull_remote_events(
    data: &mut AppData,
    account: &GoogleAccount,
    access_token: &str,
    summary: &mut GoogleSyncSummary,
) -> Result<String, String> {
    let (remote_events, next_sync_token) =
        list_changed_events(access_token, &account.calendar_id, &account.sync_token)?;
    for remote in remote_events
        .iter()
        .filter(|event| event.recurring_event_id.is_none())
    {
        merge_remote_event(data, account, remote, summary)?;
    }
    for remote in remote_events
        .iter()
        .filter(|event| event.recurring_event_id.is_some())
    {
        merge_remote_event(data, account, remote, summary)?;
    }
    Ok(next_sync_token)
}

fn delete_removed_targets(
    data: &mut AppData,
    account: &GoogleAccount,
    access_token: &str,
    summary: &mut GoogleSyncSummary,
) -> Result<(), String> {
    for event in &mut data.events {
        let links = event
            .google_links
            .iter()
            .filter(|link| {
                link.account_id == account.id
                    && link.calendar_id == account.calendar_id
                    && !link.event_id.is_empty()
            })
            .cloned()
            .collect::<Vec<_>>();
        if event.sync_targets.contains(&account.id) {
            continue;
        }
        for link in links {
            delete_remote_event(access_token, account, &link.event_id)?;
            event
                .google_links
                .retain(|item| !(item.account_id == account.id && item.event_id == link.event_id));
            summary.deleted += 1;
        }
    }
    Ok(())
}

fn push_local_deletions(
    data: &mut AppData,
    account: &GoogleAccount,
    access_token: &str,
    summary: &mut GoogleSyncSummary,
) -> Result<(), String> {
    for deleted in &mut data.deleted_events {
        let links = deleted
            .event
            .google_links
            .iter()
            .filter(|link| link.account_id == account.id && link.calendar_id == account.calendar_id)
            .cloned()
            .collect::<Vec<_>>();
        for link in links {
            let event_id = if !link.event_id.is_empty() {
                Some(link.event_id.clone())
            } else if let (Some(master), Some(original_date)) = (
                link.recurring_event_id.as_deref(),
                link.original_start.as_deref(),
            ) {
                find_remote_instance(access_token, account, master, original_date)?
                    .map(|event| event.id)
            } else {
                None
            };
            if let Some(event_id) = event_id {
                delete_remote_event(access_token, account, &event_id)?;
                summary.deleted += 1;
            }
            deleted.event.google_links.retain(|item| {
                !(item.account_id == account.id
                    && item.calendar_id == account.calendar_id
                    && item.event_id == link.event_id)
            });
        }
    }
    Ok(())
}

fn push_local_events(
    data: &mut AppData,
    account: &GoogleAccount,
    access_token: &str,
    summary: &mut GoogleSyncSummary,
) -> Result<(), String> {
    for index in 0..data.events.len() {
        let event = data.events[index].clone();
        if !event.sync_targets.contains(&account.id) {
            continue;
        }
        let existing_link = event
            .google_links
            .iter()
            .find(|link| link.account_id == account.id && link.calendar_id == account.calendar_id)
            .cloned();
        let response = match existing_link.as_ref() {
            Some(link) if link.event_id.is_empty() => {
                let master_id = link
                    .recurring_event_id
                    .as_deref()
                    .ok_or_else(|| "Googleの繰り返し予定との対応を確認できません".to_string())?;
                let original_date = link
                    .original_start
                    .as_deref()
                    .or_else(|| {
                        event
                            .recurrence_exception
                            .as_ref()
                            .map(|item| item.original_date.as_str())
                    })
                    .ok_or_else(|| "繰り返し予定の元の日付を確認できません".to_string())?;
                let instance =
                    find_remote_instance(access_token, account, master_id, original_date)?
                        .ok_or_else(|| "Google側の対象回を確認できません".to_string())?;
                Some(patch_remote_event(
                    access_token,
                    account,
                    &instance.id,
                    &event,
                    false,
                )?)
            }
            Some(link) if event.updated_at != link.local_updated_at => Some(patch_remote_event(
                access_token,
                account,
                &link.event_id,
                &event,
                event.recurrence_exception.is_none(),
            )?),
            Some(_) => None,
            None => {
                if let Some(exception) = event.recurrence_exception.as_ref() {
                    let master_link = data
                        .events
                        .iter()
                        .find(|item| item.id == exception.master_id)
                        .and_then(|master| {
                            master.google_links.iter().find(|link| {
                                link.account_id == account.id
                                    && link.calendar_id == account.calendar_id
                                    && !link.event_id.is_empty()
                            })
                        });
                    if let Some(master_link) = master_link {
                        let instance = find_remote_instance(
                            access_token,
                            account,
                            &master_link.event_id,
                            &exception.original_date,
                        )?
                        .ok_or_else(|| {
                            "Google側の繰り返し予定に対象回が見つかりません".to_string()
                        })?;
                        Some(patch_remote_event(
                            access_token,
                            account,
                            &instance.id,
                            &event,
                            false,
                        )?)
                    } else {
                        None
                    }
                } else {
                    Some(create_remote_event(access_token, account, &event)?)
                }
            }
        };
        if let Some(remote) = response {
            let local_updated_at = data.events[index].updated_at.clone();
            let link = google_link(account, &remote, &local_updated_at);
            update_link(&mut data.events[index], account, link);
            summary.pushed += 1;
        }
    }
    Ok(())
}

fn sync_account(
    data: &mut AppData,
    account: &GoogleAccount,
    access_token: &str,
    summary: &mut GoogleSyncSummary,
) -> Result<String, String> {
    let next_sync_token = pull_remote_events(data, account, access_token, summary)?;
    delete_removed_targets(data, account, access_token, summary)?;
    push_local_deletions(data, account, access_token, summary)?;
    push_local_events(data, account, access_token, summary)?;
    Ok(next_sync_token)
}

fn is_reauthentication_error(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("認証情報がありません")
        || lower.contains("invalid_grant")
        || lower.contains("invalid_client")
        || lower.contains("unauthorized_client")
        || lower.contains("401 unauthorized")
}

fn sync_all_blocking() -> Result<GoogleSyncResult, String> {
    let _guard = GOOGLE_SYNC_LOCK
        .lock()
        .map_err(|_| "Google同期の排他制御を開始できません".to_string())?;
    let _data_guard = APP_DATA_LOCK
        .lock()
        .map_err(|_| "予定データの同期を開始できません".to_string())?;
    let mut data = load_app_data_inner()?;
    let mut summary = GoogleSyncSummary::default();

    if !data.settings.google.enabled {
        return Ok(GoogleSyncResult { data, summary });
    }
    let Some(client) = data.settings.google.client.clone() else {
        summary
            .warnings
            .push("OAuthクライアント設定がないため同期を開始できませんでした".into());
        return Ok(GoogleSyncResult { data, summary });
    };

    let accounts = data.settings.google.accounts.clone();
    if accounts.len() > 3 {
        return Err("Googleアカウントは3件まで接続できます".into());
    }

    for account in accounts.into_iter().filter(|account| account.sync_enabled) {
        let result = refresh_access_token(&client, &account.id).and_then(|access_token| {
            sync_account(&mut data, &account, &access_token, &mut summary)
        });
        let now = Utc::now().to_rfc3339();
        let Some(stored_account) = data
            .settings
            .google
            .accounts
            .iter_mut()
            .find(|stored| stored.id == account.id)
        else {
            continue;
        };

        match result {
            Ok(next_sync_token) => {
                stored_account.sync_token = next_sync_token;
                stored_account.last_sync_at = now;
                stored_account.last_error.clear();
                stored_account.needs_reauth = false;
                summary.accounts_synced += 1;
            }
            Err(message) => {
                stored_account.last_error = message.clone();
                stored_account.needs_reauth = is_reauthentication_error(&message);
                summary
                    .warnings
                    .push(format!("{}: {message}", account.email));
            }
        }
    }

    write_json_with_backup(&portable_data_dir()?.join("calendar-data.json"), &data)?;
    Ok(GoogleSyncResult { data, summary })
}

#[tauri::command]
pub(crate) async fn google_sync_all() -> Result<GoogleSyncResult, String> {
    tauri::async_runtime::spawn_blocking(sync_all_blocking)
        .await
        .map_err(|error| format!("Google同期処理を完了できません: {error}"))?
}

fn connect_account_blocking(config: GoogleOAuthClient) -> Result<GoogleConnectionResult, String> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|error| format!("Google認証用のローカルポートを開けません: {error}"))?;
    let port = listener
        .local_addr()
        .map_err(|error| format!("Google認証用ポートを確認できません: {error}"))?
        .port();
    let redirect = format!("http://127.0.0.1:{port}");
    let client = oauth_client(&config, Some(redirect))?;
    let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
    let (authorization_url, csrf) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".into()))
        .add_scope(Scope::new("email".into()))
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/calendar.events".into(),
        ))
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/calendar.calendarlist.readonly".into(),
        ))
        .add_extra_param("access_type", "offline")
        .add_extra_param("prompt", "consent select_account")
        .set_pkce_challenge(challenge)
        .url();
    open_browser(authorization_url.as_str())?;
    let code = wait_for_authorization(listener, csrf.secret())?;
    let token = client
        .exchange_code(AuthorizationCode::new(code))
        .set_pkce_verifier(verifier)
        .request(&api_client()?)
        .map_err(|error| format!("Googleの認証コードを交換できません: {error}"))?;
    let refresh_token = token.refresh_token().ok_or_else(|| {
        "Googleから更新トークンを受け取れませんでした。接続を解除して再度お試しください".to_string()
    })?;
    let access_token = token.access_token().secret();
    let client = api_client()?;
    let user: UserInfoResponse = parse_google_response(
        client
            .get(GOOGLE_USER_INFO_URL)
            .bearer_auth(access_token)
            .send()
            .map_err(|error| format!("Googleアカウント情報を取得できません: {error}"))?,
    )?;
    let existing = APP_DATA_LOCK
        .lock()
        .map_err(|_| "接続済みGoogleアカウントを確認できません".to_string())
        .and_then(|_guard| load_app_data_inner())
        .map(|data| data.settings.google.accounts)
        .unwrap_or_default();
    if existing.len() >= 3 && !existing.iter().any(|account| account.id == user.id) {
        return Err("Googleアカウントは3件まで接続できます".into());
    }
    let calendars = list_calendars_with_token(access_token)?;
    let selected = calendars
        .iter()
        .find(|calendar| calendar.primary)
        .or_else(|| calendars.first())
        .ok_or_else(|| "編集可能なGoogleカレンダーがありません".to_string())?;
    credentials::store_refresh_token(&user.id, refresh_token.secret())?;
    let now = Utc::now().to_rfc3339();
    Ok(GoogleConnectionResult {
        account: GoogleAccount {
            id: user.id,
            email: user.email,
            display_name: user.name,
            calendar_id: selected.id.clone(),
            calendar_name: selected.name.clone(),
            sync_enabled: true,
            sync_token: String::new(),
            connected_at: now,
            last_sync_at: String::new(),
            last_error: String::new(),
            needs_reauth: false,
        },
        calendars,
    })
}

#[tauri::command]
pub(crate) async fn google_connect_account(
    client: GoogleOAuthClient,
) -> Result<GoogleConnectionResult, String> {
    tauri::async_runtime::spawn_blocking(move || connect_account_blocking(client))
        .await
        .map_err(|error| format!("Google接続処理を完了できません: {error}"))?
}

#[tauri::command]
pub(crate) async fn google_list_calendars(
    client: GoogleOAuthClient,
    account_id: String,
) -> Result<Vec<GoogleCalendarOption>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let token = refresh_access_token(&client, &account_id)?;
        list_calendars_with_token(&token)
    })
    .await
    .map_err(|error| format!("Googleカレンダー一覧の取得を完了できません: {error}"))?
}

#[tauri::command]
pub(crate) fn google_credential_statuses(
    account_ids: Vec<String>,
) -> Result<Vec<CredentialStatus>, String> {
    account_ids
        .into_iter()
        .map(|account_id| {
            let available = credentials::read_refresh_token(&account_id)?.is_some();
            Ok(CredentialStatus {
                account_id,
                available,
            })
        })
        .collect()
}

#[tauri::command]
pub(crate) async fn google_disconnect_account(
    account_id: String,
) -> Result<DisconnectResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let refresh_token = credentials::read_refresh_token(&account_id)?;
        let revoked = if let Some(token) = refresh_token.as_deref() {
            api_client()?
                .post(GOOGLE_REVOKE_URL)
                .form(&[("token", token)])
                .send()
                .map(|response| response.status().is_success())
                .unwrap_or(false)
        } else {
            false
        };
        credentials::delete_refresh_token(&account_id)?;
        Ok(DisconnectResult {
            revoked,
            message: if revoked {
                "Googleとの接続を解除しました".into()
            } else {
                "このPCの認証情報を削除しました。必要に応じてGoogleアカウント側でもアクセス権を解除してください".into()
            },
        })
    })
    .await
    .map_err(|error| format!("Google接続の解除を完了できません: {error}"))?
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account() -> GoogleAccount {
        GoogleAccount {
            id: "account-1".into(),
            email: "user@example.invalid".into(),
            display_name: "利用者".into(),
            calendar_id: "primary".into(),
            calendar_name: "メイン".into(),
            sync_enabled: true,
            sync_token: String::new(),
            connected_at: String::new(),
            last_sync_at: String::new(),
            last_error: String::new(),
            needs_reauth: false,
        }
    }

    fn local_event(id: &str, date: &str) -> CalendarEvent {
        CalendarEvent {
            id: id.into(),
            title: "予定".into(),
            date: date.into(),
            end_date: date.into(),
            annual: false,
            recurrence: None,
            recurrence_exception: None,
            all_day: true,
            start_time: String::new(),
            end_time: String::new(),
            location: String::new(),
            notes: String::new(),
            reminders: EventReminders::default(),
            style: EventStyle {
                color: "#78a88f".into(),
            },
            origin: EventOrigin::Local,
            sync_targets: Vec::new(),
            google_links: Vec::new(),
            sync_conflict: None,
            created_at: "2026-01-01T00:00:00Z".into(),
            updated_at: "2026-01-01T00:00:00Z".into(),
        }
    }

    fn remote_event(id: &str, date: &str) -> GoogleApiEvent {
        GoogleApiEvent {
            id: id.into(),
            status: "confirmed".into(),
            etag: "etag-2".into(),
            summary: "Google側の予定".into(),
            description: String::new(),
            location: String::new(),
            start: GoogleEventDateTime {
                date: Some(date.into()),
                ..Default::default()
            },
            end: GoogleEventDateTime {
                date: Some(add_one_day(date).unwrap()),
                ..Default::default()
            },
            recurrence: Vec::new(),
            reminders: GoogleEventReminders::default(),
            recurring_event_id: None,
            original_start_time: None,
            updated: "2026-01-02T00:00:00Z".into(),
            extended_properties: GoogleExtendedProperties::default(),
        }
    }

    #[test]
    fn oauth_client_rejects_empty_client_id() {
        let error = oauth_client(
            &GoogleOAuthClient {
                client_id: String::new(),
                client_secret: String::new(),
                project_id: String::new(),
            },
            None,
        )
        .expect_err("empty client id must fail");
        assert!(error.contains("クライアントID"));
    }

    #[test]
    fn callback_rejects_wrong_state_without_exposing_code() {
        let callback = Url::parse("http://127.0.0.1/?state=wrong&code=sensitive").unwrap();
        let state = callback
            .query_pairs()
            .find(|(key, _)| key == "state")
            .map(|(_, value)| value.into_owned());
        assert_ne!(state.as_deref(), Some("expected"));
    }

    #[test]
    fn simple_yearly_recurrence_is_sent_as_google_rrule() {
        let mut event = local_event("birthday", "2026-07-22");
        event.recurrence = Some(EventRecurrence::Simple {
            frequency: "yearly".into(),
            interval: 1,
            week_days: Vec::new(),
            monthly_mode: "day-of-month".into(),
            end: RecurrenceEnd::Never,
            excluded_dates: vec!["2027-07-22".into()],
        });
        let lines = recurrence_lines(&event).unwrap();
        assert_eq!(lines[0], "RRULE:FREQ=YEARLY");
        assert_eq!(lines[1], "EXDATE;VALUE=DATE:20270722");
    }

    #[test]
    fn all_day_mutation_uses_exclusive_google_end_date() {
        let mut event = local_event("holiday", "2026-07-22");
        event.end_date = "2026-07-24".into();
        let mutation = mutation_from_event(&event, true).unwrap();
        assert_eq!(mutation.start.date.as_deref(), Some("2026-07-22"));
        assert_eq!(mutation.end.date.as_deref(), Some("2026-07-25"));
    }

    #[test]
    fn explicit_popup_and_email_reminders_round_trip_with_google() {
        let mut event = local_event("reminders", "2026-07-22");
        event.reminders = EventReminders {
            use_google_default: false,
            popup_minutes: vec![10, 60],
            email_minutes: vec![1_440],
        };
        let mutation = mutation_from_event(&event, true).unwrap();
        assert!(!mutation.reminders.use_default);
        assert_eq!(mutation.reminders.overrides.len(), 3);
        assert!(mutation
            .reminders
            .overrides
            .iter()
            .any(|reminder| reminder.method == "popup" && reminder.minutes == 10));

        let mut remote = remote_event("remote-reminders", "2026-07-22");
        remote.reminders = mutation.reminders;
        let imported = local_event_from_google(&account(), &remote, None).unwrap();
        assert_eq!(imported.reminders, event.reminders);
    }

    #[test]
    fn local_popup_can_coexist_with_google_default_without_false_difference() {
        let mut event = local_event("local-popup", "2026-07-22");
        event.reminders.popup_minutes = vec![10];
        let mut remote = remote_event("remote-popup", "2026-07-22");
        remote.summary = event.title.clone();
        assert!(remote_matches_local(&remote, &event));
    }

    #[test]
    fn multi_day_google_event_imports_the_inclusive_local_end_date() {
        let mut remote = remote_event("trip", "2026-09-01");
        remote.end.date = Some("2026-09-04".into());
        let imported = local_event_from_google(&account(), &remote, None).unwrap();
        assert_eq!(imported.date, "2026-09-01");
        assert_eq!(imported.end_date, "2026-09-03");
    }

    #[test]
    fn overnight_mutation_uses_the_following_google_end_date() {
        let mut event = local_event("late-shift", "2026-07-22");
        event.all_day = false;
        event.start_time = "23:30".into();
        event.end_time = "00:30".into();
        event.end_date = "2026-07-23".into();
        let mutation = mutation_from_event(&event, true).unwrap();
        assert_eq!(
            mutation.start.date_time.as_deref(),
            Some("2026-07-22T23:30:00")
        );
        assert_eq!(
            mutation.end.date_time.as_deref(),
            Some("2026-07-23T00:30:00")
        );
    }

    #[test]
    fn edits_on_both_sides_are_preserved_as_two_conflicted_events() {
        let account = account();
        let mut local = local_event("local", "2026-07-22");
        local.title = "Koyomado側".into();
        local.updated_at = "2026-01-03T00:00:00Z".into();
        local.sync_targets.push(account.id.clone());
        local.google_links.push(GoogleEventLink {
            account_id: account.id.clone(),
            calendar_id: account.calendar_id.clone(),
            event_id: "remote".into(),
            etag: "etag-1".into(),
            google_updated_at: "2026-01-01T00:00:00Z".into(),
            local_updated_at: "2026-01-01T00:00:00Z".into(),
            recurring_event_id: None,
            original_start: None,
        });
        let mut data = AppData::default();
        data.events.push(local);
        let mut summary = GoogleSyncSummary::default();

        merge_remote_event(
            &mut data,
            &account,
            &remote_event("remote", "2026-07-23"),
            &mut summary,
        )
        .unwrap();

        assert_eq!(data.events.len(), 2);
        assert_eq!(summary.conflicts, 1);
        assert!(data
            .events
            .iter()
            .all(|event| event.sync_conflict.is_some()));
        assert!(data
            .events
            .iter()
            .any(|event| event.title == "Koyomado側" && event.google_links.is_empty()));
        assert!(data
            .events
            .iter()
            .any(|event| event.title == "Google側の予定" && !event.google_links.is_empty()));
    }

    #[test]
    fn cancelled_google_instance_excludes_only_that_occurrence() {
        let account = account();
        let mut master = local_event("master", "2026-07-01");
        master.recurrence = Some(EventRecurrence::Simple {
            frequency: "daily".into(),
            interval: 1,
            week_days: Vec::new(),
            monthly_mode: "day-of-month".into(),
            end: RecurrenceEnd::Never,
            excluded_dates: Vec::new(),
        });
        master.google_links.push(GoogleEventLink {
            account_id: account.id.clone(),
            calendar_id: account.calendar_id.clone(),
            event_id: "remote-master".into(),
            etag: String::new(),
            google_updated_at: String::new(),
            local_updated_at: master.updated_at.clone(),
            recurring_event_id: None,
            original_start: None,
        });
        let mut cancelled = remote_event("cancelled-instance", "2026-07-08");
        cancelled.status = "cancelled".into();
        cancelled.recurring_event_id = Some("remote-master".into());
        cancelled.original_start_time = Some(GoogleEventDateTime {
            date: Some("2026-07-08".into()),
            ..Default::default()
        });
        let mut data = AppData::default();
        data.events.push(master);
        let mut summary = GoogleSyncSummary::default();

        merge_remote_event(&mut data, &account, &cancelled, &mut summary).unwrap();

        let excluded = match data.events[0].recurrence.as_ref().unwrap() {
            EventRecurrence::Simple { excluded_dates, .. } => excluded_dates,
            _ => panic!("simple recurrence expected"),
        };
        assert_eq!(excluded, &["2026-07-08"]);
    }

    #[test]
    fn local_id_for_google_event_is_stable_and_account_scoped() {
        assert_eq!(
            stable_local_event_id("account-1", "event-1"),
            stable_local_event_id("account-1", "event-1")
        );
        assert_ne!(
            stable_local_event_id("account-1", "event-1"),
            stable_local_event_id("account-2", "event-1")
        );
    }

    #[test]
    fn interrupted_create_is_relinked_without_duplicate_import() {
        let account = account();
        let mut local = local_event("local-id", "2026-07-22");
        local.sync_targets.push(account.id.clone());
        let mut remote = remote_event("remote-id", "2026-07-22");
        remote.summary = local.title.clone();
        remote
            .extended_properties
            .private
            .insert("koyomadoId".into(), local.id.clone());
        let mut data = AppData::default();
        data.events.push(local);
        let mut summary = GoogleSyncSummary::default();

        merge_remote_event(&mut data, &account, &remote, &mut summary).unwrap();

        assert_eq!(data.events.len(), 1);
        assert_eq!(data.events[0].google_links[0].event_id, "remote-id");
        assert_eq!(summary.pulled, 0);
    }
}
