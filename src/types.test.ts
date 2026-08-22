import { describe, expect, it } from "vitest";

import { normalizeData } from "./lib/store";
import { createEmptyEvent, DATA_VERSION, DEFAULT_DATA, THEMES } from "./types";

describe("保存形式", () => {
  it("通知設定対応のversion 5を使用する", () => {
    expect(DATA_VERSION).toBe(5);
  });

  it("新しい予定はKoyomadoとGoogleで同じ通知設定を使う", () => {
    const event = createEmptyEvent("new-reminder", "2026-09-01", "2026-08-23T00:00:00Z");
    expect(event.reminders).toEqual({ useGoogleDefault: false, popupMinutes: [], emailMinutes: [] });
  });

  it("終了日がない旧予定は開始日と同じ日へ補完する", () => {
    const legacyEvent = createEmptyEvent("legacy", "2026-09-01", "2026-08-22T00:00:00Z") as Partial<ReturnType<typeof createEmptyEvent>>;
    delete legacyEvent.endDate;
    const normalized = normalizeData({
      ...structuredClone(DEFAULT_DATA),
      events: [{ ...legacyEvent, title: "旧予定" }],
    });
    expect(normalized.events[0].endDate).toBe("2026-09-01");
  });

  it("version 4の予定へ安全な通知初期値を補完する", () => {
    const legacy = structuredClone(DEFAULT_DATA) as unknown as Record<string, unknown>;
    legacy.version = 4;
    const event = createEmptyEvent("legacy-reminder", "2026-09-01", "2026-08-23T00:00:00Z") as unknown as Record<string, unknown>;
    event.title = "移行前の予定";
    delete event.reminders;
    legacy.events = [event];
    const settings = legacy.settings as Record<string, unknown>;
    delete settings.notifications;

    const normalized = normalizeData(legacy);
    expect(normalized.events[0].reminders).toEqual({ useGoogleDefault: true, popupMinutes: [], emailMinutes: [] });
    expect(normalized.settings.notifications).toEqual(DEFAULT_DATA.settings.notifications);
  });

  it("通知時刻はGoogleの上限内で重複なく5件までに整える", () => {
    const source = structuredClone(DEFAULT_DATA);
    const event = createEmptyEvent("reminder", "2026-09-01", "2026-08-23T00:00:00Z");
    event.title = "通知予定";
    event.reminders.popupMinutes = [10, 5, 10, -1, 40_321, 30, 60, 120, 240];
    source.events = [event];
    expect(normalizeData(source).events[0].reminders.popupMinutes).toEqual([5, 10, 30, 60, 120]);
  });

  it("壊れた通知設定は安全な既定値へ戻す", () => {
    const source = structuredClone(DEFAULT_DATA) as unknown as Record<string, unknown>;
    const settings = source.settings as Record<string, unknown>;
    settings.notifications = { soundId: "unknown", volume: "not-a-number", customSound: null };
    const event = createEmptyEvent("invalid-reminder", "2026-09-01", "2026-08-23T00:00:00Z") as unknown as Record<string, unknown>;
    event.reminders = { useGoogleDefault: "yes", popupMinutes: [10], emailMinutes: [] };
    source.events = [event];

    const normalized = normalizeData(source);
    expect(normalized.settings.notifications).toEqual(DEFAULT_DATA.settings.notifications);
    expect(normalized.events[0].reminders.useGoogleDefault).toBe(true);
  });

  it("通知音の長さを3秒から60秒へ収める", () => {
    const source = structuredClone(DEFAULT_DATA);
    source.settings.notifications.soundDurationSeconds = 1;
    expect(normalizeData(source).settings.notifications.soundDurationSeconds).toBe(3);
    source.settings.notifications.soundDurationSeconds = 99;
    expect(normalizeData(source).settings.notifications.soundDurationSeconds).toBe(60);
  });

  it("旧設定へ表示倍率100%を補い、80～130%の5%刻みに整える", () => {
    const legacy = structuredClone(DEFAULT_DATA) as unknown as Record<string, unknown>;
    const legacySettings = legacy.settings as Record<string, unknown>;
    delete legacySettings.uiScalePercent;
    expect(normalizeData(legacy).settings.uiScalePercent).toBe(100);

    const source = structuredClone(DEFAULT_DATA);
    source.settings.uiScalePercent = 77;
    expect(normalizeData(source).settings.uiScalePercent).toBe(80);
    source.settings.uiScalePercent = 127;
    expect(normalizeData(source).settings.uiScalePercent).toBe(125);
    source.settings.uiScalePercent = 200;
    expect(normalizeData(source).settings.uiScalePercent).toBe(130);
  });

  it("version 3の設定は既定保存先なしで読み込める", () => {
    const legacy = structuredClone(DEFAULT_DATA) as unknown as Record<string, unknown>;
    legacy.version = 3;
    const settings = legacy.settings as { google: Record<string, unknown> };
    delete settings.google.defaultSyncTargets;
    expect(normalizeData(legacy).settings.google.defaultSyncTargets).toEqual([]);
  });

  it("既定保存先は同期中の接続アカウントだけを重複なく保持する", () => {
    const source = structuredClone(DEFAULT_DATA);
    source.settings.google.accounts = [
      { id: "active", email: "active@example.invalid", displayName: "Active", calendarId: "primary", calendarName: "Main", syncEnabled: true, syncToken: "", connectedAt: "", lastSyncAt: "", lastError: "", needsReauth: false },
      { id: "paused", email: "paused@example.invalid", displayName: "Paused", calendarId: "primary", calendarName: "Main", syncEnabled: false, syncToken: "", connectedAt: "", lastSyncAt: "", lastError: "", needsReauth: false },
    ];
    source.settings.google.defaultSyncTargets = ["active", "active", "paused", "missing"];
    expect(normalizeData(source).settings.google.defaultSyncTargets).toEqual(["active"]);
  });

  it("新しい予定へ既定保存先を複製し、呼び出し側の配列と共有しない", () => {
    const defaults = ["first", "first", "second"];
    const event = createEmptyEvent("new", "2026-09-01", "2026-08-22T00:00:00Z", defaults);
    defaults.push("third");
    expect(event.syncTargets).toEqual(["first", "second"]);
  });
});

describe("背景テーマ", () => {
  it("8種類の重複しないテーマを提供する", () => {
    expect(THEMES).toHaveLength(8);
    expect(new Set(THEMES.map((theme) => theme.id)).size).toBe(8);
  });

  it("すべてのテーマに表示用の配色が3色ある", () => {
    THEMES.forEach((theme) => {
      expect(theme.name).not.toBe("");
      expect(theme.colors).toHaveLength(3);
    });
  });
});
