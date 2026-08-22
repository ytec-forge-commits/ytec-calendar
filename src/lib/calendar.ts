import { between } from "@gahojin-inc/holiday-japanese";
import { rrulestr } from "rrule";
import type { CalendarEvent, DayCell, EventContent, EventRecurrence, SimpleRecurrence } from "../types";

const pad = (value: number) => String(value).padStart(2, "0");

export function toDateKey(date: Date): string {
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
}

export function fromDateKey(value: string): Date {
  const [year, month, day] = value.split("-").map(Number);
  return new Date(year, month - 1, day);
}

export function monthTitle(date: Date): string {
  return `${date.getFullYear()}年 ${date.getMonth() + 1}月`;
}

export function longDateLabel(date: Date): string {
  return new Intl.DateTimeFormat("ja-JP", {
    month: "long",
    day: "numeric",
    weekday: "long",
  }).format(date);
}

export function getTodayView(now = new Date()): { date: Date; dateKey: string; displayMonth: Date } {
  return {
    date: now,
    dateKey: toDateKey(now),
    displayMonth: new Date(now.getFullYear(), now.getMonth(), 1),
  };
}

export function getMonthCells(displayMonth: Date, today = new Date()): DayCell[] {
  const first = new Date(displayMonth.getFullYear(), displayMonth.getMonth(), 1);
  const start = new Date(first);
  start.setDate(first.getDate() - first.getDay());
  const todayKey = toDateKey(today);

  return Array.from({ length: 42 }, (_, index) => {
    const date = new Date(start);
    date.setDate(start.getDate() + index);
    const dateKey = toDateKey(date);
    return {
      date,
      dateKey,
      isCurrentMonth: date.getMonth() === displayMonth.getMonth(),
      isToday: dateKey === todayKey,
    };
  });
}

export function getHolidayMap(start: Date, end: Date): Map<string, string> {
  return new Map(between(start, end).map((holiday) => [toDateKey(holiday.date), holiday.nameJa]));
}

export function sortEvents(events: CalendarEvent[]): CalendarEvent[] {
  return [...events].sort((a, b) => {
    if (a.date !== b.date) return a.date.localeCompare(b.date);
    if (a.allDay !== b.allDay) return a.allDay ? -1 : 1;
    return a.startTime.localeCompare(b.startTime) || a.title.localeCompare(b.title, "ja");
  });
}

function utcDayNumber(dateKey: string): number {
  const [year, month, day] = dateKey.split("-").map(Number);
  return Math.floor(Date.UTC(year, month - 1, day) / 86_400_000);
}

function addDays(dateKey: string, amount: number): string {
  const [year, month, day] = dateKey.split("-").map(Number);
  const date = new Date(Date.UTC(year, month - 1, day + amount));
  return `${date.getUTCFullYear()}-${pad(date.getUTCMonth() + 1)}-${pad(date.getUTCDate())}`;
}

function monthsBetween(start: string, target: string): number {
  const [startYear, startMonth] = start.split("-").map(Number);
  const [targetYear, targetMonth] = target.split("-").map(Number);
  return (targetYear - startYear) * 12 + targetMonth - startMonth;
}

function sameWeekdayOrdinal(start: string, target: string): boolean {
  const startDate = fromDateKey(start);
  const targetDate = fromDateKey(target);
  return startDate.getDay() === targetDate.getDay()
    && Math.floor((startDate.getDate() - 1) / 7) === Math.floor((targetDate.getDate() - 1) / 7);
}

function simplePatternMatches(start: string, target: string, recurrence: SimpleRecurrence): boolean {
  if (target < start) return false;
  const interval = Math.max(1, recurrence.interval);
  const startDate = fromDateKey(start);
  const targetDate = fromDateKey(target);
  switch (recurrence.frequency) {
    case "daily":
      return (utcDayNumber(target) - utcDayNumber(start)) % interval === 0;
    case "weekly": {
      const startWeek = utcDayNumber(start) - startDate.getDay();
      const targetWeek = utcDayNumber(target) - targetDate.getDay();
      const weekDifference = Math.floor((targetWeek - startWeek) / 7);
      const weekDays = recurrence.weekDays.length ? recurrence.weekDays : [startDate.getDay()];
      return weekDifference >= 0 && weekDifference % interval === 0 && weekDays.includes(targetDate.getDay());
    }
    case "monthly": {
      const difference = monthsBetween(start, target);
      if (difference < 0 || difference % interval !== 0) return false;
      return recurrence.monthlyMode === "weekday-of-month"
        ? sameWeekdayOrdinal(start, target)
        : startDate.getDate() === targetDate.getDate();
    }
    case "yearly":
      return (targetDate.getFullYear() - startDate.getFullYear()) % interval === 0
        && startDate.getMonth() === targetDate.getMonth()
        && startDate.getDate() === targetDate.getDate();
  }
}

function occurrenceNumber(start: string, target: string, recurrence: SimpleRecurrence): number {
  if (recurrence.frequency === "daily") {
    return Math.floor((utcDayNumber(target) - utcDayNumber(start)) / Math.max(1, recurrence.interval)) + 1;
  }
  let count = 0;
  let cursor = start;
  while (cursor <= target) {
    if (simplePatternMatches(start, cursor, recurrence)) count += 1;
    cursor = addDays(cursor, 1);
  }
  return count;
}

function simpleRecurrenceMatches(event: CalendarEvent, dateKey: string, recurrence: SimpleRecurrence): boolean {
  if (recurrence.excludedDates.includes(dateKey) || !simplePatternMatches(event.date, dateKey, recurrence)) return false;
  if (recurrence.end.type === "until" && dateKey > recurrence.end.date) return false;
  if (recurrence.end.type === "count" && occurrenceNumber(event.date, dateKey, recurrence) > recurrence.end.count) return false;
  return true;
}

