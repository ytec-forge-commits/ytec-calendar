import { afterEach, describe, expect, it, vi } from "vitest";
import { createStartupLoader } from "./startupLoader";

afterEach(() => vi.useRealTimers());

describe("startup data recovery", () => {
  it("retries at ten second intervals, stops after three retries and allows a manual retry", async () => {
    vi.useFakeTimers();
    let available = false;
    let calls = 0;
    const events: string[] = [];
    const loader = createStartupLoader(async () => { calls++; if (!available) throw new Error("unavailable"); return "saved calendar"; }, {
      onLoading: () => events.push("loading"), onError: () => events.push("error"), onSuccess: data => events.push(data),
    });
    loader.start(); await vi.advanceTimersByTimeAsync(30_000);
    expect(calls).toBe(4);
    await vi.advanceTimersByTimeAsync(60_000); expect(calls).toBe(4);
    available = true; loader.retry(); await vi.advanceTimersByTimeAsync(0);
    expect(events.at(-1)).toBe("saved calendar"); expect(calls).toBe(5);
    loader.dispose();
  });
  it("waits while changing storage and does not retry after success or disposal", async () => {
    vi.useFakeTimers();
    let calls = 0;
    const loader = createStartupLoader(async () => { if (++calls === 1) throw new Error("offline"); return "ok"; }, { onLoading: () => {}, onError: () => {}, onSuccess: () => {} });
    loader.start(); await vi.advanceTimersByTimeAsync(0);
    loader.setPaused(true); await vi.advanceTimersByTimeAsync(30_000); expect(calls).toBe(1);
    loader.setPaused(false); await vi.advanceTimersByTimeAsync(10_000); expect(calls).toBe(2);
    await vi.advanceTimersByTimeAsync(30_000); expect(calls).toBe(2);
    loader.dispose();
  });
  it("ignores an unfinished load after the user switches data or the component unmounts", async () => {
    let finish!: (value: string) => void;
    const success = vi.fn();
    const loader = createStartupLoader(() => new Promise<string>(resolve => { finish = resolve; }), { onLoading: () => {}, onError: () => {}, onSuccess: success });
    loader.start(); loader.dispose(); finish("old data"); await Promise.resolve();
    expect(success).not.toHaveBeenCalled();
  });
});
