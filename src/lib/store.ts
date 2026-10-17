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
import { mergeAppData } from "./dataSafety";

const WEB_STORAGE_KEY = "koyomado-preview-data-v1";
export async function openUserManual(): Promise<void> {
  if (!isTauriRuntime()) throw new Error("操作説明書はWindowsアプリから開いてください。");
  const { invoke } = await import("@tauri-apps/api/core");
  await invoke("open_user_manual");
}

const LEGACY_WEB_STORAGE_KEY = "ytec-calendar-preview-data-v1";
const dataLocations = new WeakMap<AppData, string>();
interface CalendarSnapshot { data: AppData; directory: string }

function readSnapshot(snapshot: CalendarSnapshot): AppData {
  const data = normalizeData(snapshot.data);
  dataLocations.set(data, snapshot.directory);
  return data;
}

export function sameDataLocation(a: AppData, b: AppData): boolean {
  return dataLocations.get(a) === dataLocations.get(b);
}

export interface AutoBackupSettings {
  enabled: boolean; directory: string; frequency: "daily" | "weekly" | "monthly"; generations: number;
  lastSuccess: string; lastError: string;
}
export interface BackupRestorePreview {
  token: string; createdAt: string; eventCount: number; deletedEventCount: number;
  soundCount: number; googleAccountCount: number; targetDirectory: string;
}
export async function getAutoBackup(): Promise<AutoBackupSettings> {
  if (!isTauriRuntime()) return { enabled: false, directory: "", frequency: "daily", generations: 10, lastSuccess: "", lastError: "" };
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("get_auto_backup");
}
export async function configureAutoBackup(settings: Pick<AutoBackupSettings, "enabled" | "directory" | "frequency" | "generations">): Promise<AutoBackupSettings> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("configure_auto_backup", settings);
}
export async function runDueBackup(): Promise<string | null> {
  if (!isTauriRuntime()) return null;
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("run_due_backup");
}
export async function chooseBackupFile(save: boolean): Promise<string | null> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("choose_backup_file", { save });
}
export async function exportBackup(destination: string): Promise<string> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("export_backup", { destination });
}
export async function previewBackupRestore(archive: string, target: string): Promise<BackupRestorePreview> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke("preview_backup_restore", { archive, target });
}
export async function applyBackupRestore(token: string): Promise<AppData> {
  const { invoke } = await import("@tauri-apps/api/core");
  return readSnapshot(await invoke<CalendarSnapshot>("apply_backup_restore", { token }));
}

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

function normalizeReminderMinutes(value: unknown): number[] {
  if (!Array.isArray(value)) return [];
  return [...new Set(value
    .map(Number)
    .filter((minutes) => Number.isInteger(minutes) && minutes >= 0 && minutes <= 40_320))]
    .sort((a, b) => a - b)
    .slice(0, 5);
}

function normalizeEvent(value: unknown): CalendarEvent {
  const event = value as Partial<CalendarEvent>;
  const recurrence = normalizeRecurrence(event);
  const date = event.date ?? "";
  const endDate = event.endDate && event.endDate >= date ? event.endDate : date;
  const emailMinutes = normalizeReminderMinutes(event.reminders?.emailMinutes);
  const popupMinutes = normalizeReminderMinutes(event.reminders?.popupMinutes).slice(0, Math.max(0, 5 - emailMinutes.length));
  return {
    ...(event as CalendarEvent),
    date,
    endDate,
    annual: recurrence?.kind === "simple" && recurrence.frequency === "yearly",
    recurrence,
    recurrenceException: event.recurrenceException ?? null,
    reminders: {
      useGoogleDefault: typeof event.reminders?.useGoogleDefault === "boolean" ? event.reminders.useGoogleDefault : true,
      popupMinutes,
      emailMinutes,
    },
    origin: event.origin ?? { kind: "local" },
    syncTargets: Array.isArray(event.syncTargets) ? event.syncTargets : [],
    googleLinks: Array.isArray(event.googleLinks) ? event.googleLinks : [],
    syncConflict: event.syncConflict ?? null,
  };
}

