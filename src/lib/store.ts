import type {
  AppData,
  CalendarEvent,
  EventRecurrence,
  GoogleAccount,
  GoogleCalendarOption,
  GoogleOAuthClient,
  WindowDisplayMode,
} from "../types";
import { DATA_VERSION, DEFAULT_DATA, DEFAULT_SIMPLE_RECURRENCE } from "../types";

const WEB_STORAGE_KEY = "koyomado-preview-data-v1";
const LEGACY_WEB_STORAGE_KEY = "ytec-calendar-preview-data-v1";

export function isTauriRuntime(): boolean {
  return "__TAURI_INTERNALS__" in window;
}

function normalizeRecurrence(event: Partial<CalendarEvent>): EventRecurrence | null {
  if (event.recurrence && typeof event.recurrence === "object") {
    if (event.recurrence.kind === "google") {
      return {
        ...event.recurrence,
        lines: Array.isArray(event.recurrence.lines) ? event.recurrence.lines.filter((line) => typeof line === "string") : [],
        timeZone: event.recurrence.timeZone || "Asia/Tokyo",
        excludedDates: Array.isArray(event.recurrence.excludedDates) ? event.recurrence.excludedDates : [],
      };
    }
    if (event.recurrence.kind === "simple") {
      return {
        ...event.recurrence,
        interval: Math.max(1, Number(event.recurrence.interval) || 1),
        weekDays: Array.isArray(event.recurrence.weekDays) ? event.recurrence.weekDays.filter((day) => Number.isInteger(day) && day >= 0 && day <= 6) : [],
        excludedDates: Array.isArray(event.recurrence.excludedDates) ? event.recurrence.excludedDates : [],
      };
    }
  }
  return event.annual ? structuredClone(DEFAULT_SIMPLE_RECURRENCE) : null;
}

function normalizeEvent(value: unknown): CalendarEvent {
  const event = value as Partial<CalendarEvent>;
  const recurrence = normalizeRecurrence(event);
  const date = event.date ?? "";
  const endDate = event.endDate && event.endDate >= date ? event.endDate : date;
  return {
    ...(event as CalendarEvent),
    date,
    endDate,
    annual: recurrence?.kind === "simple" && recurrence.frequency === "yearly",
    recurrence,
    recurrenceException: event.recurrenceException ?? null,
    origin: event.origin ?? { kind: "local" },
    syncTargets: Array.isArray(event.syncTargets) ? event.syncTargets : [],
    googleLinks: Array.isArray(event.googleLinks) ? event.googleLinks : [],
    syncConflict: event.syncConflict ?? null,
  };
}

export function normalizeData(value: unknown): AppData {
  if (!value || typeof value !== "object") return structuredClone(DEFAULT_DATA);
  const candidate = value as Omit<Partial<AppData>, "version"> & { version?: number };
  if (![1, 2, DATA_VERSION].includes(candidate.version ?? -1) || !Array.isArray(candidate.events)) return structuredClone(DEFAULT_DATA);
  const google = candidate.settings?.google;
  return {
    version: DATA_VERSION,
    events: candidate.events.map(normalizeEvent),
    deletedEvents: Array.isArray(candidate.deletedEvents)
      ? candidate.deletedEvents.map((event) => ({ ...normalizeEvent(event), deletedAt: event.deletedAt }))
      : [],
    settings: {
      theme: candidate.settings?.theme ?? DEFAULT_DATA.settings.theme,
      sidebarCollapsed: candidate.settings?.sidebarCollapsed ?? false,
      windowDisplayMode: candidate.settings?.windowDisplayMode ?? "taskbar",
      google: {
        enabled: google?.enabled ?? false,
        client: google?.client ?? null,
        accounts: Array.isArray(google?.accounts) ? google.accounts.slice(0, 3) : [],
      },
    },
  };
}

export async function loadAppData(): Promise<AppData> {
  if (isTauriRuntime()) {
    const { invoke } = await import("@tauri-apps/api/core");
    return normalizeData(await invoke<AppData>("load_app_data"));
  }

  const stored = localStorage.getItem(WEB_STORAGE_KEY) ?? localStorage.getItem(LEGACY_WEB_STORAGE_KEY);
  if (!stored) return structuredClone(DEFAULT_DATA);
  try {
    return normalizeData(JSON.parse(stored));
  } catch {
    return structuredClone(DEFAULT_DATA);
  }
}

