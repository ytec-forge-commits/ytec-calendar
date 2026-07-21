import type { AppData } from "../types";
import { DATA_VERSION, DEFAULT_DATA } from "../types";

const WEB_STORAGE_KEY = "koyomado-preview-data-v1";
const LEGACY_WEB_STORAGE_KEY = "ytec-calendar-preview-data-v1";

export function isTauriRuntime(): boolean {
  return "__TAURI_INTERNALS__" in window;
}

function normalizeData(value: unknown): AppData {
  if (!value || typeof value !== "object") return structuredClone(DEFAULT_DATA);
  const candidate = value as Omit<Partial<AppData>, "version"> & { version?: number };
  if ((candidate.version !== 1 && candidate.version !== DATA_VERSION) || !Array.isArray(candidate.events)) return structuredClone(DEFAULT_DATA);
  return {
    version: DATA_VERSION,
    events: candidate.events.map((event) => ({ ...event, annual: event.annual ?? false })),
    deletedEvents: Array.isArray(candidate.deletedEvents) ? candidate.deletedEvents : [],
    settings: {
      theme: candidate.settings?.theme ?? DEFAULT_DATA.settings.theme,
      sidebarCollapsed: candidate.settings?.sidebarCollapsed ?? false,
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
