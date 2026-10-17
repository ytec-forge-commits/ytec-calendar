import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { createEmptyEvent, DEFAULT_DATA, type AppData } from "../types";
import { loadAppData, saveAppData } from "./store";

const key = "koyomado-preview-data-v1";
const event = (id: string) => ({ ...createEmptyEvent(id, "2026-10-17", "2026-10-17T00:00:00Z"), title: id });
const read = () => JSON.parse(localStorage.getItem(key)!) as AppData;
beforeEach(() => {
  const items = new Map<string, string>();
  vi.stubGlobal("window", {});
  vi.stubGlobal("localStorage", {
    getItem: (name: string) => items.get(name) ?? null,
    setItem: (name: string, value: string) => { items.set(name, value); },
  });
  localStorage.setItem(key, JSON.stringify(DEFAULT_DATA));
});
afterEach(() => vi.unstubAllGlobals());

describe("shared data protection", () => {
  it("does not treat unreadable saved data as an empty calendar", async () => {
    localStorage.setItem(key, "broken JSON");
    await expect(loadAppData()).rejects.toThrow();
    expect(localStorage.getItem(key)).toBe("broken JSON");
  });
  it("keeps the other window's event when a stale window changes only its theme", async () => {
    const base = await loadAppData();
    const office = structuredClone(base); office.events.push(event("office"));
    await saveAppData(office, base);
    const home = structuredClone(base); home.settings.theme = "forest-breath";
    await saveAppData(home, base);
    expect(read().events.map(e => e.id)).toEqual(["office"]);
  });
  it("keeps additions from two windows without restarting either", async () => {
    const base = await loadAppData();
    const office = structuredClone(base); office.events.push(event("office"));
    const home = structuredClone(base); home.events.push(event("home"));
    await saveAppData(office, base); await saveAppData(home, base);
    expect(read().events.map(e => e.id)).toEqual(["office", "home"]);
  });
  it("rejects conflicting edits to the same event without changing the saved version", async () => {
    const initial = structuredClone(DEFAULT_DATA); initial.events = [event("shared")];
    localStorage.setItem(key, JSON.stringify(initial));
    const base = await loadAppData();
    const office = structuredClone(base); office.events[0].title = "office edit";
    const home = structuredClone(base); home.events[0].title = "home edit";
    await saveAppData(office, base);
    await expect(saveAppData(home, base)).rejects.toThrow(/別の画面/);
    expect(read().events[0].title).toBe("office edit");
  });
  it("does not resurrect an event deleted by the other window", async () => {
    const initial = structuredClone(DEFAULT_DATA); initial.events = [event("shared")];
    localStorage.setItem(key, JSON.stringify(initial));
    const base = await loadAppData();
    const office = structuredClone(base); office.events = [];
    office.deletedEvents = [{ ...base.events[0], deletedAt: "2026-10-17T01:00:00Z" }];
    await saveAppData(office, base);
    const home = structuredClone(base); home.settings.sidebarCollapsed = true;
    await saveAppData(home, base);
    expect(read().events).toHaveLength(0);
    expect(read().deletedEvents.map(e => e.id)).toEqual(["shared"]);
  });
  it("rejects an edit to a concurrently deleted event", async () => {
    const initial = structuredClone(DEFAULT_DATA); initial.events = [event("shared")];
    localStorage.setItem(key, JSON.stringify(initial));
    const base = await loadAppData();
    const office = structuredClone(base); office.events = [];
    office.deletedEvents = [{ ...base.events[0], deletedAt: "2026-10-17T01:00:00Z" }];
    await saveAppData(office, base);
    const home = structuredClone(base); home.events[0].title = "home edit";
    await expect(saveAppData(home, base)).rejects.toThrow(/別の画面/);
    expect(read().events).toHaveLength(0);
    expect(read().deletedEvents.map(e => e.id)).toEqual(["shared"]);
  });
  it("keeps different settings changed by separate windows", async () => {
    const base = await loadAppData();
    const office = structuredClone(base); office.settings.sidebarCollapsed = true;
    await saveAppData(office, base);
    const home = structuredClone(base); home.settings.theme = "forest-breath";
    const saved = await saveAppData(home, base);
    expect(saved.settings.sidebarCollapsed).toBe(true);
    expect(saved.settings.theme).toBe("forest-breath");
  });
  it("rejects a new exception if the other window deleted its parent series", async () => {
    const initial = structuredClone(DEFAULT_DATA);
    initial.events = [{ ...event("master"), recurrence: { kind: "simple", frequency: "weekly", interval: 1, weekDays: [6], monthlyMode: "day-of-month", end: { type: "never" }, excludedDates: [] } }];
    localStorage.setItem(key, JSON.stringify(initial));
    const base = await loadAppData();
    const office = structuredClone(base); office.events = [];
    office.deletedEvents = [{ ...base.events[0], deletedAt: "2026-10-17T01:00:00Z" }];
    await saveAppData(office, base);
    const home = structuredClone(base);
    home.events.push({ ...event("exception"), recurrenceException: { masterId: "master", originalDate: "2026-10-17" } });
    await expect(saveAppData(home, base)).rejects.toThrow(/別の画面/);
    expect(read().events).toHaveLength(0);
  });
});
