import { describe, expect, it, vi } from "vitest";
import { copyEventContent, duplicateEventToDate, endTimeOneHourAfter, eventsForDate, getHolidayMap, getMonthCells, getTodayView, isValidEventRange, isValidTimeRange, moveEventToDate, pasteEventContent, recurrenceMatches, shiftMonth, toDateKey, upcomingEvents } from "./calendar";
import type { CalendarEvent } from "../types";
import { createEmptyEvent } from "../types";

const makeEvent = (id: string, date: string, startTime = "10:00", allDay = false): CalendarEvent => ({
  ...createEmptyEvent(id, date, "2026-07-21T00:00:00.000Z"),
  title: id,
  startTime,
  endTime: "11:00",
  allDay,
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

  it("今日の表示先は呼び出した時点の現在日を使う", () => {
    vi.useFakeTimers();
    try {
      vi.setSystemTime(new Date(2026, 6, 22, 23, 59));
      expect(getTodayView()).toMatchObject({ dateKey: "2026-07-22" });

      vi.setSystemTime(new Date(2026, 6, 23, 0, 1));
      const nextDay = getTodayView();
      expect(nextDay).toMatchObject({ dateKey: "2026-07-23" });
      expect(toDateKey(nextDay.displayMonth)).toBe("2026-07-01");
    } finally {
      vi.useRealTimers();
    }
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

  it("毎年の記念日を年に関係なく同じ月日に表示する", () => {
    const anniversary = {
      ...makeEvent("誕生日", "2024-07-24", "", true),
      annual: true,
      recurrence: { kind: "simple" as const, frequency: "yearly" as const, interval: 1, weekDays: [], monthlyMode: "day-of-month" as const, end: { type: "never" as const }, excludedDates: [] },
    };
    const occurrence = eventsForDate([anniversary], "2027-07-24");

    expect(occurrence).toHaveLength(1);
    expect(occurrence[0]).toMatchObject({ id: "誕生日", date: "2027-07-24", annual: true });
    expect(eventsForDate([anniversary], "2027-07-25")).toHaveLength(0);
  });

  it("直近予定にも毎年の記念日を展開する", () => {
    const anniversary = {
      ...makeEvent("記念日", "2021-01-02", "", true),
      annual: true,
      recurrence: { kind: "simple" as const, frequency: "yearly" as const, interval: 1, weekDays: [], monthlyMode: "day-of-month" as const, end: { type: "never" as const }, excludedDates: [] },
    };

    expect(upcomingEvents([anniversary], new Date(2026, 11, 30), 4)).toMatchObject([
      { id: "記念日", date: "2027-01-02", annual: true },
    ]);
  });

  it("毎日・毎週・毎月・毎年の繰り返しを指定間隔で展開する", () => {
    const simple = (frequency: "daily" | "weekly" | "monthly" | "yearly") => ({
      kind: "simple" as const,
      frequency,
      interval: 2,
      weekDays: frequency === "weekly" ? [1, 5] : [],
      monthlyMode: "day-of-month" as const,
      end: { type: "never" as const },
      excludedDates: [],
    });
    const daily = { ...makeEvent("daily", "2026-01-01"), recurrence: simple("daily") };
    const weekly = { ...makeEvent("weekly", "2026-01-05"), recurrence: simple("weekly") };
    const monthly = { ...makeEvent("monthly", "2026-01-15"), recurrence: simple("monthly") };
    const yearly = { ...makeEvent("yearly", "2024-02-29"), recurrence: simple("yearly") };

    expect(recurrenceMatches(daily, "2026-01-03")).toBe(true);
    expect(recurrenceMatches(daily, "2026-01-02")).toBe(false);
    expect(recurrenceMatches(weekly, "2026-01-09")).toBe(true);
    expect(recurrenceMatches(weekly, "2026-01-12")).toBe(false);
    expect(recurrenceMatches(weekly, "2026-01-19")).toBe(true);
    expect(recurrenceMatches(monthly, "2026-03-15")).toBe(true);
    expect(recurrenceMatches(monthly, "2026-02-15")).toBe(false);
    expect(recurrenceMatches(yearly, "2028-02-29")).toBe(true);
    expect(recurrenceMatches(yearly, "2026-02-28")).toBe(false);
  });

  it("毎月の第何曜日、終了日、回数、除外日を正しく扱う", () => {
    const monthly = {
      ...makeEvent("monthly-weekday", "2026-01-13"),
      recurrence: {
        kind: "simple" as const,
        frequency: "monthly" as const,
        interval: 1,
        weekDays: [],
        monthlyMode: "weekday-of-month" as const,
        end: { type: "until" as const, date: "2026-03-31" },
        excludedDates: ["2026-02-10"],
      },
    };
    expect(recurrenceMatches(monthly, "2026-02-10")).toBe(false);
    expect(recurrenceMatches(monthly, "2026-03-10")).toBe(true);
    expect(recurrenceMatches(monthly, "2026-04-14")).toBe(false);

    const counted = {
      ...makeEvent("counted", "2026-01-01"),
      recurrence: {
        kind: "simple" as const,
        frequency: "daily" as const,
        interval: 1,
        weekDays: [],
        monthlyMode: "day-of-month" as const,
        end: { type: "count" as const, count: 3 },
        excludedDates: ["2026-01-02"],
      },
    };
    expect(recurrenceMatches(counted, "2026-01-03")).toBe(true);
    expect(recurrenceMatches(counted, "2026-01-04")).toBe(false);
  });

  it("繰り返しの1回だけを移動した例外は元の日に重複せず移動先に出る", () => {
    const master = {
      ...makeEvent("series", "2026-07-01"),
      recurrence: {
        kind: "simple" as const,
        frequency: "daily" as const,
        interval: 1,
        weekDays: [],
        monthlyMode: "day-of-month" as const,
        end: { type: "never" as const },
        excludedDates: [],
      },
    };
    const moved = {
      ...makeEvent("exception", "2026-07-10"),
      recurrenceException: { masterId: master.id, originalDate: "2026-07-08" },
    };
    expect(eventsForDate([master, moved], "2026-07-08")).toHaveLength(0);
    expect(eventsForDate([master, moved], "2026-07-10").map((event) => event.id).sort()).toEqual(["exception", "series"]);
  });

  it("複数日にまたがる繰り返し予定は各回の期間を保って展開する", () => {
    const master = {
      ...makeEvent("weekly-trip", "2026-09-01", "", true),
      endDate: "2026-09-03",
      recurrence: {
        kind: "simple" as const,
        frequency: "weekly" as const,
        interval: 1,
        weekDays: [2],
        monthlyMode: "day-of-month" as const,
        end: { type: "never" as const },
        excludedDates: [],
      },
    };
    const middleDay = eventsForDate([master], "2026-09-09");
    expect(middleDay).toHaveLength(1);
    expect(middleDay[0]).toMatchObject({
      date: "2026-09-08",
      endDate: "2026-09-10",
      occurrence: { masterId: "weekly-trip", originalDate: "2026-09-08" },
    });
  });

  it("GoogleのRRULEをKoyomadoの月表示へ展開する", () => {
    const imported = {
      ...makeEvent("google-rule", "2026-07-06"),
      recurrence: {
        kind: "google" as const,
        lines: ["RRULE:FREQ=WEEKLY;BYDAY=MO,WE;COUNT=4"],
        timeZone: "Asia/Tokyo",
        excludedDates: [],
      },
    };
    expect(recurrenceMatches(imported, "2026-07-06")).toBe(true);
    expect(recurrenceMatches(imported, "2026-07-08")).toBe(true);
    expect(recurrenceMatches(imported, "2026-07-13")).toBe(true);
    expect(recurrenceMatches(imported, "2026-07-20")).toBe(false);
  });

  it("同じ日に5件、月内に10件あっても予定データを欠落させない", () => {
    const fiveOnOneDay = Array.from({ length: 5 }, (_, index) => makeEvent(`day-${index + 1}`, "2026-07-22", `0${index + 8}:00`));
    const tenInMonth = Array.from({ length: 10 }, (_, index) => makeEvent(`month-${index + 1}`, `2026-07-${String(index + 1).padStart(2, "0")}`));
    expect(eventsForDate(fiveOnOneDay, "2026-07-22")).toHaveLength(5);
    expect(tenInMonth.flatMap((event) => eventsForDate(tenInMonth, event.date))).toHaveLength(10);
  });

  it("月移動で年をまたげる", () => {
    expect(toDateKey(shiftMonth(new Date(2026, 11, 1), 1))).toBe("2027-01-01");
  });

  it("開始時刻の1時間後を終了時刻の初期値にする", () => {
    expect(endTimeOneHourAfter("09:15")).toBe("10:15");
    expect(endTimeOneHourAfter("23:30")).toBe("00:30");
    expect(endTimeOneHourAfter("")).toBe("");
  });

  it("終了日時は同日と日をまたぐ予定の両方を検証する", () => {
    expect(isValidTimeRange("09:00", "10:00")).toBe(true);
    expect(isValidTimeRange("10:00", "10:00")).toBe(false);
    expect(isValidTimeRange("23:30", "00:30")).toBe(false);
    expect(isValidEventRange({ date: "2026-09-01", endDate: "2026-09-02", allDay: false, startTime: "23:30", endTime: "00:30" })).toBe(true);
    expect(isValidEventRange({ date: "2026-09-02", endDate: "2026-09-01", allDay: true, startTime: "", endTime: "" })).toBe(false);
  });

  it("複数日の予定は期間中の各日に表示し、今後の一覧には重複させない", () => {
    const trip = { ...makeEvent("出張", "2026-09-01", "", true), endDate: "2026-09-03" };
    expect(eventsForDate([trip], "2026-08-31")).toHaveLength(0);
    expect(eventsForDate([trip], "2026-09-01")).toHaveLength(1);
    expect(eventsForDate([trip], "2026-09-02")).toHaveLength(1);
    expect(eventsForDate([trip], "2026-09-03")).toHaveLength(1);
    expect(eventsForDate([trip], "2026-09-04")).toHaveLength(0);
    expect(upcomingEvents([trip], new Date(2026, 8, 1), 3)).toHaveLength(1);
  });

  it("日本の祝日名をオフラインデータから取得する", () => {
    const holidays = getHolidayMap(new Date(2026, 6, 1), new Date(2026, 6, 31));
    expect(holidays.get("2026-07-20")).toBe("海の日");
    expect(holidays.has("2026-07-21")).toBe(false);
  });

  it("予定内容の貼り付けでは貼り付け先の日付と識別情報を維持する", () => {
    const source = {
      ...makeEvent("お休み", "2026-07-22", "", true),
      endDate: "2026-07-24",
      title: "お休み",
      annual: true,
      location: "自宅",
      notes: "連絡不要",
      style: { color: "#b49ac7" },
    };
    const target = makeEvent("new-id", "2026-07-29", "09:00");
    const pasted = pasteEventContent(target, copyEventContent(source));

    expect(pasted).toMatchObject({
      id: "new-id",
      date: "2026-07-29",
      endDate: "2026-07-31",
      title: "お休み",
      annual: true,
      allDay: true,
      location: "自宅",
      notes: "連絡不要",
      style: { color: "#b49ac7" },
    });
    expect(pasted.createdAt).toBe(target.createdAt);
  });

  it("ドラッグ移動では識別情報を維持して日付だけを変更する", () => {
    const source = { ...makeEvent("event-1", "2026-07-22"), endDate: "2026-07-24", annual: true };
    const moved = moveEventToDate(source, "2026-07-25", "2026-07-22T01:00:00.000Z");

    expect(moved).toMatchObject({
      id: "event-1",
      date: "2026-07-25",
      endDate: "2026-07-27",
      annual: true,
      createdAt: source.createdAt,
      updatedAt: "2026-07-22T01:00:00.000Z",
    });
    expect(source.date).toBe("2026-07-22");
  });

  it("Ctrlドラッグ複製では内容を保ち新しい識別情報を割り当てる", () => {
    const source = {
      ...makeEvent("event-1", "2026-07-22"),
      endDate: "2026-07-24",
      title: "訪問予定",
      annual: true,
      notes: "資料を持参",
      style: { color: "#b49ac7" },
    };
    const copied = duplicateEventToDate(source, "2026-07-26", "event-2", "2026-07-22T02:00:00.000Z");

    expect(copied).toMatchObject({
      id: "event-2",
      date: "2026-07-26",
      endDate: "2026-07-28",
      title: "訪問予定",
      annual: true,
      notes: "資料を持参",
      createdAt: "2026-07-22T02:00:00.000Z",
      updatedAt: "2026-07-22T02:00:00.000Z",
      style: { color: "#b49ac7" },
    });
    expect(copied.style).not.toBe(source.style);
    expect(source.id).toBe("event-1");
    expect(source.date).toBe("2026-07-22");
  });
});
