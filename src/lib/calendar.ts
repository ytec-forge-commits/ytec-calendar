import { between } from "@gahojin-inc/holiday-japanese";
import type { CalendarEvent, DayCell, EventContent } from "../types";

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
  const monthAndDay = dateKey.slice(5);
  return sortEvents(events.flatMap((event) => {
    if (event.date === dateKey) return [event];
    if (event.annual && event.date.slice(5) === monthAndDay) return [{ ...event, date: dateKey }];
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
    style: { ...event.style },
    createdAt,
    updatedAt: createdAt,
  };
}

export function isValidTimeRange(startTime: string, endTime: string): boolean {
  return Boolean(startTime && endTime && startTime < endTime);
}
