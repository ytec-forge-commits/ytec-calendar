import { describe, expect, it } from "vitest";

import { DATA_VERSION, THEMES } from "./types";

describe("保存形式", () => {
  it("Google同期と繰り返し予定対応のversion 3を使用する", () => {
    expect(DATA_VERSION).toBe(3);
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