function dateKeyInTimeZone(date: Date, timeZone: string): string {
  const parts = new Intl.DateTimeFormat("en-CA", {
    timeZone,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  }).formatToParts(date);
  const part = (type: Intl.DateTimeFormatPartTypes) => parts.find((item) => item.type === type)?.value ?? "";
  return `${part("year")}-${part("month")}-${part("day")}`;
}

function googleRecurrenceMatches(event: CalendarEvent, dateKey: string, recurrence: Extract<EventRecurrence, { kind: "google" }>): boolean {
  if (dateKey < event.date || recurrence.excludedDates.includes(dateKey) || recurrence.lines.length === 0) return false;
  try {
    const compactStartDate = event.date.replaceAll("-", "");
    const timeZone = event.allDay ? "UTC" : recurrence.timeZone || "Asia/Tokyo";
    const compactStartTime = (event.startTime || "00:00").replace(":", "");
    const hasStart = recurrence.lines.some((line) => line.toUpperCase().startsWith("DTSTART"));
    const startLine = event.allDay
      ? `DTSTART:${compactStartDate}T000000Z`
      : `DTSTART;TZID=${timeZone}:${compactStartDate}T${compactStartTime}00`;
    const source = [...(hasStart ? [] : [startLine]), ...recurrence.lines].join("\n");
    const rule = rrulestr(source, { forceset: true });
    const [targetYear, targetMonth, targetDay] = dateKey.split("-").map(Number);
    const start = new Date(Date.UTC(targetYear, targetMonth - 1, targetDay) - 36 * 60 * 60 * 1000);
    const end = new Date(Date.UTC(targetYear, targetMonth - 1, targetDay + 1) + 36 * 60 * 60 * 1000);
    return rule.between(start, end, true).some((date) => dateKeyInTimeZone(date, timeZone) === dateKey);
  } catch {
    return dateKey === event.date;
  }
}

export function recurrenceMatches(event: CalendarEvent, dateKey: string): boolean {
  if (!event.recurrence) return event.date === dateKey;
  return event.recurrence.kind === "simple"
    ? simpleRecurrenceMatches(event, dateKey, event.recurrence)
    : googleRecurrenceMatches(event, dateKey, event.recurrence);
}

export function isRecurringEvent(event: CalendarEvent): boolean {
  return Boolean(event.recurrence);
}

export function recurrenceLabel(recurrence: EventRecurrence | null): string {
  if (!recurrence) return "繰り返しなし";
  if (recurrence.kind === "google") return "Googleカレンダーの繰り返し";
  const frequency = { daily: "日", weekly: "週", monthly: "か月", yearly: "年" }[recurrence.frequency];
  const prefix = recurrence.interval === 1 ? "毎" : `${recurrence.interval}`;
  return recurrence.interval === 1 ? `毎${frequency}` : `${prefix}${frequency}ごと`;
}

export function eventsForDate(events: CalendarEvent[], dateKey: string): CalendarEvent[] {
  const exceptionKeys = new Set(events.flatMap((event) => event.recurrenceException
    ? [`${event.recurrenceException.masterId}:${event.recurrenceException.originalDate}`]
    : []));
  return sortEvents(events.flatMap((event) => {
    if (event.recurrenceException) return event.date === dateKey ? [event] : [];
    if (!event.recurrence) return event.date === dateKey ? [event] : [];
    if (exceptionKeys.has(`${event.id}:${dateKey}`)) return [];
    if (recurrenceMatches(event, dateKey)) {
      return [{ ...event, date: dateKey, occurrence: { masterId: event.id, originalDate: dateKey } }];
    }
    return [];
  }));
}

export function upcomingEvents(events: CalendarEvent[], today = new Date(), days = 7): CalendarEvent[] {
  const occurrences: CalendarEvent[] = [];
  for (let offset = 0; offset <= days; offset += 1) {
    const date = new Date(today.getFullYear(), today.getMonth(), today.getDate() + offset);
    occurrences.push(...eventsForDate(events, toDateKey(date)));
  }
  return sortEvents(occurrences);
}

export function shiftMonth(date: Date, amount: number): Date {
  return new Date(date.getFullYear(), date.getMonth() + amount, 1);
}

export function formatEventTime(event: CalendarEvent): string {
  return event.allDay ? "終日" : event.startTime;
}

export function copyEventContent(event: CalendarEvent): EventContent {
  return {
    title: event.title,
    annual: event.annual,
    recurrence: event.recurrence ? structuredClone(event.recurrence) : null,
    allDay: event.allDay,
    startTime: event.startTime,
    endTime: event.endTime,
    location: event.location,
    notes: event.notes,
    style: { ...event.style },
  };
}

export function pasteEventContent(target: CalendarEvent, content: EventContent): CalendarEvent {
  return {
    ...target,
    ...content,
    style: { ...content.style },
  };
}

export function moveEventToDate(event: CalendarEvent, targetDate: string, updatedAt: string): CalendarEvent {
  return {
    ...event,
    date: targetDate,
    occurrence: undefined,
    updatedAt,
  };
}

export function duplicateEventToDate(
  event: CalendarEvent,
  targetDate: string,
  id: string,
  createdAt: string,
): CalendarEvent {
  return {
    ...event,
    id,
    date: targetDate,
    occurrence: undefined,
    recurrenceException: null,
    googleLinks: [],
    syncConflict: null,
    origin: { kind: "local" },
    style: { ...event.style },
    createdAt,
    updatedAt: createdAt,
  };
}

export function isValidTimeRange(startTime: string, endTime: string): boolean {
  return Boolean(startTime && endTime && startTime < endTime);
}
