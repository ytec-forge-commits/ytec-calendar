import { describe, expect, it } from "vitest";

import { normalizeData } from "./lib/store";
import { createEmptyEvent, DATA_VERSION, DEFAULT_DATA, THEMES } from "./types";

describe("保存形式", () => {
  it("Google同期と繰り返し予定対応のversion 3を使用する", () => {
    expect(DATA_VERSION).toBe(3);
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
