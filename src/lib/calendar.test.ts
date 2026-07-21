import { describe, expect, it } from "vitest";
import { copyEventContent, eventsForDate, getHolidayMap, getMonthCells, isValidTimeRange, pasteEventContent, shiftMonth, toDateKey, upcomingEvents } from "./calendar";
import type { CalendarEvent } from "../types";
import { DEFAULT_EVENT_STYLE } from "../types";

const makeEvent = (id: string, date: string, startTime = "10:00", allDay = false): CalendarEvent => ({
  id,
  title: id,
  date,
  allDay,
  startTime,
  endTime: "11:00",
  location: "",
  notes: "",
  style: DEFAULT_EVENT_STYLE,
  createdAt: "2026-07-21T00:00:00.000Z",
  updatedAt: "2026-07-21T00:00:00.000Z",
});

describe("calendar utilities", () => {
  it("月表示を日曜始まりの42日で生成する", () => {
    const cells = getMonthCells(new Date(2026, 6, 1), new Date(2026, 6, 21));
    expect(cells).toHaveLength(42);
    expect(cells[0].dateKey).toBe("2026-06-28");
    expect(cells[41].dateKey).toBe("2026-08-08");
    expect(cells.find((cell) => cell.dateKey === "2026-07-21")?.isToday).toBe(true);
  });

  it("日付キーはローカル日付を維持する", () => {
    expect(toDateKey(new Date(2026, 0, 5))).toBe("2026-01-05");
  });

  it("終日予定を先頭にして時刻順に並べる", () => {
    const events = [
      makeEvent("遅い", "2026-07-21", "15:00"),
      makeEvent("終日", "2026-07-21", "", true),
      makeEvent("早い", "2026-07-21", "09:00"),
    ];
    expect(eventsForDate(events, "2026-07-21").map((event) => event.id)).toEqual(["終日", "早い", "遅い"]);
  });

  it("今日から指定日数先までの予定を返す", () => {
    const events = [
      makeEvent("今日", "2026-07-21"),
      makeEvent("7日後", "2026-07-28"),
      makeEvent("8日後", "2026-07-29"),
    ];
    expect(upcomingEvents(events, new Date(2026, 6, 21), 7).map((event) => event.id)).toEqual(["今日", "7日後"]);
  });

  it("月移動で年をまたげる", () => {
    expect(toDateKey(shiftMonth(new Date(2026, 11, 1), 1))).toBe("2027-01-01");
  });

  it("終了時刻は開始時刻より後だけを許可する", () => {
    expect(isValidTimeRange("09:00", "10:00")).toBe(true);
    expect(isValidTimeRange("10:00", "10:00")).toBe(false);
    expect(isValidTimeRange("11:00", "10:00")).toBe(false);
  });

  it("日本の祝日名をオフラインデータから取得する", () => {
    const holidays = getHolidayMap(new Date(2026, 6, 1), new Date(2026, 6, 31));
    expect(holidays.get("2026-07-20")).toBe("海の日");
    expect(holidays.has("2026-07-21")).toBe(false);
  });

  it("予定内容の貼り付けでは貼り付け先の日付と識別情報を維持する", () => {
    const source = {
      ...makeEvent("お休み", "2026-07-22", "", true),
      title: "お休み",
      location: "自宅",
      notes: "連絡不要",
      style: { color: "#b49ac7" },
    };
    const target = makeEvent("new-id", "2026-07-29", "09:00");
    const pasted = pasteEventContent(target, copyEventContent(source));

    expect(pasted).toMatchObject({
      id: "new-id",
      date: "2026-07-29",
      title: "お休み",
      allDay: true,
      location: "自宅",
      notes: "連絡不要",
      style: { color: "#b49ac7" },
    });
    expect(pasted.createdAt).toBe(target.createdAt);
  });
});