export async function saveAppData(data: AppData): Promise<void> {
  if (isTauriRuntime()) {
    const { invoke } = await import("@tauri-apps/api/core");
    await invoke("save_app_data", { data });
    return;
  }
  localStorage.setItem(WEB_STORAGE_KEY, JSON.stringify(data));
}

export async function setSidebarWindowMode(collapsed: boolean): Promise<void> {
  if (!isTauriRuntime()) return;
  const { invoke } = await import("@tauri-apps/api/core");
  await invoke("set_sidebar_window_mode", { collapsed });
}

export async function setWindowDisplayMode(mode: WindowDisplayMode): Promise<void> {
  if (!isTauriRuntime()) return;
  const { invoke } = await import("@tauri-apps/api/core");
  await invoke("set_window_display_mode", { mode });
}

export async function getDataDirectory(): Promise<string> {
  if (isTauriRuntime()) {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<string>("get_data_directory");
  }
  return "Webプレビュー: このブラウザーのlocalStorage";
}

export async function getAutoStartState(): Promise<boolean | null> {
  if (!isTauriRuntime()) return null;
  const { isEnabled } = await import("@tauri-apps/plugin-autostart");
  return isEnabled();
}

export async function setAutoStartState(enabled: boolean): Promise<void> {
  if (!isTauriRuntime()) return;
  const { disable, enable } = await import("@tauri-apps/plugin-autostart");
  if (enabled) await enable();
  else await disable();
}

export interface GoogleConnectionResult {
  account: GoogleAccount;
  calendars: GoogleCalendarOption[];
}

export interface GoogleCredentialStatus {
  accountId: string;
  available: boolean;
}

export interface GoogleDisconnectResult {
  revoked: boolean;
  message: string;
}

export interface GoogleSyncSummary {
  accountsSynced: number;
  pulled: number;
  pushed: number;
  deleted: number;
  conflicts: number;
  warnings: string[];
}

export interface GoogleSyncResult {
  data: AppData;
  summary: GoogleSyncSummary;
}

export function parseGoogleOAuthClientJson(source: string): GoogleOAuthClient {
  let value: unknown;
  try {
    value = JSON.parse(source);
  } catch {
    throw new Error("GoogleからダウンロードしたOAuthクライアントJSONを選択してください");
  }
  const installed = (value as { installed?: Record<string, unknown> })?.installed;
  const clientId = typeof installed?.client_id === "string" ? installed.client_id.trim() : "";
  const clientSecret = typeof installed?.client_secret === "string" ? installed.client_secret.trim() : "";
  const projectId = typeof installed?.project_id === "string" ? installed.project_id.trim() : "";
  if (!clientId) {
    throw new Error("デスクトップアプリ用のOAuthクライアントIDが見つかりません");
  }
  return { clientId, clientSecret, projectId };
}

export async function connectGoogleAccount(client: GoogleOAuthClient): Promise<GoogleConnectionResult> {
  if (!isTauriRuntime()) throw new Error("Google連携はWindowsアプリ版で設定してください");
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<GoogleConnectionResult>("google_connect_account", { client });
}

export async function listGoogleCalendars(client: GoogleOAuthClient, accountId: string): Promise<GoogleCalendarOption[]> {
  if (!isTauriRuntime()) return [];
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<GoogleCalendarOption[]>("google_list_calendars", { client, accountId });
}

export async function getGoogleCredentialStatuses(accountIds: string[]): Promise<GoogleCredentialStatus[]> {
  if (!isTauriRuntime()) return accountIds.map((accountId) => ({ accountId, available: false }));
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<GoogleCredentialStatus[]>("google_credential_statuses", { accountIds });
}

export async function disconnectGoogleAccount(accountId: string): Promise<GoogleDisconnectResult> {
  if (!isTauriRuntime()) return { revoked: false, message: "Webプレビューには認証情報がありません" };
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<GoogleDisconnectResult>("google_disconnect_account", { accountId });
}

export async function syncGoogleCalendars(): Promise<GoogleSyncResult> {
  if (!isTauriRuntime()) {
    return {
      data: await loadAppData(),
      summary: { accountsSynced: 0, pulled: 0, pushed: 0, deleted: 0, conflicts: 0, warnings: [] },
    };
  }
  const { invoke } = await import("@tauri-apps/api/core");
  const result = await invoke<GoogleSyncResult>("google_sync_all");
  return { ...result, data: normalizeData(result.data) };
}
