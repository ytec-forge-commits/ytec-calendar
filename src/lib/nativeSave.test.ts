import { isDeepStrictEqual } from "node:util";
import { afterEach, expect, it, vi } from "vitest";
import { createEmptyEvent, DEFAULT_DATA, type AppData, type CalendarEvent } from "../types";
import { loadAppData, saveAppData } from "./store";

const timestamp = "2026-11-25T00:00:00Z";
const directory = "C:\\Synthetic\\Calendar";

function yearlyGoogleEvent(): CalendarEvent {
  return {
    ...createEmptyEvent("synthetic-yearly", "2026-11-25", timestamp, ["fixture-account"]),
    title: "Synthetic yearly event",
    annual: true,
    recurrence: { kind: "google", lines: ["RRULE:FREQ=YEARLY"], timeZone: "Asia/Tokyo", excludedDates: [] },
    origin: { kind: "google", accountId: "fixture-account" },
    googleLinks: [{
      accountId: "fixture-account", calendarId: "fixture-calendar", eventId: "fixture-event",
      etag: "fixture-etag", googleUpdatedAt: timestamp, localUpdatedAt: timestamp,
    }],
  };
}

function useNativeCalendar(initial: AppData) {
  let stored = structuredClone(initial);
  let concurrentEdit: ((data: AppData) => void) | undefined;
  vi.stubGlobal("window", { __TAURI_INTERNALS__: { invoke: async (
    command: string, args?: { data: AppData; expected: AppData; expectedDirectory: string },
  ) => {
    if (command === "save_app_data" && args) {
      concurrentEdit?.(stored);
      concurrentEdit = undefined;
      if (args.expectedDirectory !== directory) throw new Error("Unexpected calendar directory");
      // The native boundary compares the complete snapshot before writing.
      if (!isDeepStrictEqual(args.expected, stored)) throw "DATA_CHANGED: snapshot changed";
      stored = structuredClone(args.data);
    } else if (command !== "load_app_data") {
      throw new Error(`Unexpected command: ${command}`);
    }
    return { data: structuredClone(stored), directory };
  } } });
  return { editBeforeNextSave: (edit: (data: AppData) => void) => { concurrentEdit = edit; } };
}

afterEach(() => vi.unstubAllGlobals());

it.each(["events", "deletedEvents"] as const)("saves a theme and preserves the Google yearly record in %s", async (list) => {
  const data = structuredClone(DEFAULT_DATA);
  const yearly = yearlyGoogleEvent();
  if (list === "events") data.events.push(yearly);
  else data.deletedEvents.push({ ...yearly, deletedAt: timestamp });
  useNativeCalendar(data);

  const base = await loadAppData();
  await saveAppData({ ...base, settings: { ...base.settings, theme: "forest-breath" } }, base);

  const saved = await loadAppData();
  expect(saved.settings.theme).toBe("forest-breath");
  expect(saved[list]).toEqual(data[list]);
});

it("saves an edit to a Google yearly event without changing its recurrence or links", async () => {
  const yearly = yearlyGoogleEvent();
  useNativeCalendar({ ...DEFAULT_DATA, events: [yearly] });
  const base = await loadAppData();
  const notes = "Edited synthetic notes";

  await saveAppData({ ...base, events: [{ ...base.events[0], notes }] }, base);

  expect((await loadAppData()).events).toEqual([{ ...yearly, notes }]);
});

it("keeps a concurrent event edit when retrying a theme save", async () => {
  const yearly = yearlyGoogleEvent();
  const native = useNativeCalendar({ ...DEFAULT_DATA, events: [yearly] });
  const base = await loadAppData();
  native.editBeforeNextSave((data) => { data.events[0].notes = "Other window"; });

  await saveAppData({ ...base, settings: { ...base.settings, theme: "forest-breath" } }, base);

  const saved = await loadAppData();
  expect(saved.settings.theme).toBe("forest-breath");
  expect(saved.events).toEqual([{ ...yearly, notes: "Other window" }]);
});

it("rejects a concurrent edit to the same Google yearly event", async () => {
  const yearly = yearlyGoogleEvent();
  const native = useNativeCalendar({ ...DEFAULT_DATA, events: [yearly] });
  const base = await loadAppData();
  native.editBeforeNextSave((data) => { data.events[0].notes = "Other window"; });

  await expect(saveAppData({ ...base, events: [{ ...base.events[0], notes: "This window" }] }, base))
    .rejects.toThrow(/同じ予定または設定/);

  expect((await loadAppData()).events).toEqual([{ ...yearly, notes: "Other window" }]);
});
