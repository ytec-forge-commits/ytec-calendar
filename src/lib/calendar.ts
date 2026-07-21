import { between } from "@gahojin-inc/holiday-japanese";
import type { CalendarEvent, DayCell } from "../types";

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

export function eventsForDate(events: CalendarEvent[], dateKey: string): CalendarEvent[] {
  return sortEvents(events.filter((event) => event.date === dateKey));
}

export function upcomingEvents(events: CalendarEvent[], today = new Date(), days = 7): CalendarEvent[] {
  const start = toDateKey(today);
  const endDate = new Date(today.getFullYear(), today.getMonth(), today.getDate() + days);
  const end = toDateKey(endDate);
  return sortEvents(events.filter((event) => event.date >= start && event.date <= end));
}

export function shiftMonth(date: Date, amount: number): Date {
  return new Date(date.getFullYear(), date.getMonth() + amount, 1);
}

export function formatEventTime(event: CalendarEvent): string {
  return event.allDay ? "終日" : event.startTime;
}

export function isValidTimeRange(startTime: string, endTime: string): boolean {
  return Boolean(startTime && endTime && startTime < endTime);
}
