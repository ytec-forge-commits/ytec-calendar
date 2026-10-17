import type { AppData, CalendarEvent, DeletedCalendarEvent } from "../types";

function canonical(value: unknown): string {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value && typeof value === "object") {
    return `{${Object.entries(value).filter(([, item]) => item !== undefined).sort(([a], [b]) => a.localeCompare(b)).map(([key, item]) => `${JSON.stringify(key)}:${canonical(item)}`).join(",")}}`;
  }
  return JSON.stringify(value) ?? "undefined";
}

export function sameData(a: unknown, b: unknown): boolean {
  return canonical(a) === canonical(b);
}

function mergeValue<T>(base: T, next: T, latest: T): T {
  if (sameData(next, base)) return latest;
  if (sameData(latest, base) || sameData(next, latest)) return next;
  throw new Error("別の画面で同じ予定または設定が変更されています。入力は保存せず残しています。内容を控えて編集画面を閉じると、最新データを読み込みます。");
}

// A recurring master, its exceptions and deletion records form one unit:
// merging their individual IDs could resurrect a deleted series or orphan an exception.
function families(data: AppData) {
  const result = new Map<string, { events: CalendarEvent[]; deletedEvents: DeletedCalendarEvent[] }>();
  const group = (event: CalendarEvent) => {
    const key = event.recurrenceException?.masterId ?? event.id;
    if (!result.has(key)) result.set(key, { events: [], deletedEvents: [] });
    return result.get(key)!;
  };
  data.events.forEach(event => group(event).events.push(event));
  data.deletedEvents.forEach(event => group(event).deletedEvents.push(event));
  return result;
}

export function mergeAppData(base: AppData, next: AppData, latest: AppData): AppData {
  const before = families(base), local = families(next), remote = families(latest);
  const events: CalendarEvent[] = [], deletedEvents: DeletedCalendarEvent[] = [];
  for (const id of new Set([...remote.keys(), ...local.keys(), ...before.keys()])) {
    const merged = mergeValue(before.get(id), local.get(id), remote.get(id));
    if (merged) { events.push(...merged.events); deletedEvents.push(...merged.deletedEvents); }
  }
  const settings = { ...latest.settings };
  for (const key of Object.keys(settings) as (keyof AppData["settings"])[]) {
    Object.assign(settings, { [key]: mergeValue(base.settings[key], next.settings[key], latest.settings[key]) });
  }
  return { ...latest, events, deletedEvents, settings };
}