export function normalizeData(value: unknown): AppData {
  if (!value || typeof value !== "object") return structuredClone(DEFAULT_DATA);
  const candidate = value as Omit<Partial<AppData>, "version"> & { version?: number };
  if (![1, 2, 3, 4, DATA_VERSION].includes(candidate.version ?? -1) || !Array.isArray(candidate.events)) return structuredClone(DEFAULT_DATA);
  const google = candidate.settings?.google;
  const accounts = Array.isArray(google?.accounts) ? google.accounts.slice(0, 3) : [];
  const activeAccountIds = new Set(accounts.filter((account) => account.syncEnabled).map((account) => account.id));
  const defaultSyncTargets = Array.isArray(google?.defaultSyncTargets)
    ? [...new Set(google.defaultSyncTargets.filter((accountId) => typeof accountId === "string" && activeAccountIds.has(accountId)))]
    : [];
  const notificationCandidate = candidate.settings?.notifications;
  const allowedSoundIds = new Set(["gentle-chimes", "deep-drop", "small-bell", "gentle-piano", "quiet-kalimba", "custom", "silent"]);
  const customSound = notificationCandidate?.customSound
    && typeof notificationCandidate.customSound.displayName === "string"
    && typeof notificationCandidate.customSound.storedFileName === "string"
    && typeof notificationCandidate.customSound.mimeType === "string"
    && (notificationCandidate.customSound.kind === "audio" || notificationCandidate.customSound.kind === "midi")
    ? notificationCandidate.customSound
    : null;
  const requestedSoundId = notificationCandidate?.soundId;
  const soundId = allowedSoundIds.has(requestedSoundId ?? "") && (requestedSoundId !== "custom" || customSound)
    ? requestedSoundId as AppData["settings"]["notifications"]["soundId"]
    : DEFAULT_DATA.settings.notifications.soundId;
  const requestedVolume = Number(notificationCandidate?.volume ?? DEFAULT_DATA.settings.notifications.volume);
  const volume = Number.isFinite(requestedVolume)
    ? Math.max(0, Math.min(100, Math.round(requestedVolume)))
    : DEFAULT_DATA.settings.notifications.volume;
  const requestedDuration = Number(notificationCandidate?.soundDurationSeconds ?? DEFAULT_DATA.settings.notifications.soundDurationSeconds);
  const soundDurationSeconds = Number.isFinite(requestedDuration)
    ? Math.max(3, Math.min(60, Math.round(requestedDuration)))
    : DEFAULT_DATA.settings.notifications.soundDurationSeconds;
  const requestedUiScale = Number(candidate.settings?.uiScalePercent ?? DEFAULT_DATA.settings.uiScalePercent);
  const uiScalePercent = Number.isFinite(requestedUiScale)
    ? Math.max(80, Math.min(130, Math.round(requestedUiScale / 5) * 5))
    : DEFAULT_DATA.settings.uiScalePercent;
  return {
    version: DATA_VERSION,
    events: candidate.events.map(normalizeEvent),
    deletedEvents: Array.isArray(candidate.deletedEvents)
      ? candidate.deletedEvents.map((event) => ({ ...normalizeEvent(event), deletedAt: event.deletedAt }))
      : [],
    settings: {
      theme: candidate.settings?.theme ?? DEFAULT_DATA.settings.theme,
      sidebarCollapsed: candidate.settings?.sidebarCollapsed ?? false,
      uiScalePercent,
      windowDisplayMode: candidate.settings?.windowDisplayMode ?? "taskbar",
      notifications: {
        soundId,
        volume,
        soundDurationSeconds,
        customSound,
      },
      google: {
        enabled: google?.enabled ?? false,
        client: google?.client ?? null,
        accounts,
        defaultSyncTargets,
      },
    },
  };
}

export async function loadAppData(): Promise<AppData> {
  if (isTauriRuntime()) {
    const { invoke } = await import("@tauri-apps/api/core");
    return readSnapshot(await invoke<CalendarSnapshot>("load_app_data"));
  }

  const stored = localStorage.getItem(WEB_STORAGE_KEY) ?? localStorage.getItem(LEGACY_WEB_STORAGE_KEY);
  if (!stored) return structuredClone(DEFAULT_DATA);
  try {
    const value = JSON.parse(stored);
    if (!value || ![1, 2, 3, 4, DATA_VERSION].includes(value.version) || !Array.isArray(value.events)) {
      throw new Error("Unsupported calendar data");
    }
    return normalizeData(value);
  } catch {
    throw new Error("保存データを読み込めません。元のデータを保護するため、空のカレンダーでの開始を停止しました。");
  }
}

