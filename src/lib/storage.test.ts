import { afterEach, expect, it, vi } from "vitest";
import { DEFAULT_DATA, createEmptyEvent } from "../types";
import { loadAppData, saveAppData } from "./store";

afterEach(() => vi.unstubAllGlobals());
it("reads the native calendar with its location", async () => {
  const data = structuredClone(DEFAULT_DATA);
  data.events.push({ ...createEmptyEvent("synthetic", "2026-10-17", "2026-10-17T00:00:00Z"), title: "synthetic" });
  vi.stubGlobal("window", { __TAURI_INTERNALS__: { invoke: async () => ({ data, directory: "C:\\Synthetic\\Local" }) } });
  expect((await loadAppData()).events.map(e => e.id)).toEqual(["synthetic"]);
});

it("never writes an old editor's data into a newly selected location", async () => {
  let directory = "C:\\Synthetic\\Local";
  let writes = 0;
  vi.stubGlobal("window", { __TAURI_INTERNALS__: { invoke: async (command: string) => {
    if (command === "load_app_data") return { data: structuredClone(DEFAULT_DATA), directory };
    if (command === "save_app_data") writes += 1;
    return { data: structuredClone(DEFAULT_DATA), directory };
  } } });
  const base = await loadAppData();
  directory = "G:\\Synthetic\\Drive";
  const draft = structuredClone(base); draft.events.push({ ...createEmptyEvent("local", "2026-10-17", "2026-10-17T00:00:00Z"), title: "local" });
  await expect(saveAppData(draft, base)).rejects.toThrow(/保存先/);
  expect(writes).toBe(0);
});
