import { describe, expect, it, vi } from "vitest";
import {
  collectDueNotifications,
  maxReminderInputAmount,
  NOTIFICATION_SOUND_AUTO_STOP_MS,
  parseMidi,
  REMINDER_PRESETS,
  reminderInputParts,
  reminderLabel,
  reminderMinutesFromInput,
  scheduleNotificationPlaybackStop,
  togglePopupReminderPreset,
  withEditedPopupReminders,
} from "./notifications";
import { createEmptyEvent } from "../types";

describe("予定通知", () => {
  it("開始10分前を過ぎた直後の予定だけを通知する", () => {
    const event = createEmptyEvent("meeting", "2026-09-01", "2026-08-23T00:00:00Z");
    event.title = "打ち合わせ";
    event.startTime = "09:00";
    event.endTime = "10:00";
    event.reminders.popupMinutes = [10];

    const due = collectDueNotifications([event], new Date(2026, 8, 1, 8, 50, 30));
    expect(due).toHaveLength(1);
    expect(due[0]).toMatchObject({ key: "meeting:2026-09-01:10", reminderMinutes: 10 });
  });

  it("通知時刻より前と猶予時間を過ぎた後は通知しない", () => {
    const event = createEmptyEvent("meeting", "2026-09-01", "2026-08-23T00:00:00Z");
    event.title = "打ち合わせ";
    event.startTime = "09:00";
    event.endTime = "10:00";
    event.reminders.popupMinutes = [10];

    expect(collectDueNotifications([event], new Date(2026, 8, 1, 8, 49, 59))).toEqual([]);
    expect(collectDueNotifications([event], new Date(2026, 8, 1, 8, 53, 0))).toEqual([]);
  });

  it("複数日の予定を日数分重複通知しない", () => {
    const event = createEmptyEvent("trip", "2026-09-01", "2026-08-23T00:00:00Z");
    event.title = "出張";
    event.allDay = true;
    event.endDate = "2026-09-03";
    event.reminders.popupMinutes = [0];

    expect(collectDueNotifications([event], new Date(2026, 8, 1, 0, 0, 30))).toHaveLength(1);
  });

  it("分数を読みやすい単位で表示する", () => {
    expect(reminderLabel(0)).toBe("開始時刻");
    expect(reminderLabel(30)).toBe("30分前");
    expect(reminderLabel(120)).toBe("2時間前");
    expect(reminderLabel(2_880)).toBe("2日前");
    expect(reminderLabel(20_160)).toBe("2週間前");
  });

  it("分・時間・日の入力と内部の分数を相互変換する", () => {
    expect(reminderInputParts(10)).toEqual({ amount: 10, unit: "minutes" });
    expect(reminderInputParts(720)).toEqual({ amount: 12, unit: "hours" });
    expect(reminderInputParts(1_440)).toEqual({ amount: 1, unit: "days" });
    expect(reminderInputParts(2_880)).toEqual({ amount: 2, unit: "days" });
    expect(reminderInputParts(90)).toEqual({ amount: 90, unit: "minutes" });
    expect(reminderMinutesFromInput(12, "hours")).toBe(720);
    expect(reminderMinutesFromInput(1, "days")).toBe(1_440);
    expect(reminderMinutesFromInput(29, "days")).toBe(40_320);
    expect(maxReminderInputAmount("minutes")).toBe(40_320);
    expect(maxReminderInputAmount("hours")).toBe(672);
    expect(maxReminderInputAmount("days")).toBe(28);
  });

  it("Koyomadoの通知を編集したらGoogle側も同じ通知を使う", () => {
    const event = createEmptyEvent("reminder-mode", "2026-09-01", "2026-08-23T00:00:00Z");
    event.reminders.useGoogleDefault = true;
    expect(event.reminders.useGoogleDefault).toBe(true);

    const reminders = withEditedPopupReminders(event.reminders, [1_440, 360, 30]);

    expect(reminders).toEqual({
      useGoogleDefault: false,
      popupMinutes: [1_440, 360, 30],
      emailMinutes: [],
    });
  });

  it("よく使う7種類をクリックだけで複数選択・解除できる", () => {
    expect(REMINDER_PRESETS.map((preset) => preset.label)).toEqual([
      "10分前", "30分前", "1時間前", "3時間前", "6時間前", "12時間前", "1日前",
    ]);
    const event = createEmptyEvent("preset-reminder", "2026-09-01", "2026-08-23T00:00:00Z");
    event.reminders.useGoogleDefault = true;

    const withThirtyMinutes = togglePopupReminderPreset(event.reminders, 30);
    const withTwoPresets = togglePopupReminderPreset(withThirtyMinutes, 360);
    const withoutThirtyMinutes = togglePopupReminderPreset(withTwoPresets, 30);

    expect(withTwoPresets).toEqual({ useGoogleDefault: false, popupMinutes: [30, 360], emailMinutes: [] });
    expect(withoutThirtyMinutes).toEqual({ useGoogleDefault: false, popupMinutes: [360], emailMinutes: [] });
  });

  it("通知が5件ある場合は未選択プリセットを追加せず、選択済みは解除できる", () => {
    const event = createEmptyEvent("full-reminder", "2026-09-01", "2026-08-23T00:00:00Z");
    event.reminders.popupMinutes = [10, 30, 60];
    event.reminders.emailMinutes = [120, 240];

    expect(togglePopupReminderPreset(event.reminders, 360)).toBe(event.reminders);
    expect(togglePopupReminderPreset(event.reminders, 30).popupMinutes).toEqual([10, 60]);
  });

  it("通知音を12秒後に自動停止し、キャンセル時は停止しない", () => {
    vi.useFakeTimers();
    const stop = vi.fn();
    const onStopped = vi.fn();
    scheduleNotificationPlaybackStop({ stop }, onStopped);
    vi.advanceTimersByTime(NOTIFICATION_SOUND_AUTO_STOP_MS - 1);
    expect(stop).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    expect(stop).toHaveBeenCalledOnce();
    expect(onStopped).toHaveBeenCalledOnce();

    const cancelledStop = vi.fn();
    const cancel = scheduleNotificationPlaybackStop({ stop: cancelledStop }, vi.fn());
    cancel();
    vi.advanceTimersByTime(NOTIFICATION_SOUND_AUTO_STOP_MS);
    expect(cancelledStop).not.toHaveBeenCalled();

    const shortStop = vi.fn();
    scheduleNotificationPlaybackStop({ stop: shortStop }, vi.fn(), 3_000);
    vi.advanceTimersByTime(2_999);
    expect(shortStop).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    expect(shortStop).toHaveBeenCalledOnce();
    vi.useRealTimers();
  });
});

describe("MIDI通知音", () => {
  it("標準MIDIのノートとテンポを解析する", () => {
    const bytes = new Uint8Array([
      0x4d, 0x54, 0x68, 0x64, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x01, 0x01, 0xe0,
      0x4d, 0x54, 0x72, 0x6b, 0x00, 0x00, 0x00, 0x14,
      0x00, 0xff, 0x51, 0x03, 0x07, 0xa1, 0x20,
      0x00, 0x90, 0x3c, 0x64,
      0x83, 0x60, 0x80, 0x3c, 0x00,
      0x00, 0xff, 0x2f, 0x00,
    ]);
    const parsed = parseMidi(bytes);
    expect(parsed.notes).toHaveLength(1);
    expect(parsed.notes[0].note).toBe(60);
    expect(parsed.notes[0].durationSeconds).toBeCloseTo(0.5, 3);
  });

  it("壊れたMIDIを拒否する", () => {
    expect(() => parseMidi(new Uint8Array([0, 1, 2]))).toThrow("ヘッダー");
  });
});