export async function saveAppData(data: AppData, base: AppData): Promise<AppData> {
  if (isTauriRuntime()) {
    const { invoke } = await import("@tauri-apps/api/core");
    // The backend checks again under a process-shared file lock. Retry only
    // the read/merge/save operation, never an actual conflicting edit.
    for (let attempt = 0; attempt < 3; attempt += 1) {
      const latest = await loadAppData();
      if (!dataLocations.has(base) || dataLocations.get(base) !== dataLocations.get(latest)) {
        throw new Error("保存先が別の画面で変更されました。入力を控えて編集を閉じ、最新の保存先を確認してください。");
      }
      const merged = mergeAppData(base, data, latest);
      try {
        return readSnapshot(await invoke<CalendarSnapshot>("save_app_data", { data: merged, expected: latest, expectedDirectory: dataLocations.get(latest) }));
      } catch (error) {
        if (!String(error).startsWith("DATA_CHANGED:")) throw error;
      }
    }
    throw new Error("別の画面で保存が続いています。入力を残していますので、少し待ってもう一度保存してください。");
  }
  const merged = mergeAppData(base, data, await loadAppData());
  localStorage.setItem(WEB_STORAGE_KEY, JSON.stringify(merged));
  return merged;
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

export interface StorageInfo { directory: string; defaultDirectory: string; available: boolean }
export interface StoragePreview {
  token: string;
  sourceDirectory: string;
  targetDirectory: string;
  sourceEventCount: number | null;
  targetEventCount: number | null;
}

export async function getStorageInfo(): Promise<StorageInfo> {
  if (!isTauriRuntime()) return { directory: "Webプレビュー: このブラウザーのlocalStorage", defaultDirectory: "", available: true };
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<StorageInfo>("get_storage_info");
}

export async function chooseStorageDirectory(): Promise<string | null> {
  if (!isTauriRuntime()) throw new Error("保存先の変更はWindowsアプリ版で利用できます");
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<string | null>("choose_storage_directory");
}

export async function previewStorageDirectory(directory: string): Promise<StoragePreview> {
  if (!isTauriRuntime()) throw new Error("保存先の変更はWindowsアプリ版で利用できます");
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<StoragePreview>("preview_storage_directory", { directory });
}

export async function applyStorageDirectory(token: string, mode: "copyCurrent" | "useExisting"): Promise<AppData> {
  if (!isTauriRuntime()) throw new Error("保存先の変更はWindowsアプリ版で利用できます");
  const { invoke } = await import("@tauri-apps/api/core");
  return readSnapshot(await invoke<CalendarSnapshot>("apply_storage_directory", { token, mode }));
}

export async function getAutoStartState(): Promise<boolean | null> {
  if (!isTauriRuntime()) return null;
  const { invoke } = await import("@tauri-apps/api/core");
  const enabled = await invoke<boolean>("get_auto_start_state");
  if (enabled) {
    await invoke("repair_auto_start").catch((error) => {
      console.warn("自動起動の耐障害設定を更新できませんでした", error);
    });
  }
  return enabled;
}

export async function setAutoStartState(enabled: boolean): Promise<void> {
  if (!isTauriRuntime()) return;
  const { invoke } = await import("@tauri-apps/api/core");
  await invoke("set_auto_start_state", { enabled });
}

export async function saveCustomNotificationSound(file: File): Promise<AppData["settings"]["notifications"]["customSound"]> {
  if (!isTauriRuntime()) throw new Error("カスタム通知音はWindowsアプリ版で設定してください");
  const { invoke } = await import("@tauri-apps/api/core");
  const bytes = Array.from(new Uint8Array(await file.arrayBuffer()));
  return invoke<AppData["settings"]["notifications"]["customSound"]>("save_custom_notification_sound", {
    fileName: file.name,
    bytes,
  });
}

export async function loadCustomNotificationSound(storedFileName: string): Promise<Uint8Array> {
  if (!isTauriRuntime()) throw new Error("カスタム通知音はWindowsアプリ版で再生してください");
  const { invoke } = await import("@tauri-apps/api/core");
  return new Uint8Array(await invoke<number[]>("load_custom_notification_sound", { storedFileName }));
}

export async function showMainWindowForNotification(): Promise<void> {
  if (!isTauriRuntime()) return;
  const { invoke } = await import("@tauri-apps/api/core");
  await invoke("show_main_window_for_notification");
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
  const result = await invoke<GoogleSyncResult & { directory: string }>("google_sync_all");
  return { ...result, data: readSnapshot(result) };
}
