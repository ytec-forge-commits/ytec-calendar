export const DATA_VERSION = 3 as const;

export type ThemeId =
  | "morning-mist"
  | "forest-breath"
  | "lavender-dusk"
  | "warm-sand"
  | "moon-water"
  | "sky-breeze"
  | "sakura-haze"
  | "birch-morning";

export interface EventStyle {
  color: string;
}

export type WindowDisplayMode = "taskbar" | "tray" | "both";

export type RecurrenceFrequency = "daily" | "weekly" | "monthly" | "yearly";
export type MonthlyRecurrenceMode = "day-of-month" | "weekday-of-month";

export type RecurrenceEnd =
  | { type: "never" }
  | { type: "until"; date: string }
  | { type: "count"; count: number };

export interface SimpleRecurrence {
  kind: "simple";
  frequency: RecurrenceFrequency;
  interval: number;
  weekDays: number[];
  monthlyMode: MonthlyRecurrenceMode;
  end: RecurrenceEnd;
  excludedDates: string[];
}

export interface GoogleRecurrence {
  kind: "google";
  lines: string[];
  timeZone: string;
  excludedDates: string[];
}

export type EventRecurrence = SimpleRecurrence | GoogleRecurrence;

export interface RecurrenceException {
  masterId: string;
  originalDate: string;
}

export interface RecurrenceOccurrence {
  masterId: string;
  originalDate: string;
}

export interface GoogleEventLink {
  accountId: string;
  calendarId: string;
  eventId: string;
  etag: string;
  googleUpdatedAt: string;
  localUpdatedAt: string;
  recurringEventId?: string;
  originalStart?: string;
}

export type EventOrigin =
  | { kind: "local" }
  | { kind: "google"; accountId: string };

export interface SyncConflict {
  accountId: string;
  detectedAt: string;
  reason: "both-edited" | "deleted-on-google";
  message: string;
}

export interface CalendarEvent {
  id: string;
  title: string;
  date: string;
  annual: boolean;
  recurrence: EventRecurrence | null;
  recurrenceException: RecurrenceException | null;
  occurrence?: RecurrenceOccurrence;
  allDay: boolean;
  startTime: string;
  endTime: string;
  location: string;
  notes: string;
  style: EventStyle;
  origin: EventOrigin;
  syncTargets: string[];
  googleLinks: GoogleEventLink[];
  syncConflict: SyncConflict | null;
  createdAt: string;
  updatedAt: string;
}

export type EventContent = Pick<
  CalendarEvent,
  "title" | "annual" | "recurrence" | "allDay" | "startTime" | "endTime" | "location" | "notes" | "style"
>;

export interface DeletedCalendarEvent extends CalendarEvent {
  deletedAt: string;
}

export interface AppSettings {
  theme: ThemeId;
  sidebarCollapsed: boolean;
  windowDisplayMode: WindowDisplayMode;
  google: GoogleIntegrationSettings;
}

export interface GoogleOAuthClient {
  clientId: string;
  clientSecret: string;
  projectId: string;
}

export interface GoogleCalendarOption {
  id: string;
  name: string;
  primary: boolean;
  accessRole: string;
}

export interface GoogleAccount {
  id: string;
  email: string;
  displayName: string;
  calendarId: string;
  calendarName: string;
  syncEnabled: boolean;
  syncToken: string;
  connectedAt: string;
  lastSyncAt: string;
  lastError: string;
  needsReauth: boolean;
}

export interface GoogleIntegrationSettings {
  enabled: boolean;
  client: GoogleOAuthClient | null;
  accounts: GoogleAccount[];
}

export interface AppData {
  version: typeof DATA_VERSION;
  events: CalendarEvent[];
  deletedEvents: DeletedCalendarEvent[];
  settings: AppSettings;
}

export interface DayCell {
  date: Date;
  dateKey: string;
  isCurrentMonth: boolean;
  isToday: boolean;
}

export interface ThemeOption {
  id: ThemeId;
  name: string;
  description: string;
  colors: [string, string, string];
}

export const DEFAULT_EVENT_STYLE: EventStyle = {
  color: "#78a88f",
};

export function createEmptyEvent(id: string, date: string, timestamp: string): CalendarEvent {
  return {
    id,
    title: "",
    date,
    annual: false,
    recurrence: null,
    recurrenceException: null,
    allDay: false,
    startTime: "09:00",
    endTime: "10:00",
    location: "",
    notes: "",
    style: structuredClone(DEFAULT_EVENT_STYLE),
    origin: { kind: "local" },
    syncTargets: [],
    googleLinks: [],
    syncConflict: null,
    createdAt: timestamp,
    updatedAt: timestamp,
  };
}

export const DEFAULT_DATA: AppData = {
  version: DATA_VERSION,
  events: [],
  deletedEvents: [],
  settings: {
    theme: "morning-mist",
    sidebarCollapsed: false,
    windowDisplayMode: "taskbar",
    google: {
      enabled: false,
      client: null,
      accounts: [],
    },
  },
};

export const DEFAULT_SIMPLE_RECURRENCE: SimpleRecurrence = {
  kind: "simple",
  frequency: "yearly",
  interval: 1,
  weekDays: [],
  monthlyMode: "day-of-month",
  end: { type: "never" },
  excludedDates: [],
};

export const THEMES: ThemeOption[] = [
  {
    id: "morning-mist",
    name: "朝もや",
    description: "淡い空色と若葉色",
    colors: ["#e9f2f1", "#dbeae4", "#87ab9d"],
  },
  {
    id: "forest-breath",
    name: "森の息吹",
    description: "深呼吸したくなる緑",
    colors: ["#e8efe6", "#d2e0d0", "#6f9173"],
  },
  {
    id: "lavender-dusk",
    name: "藤の夕暮れ",
    description: "やわらかな藤色",
    colors: ["#f0edf5", "#e2dced", "#8e80a6"],
  },
  {
    id: "warm-sand",
    name: "陽だまり",
    description: "穏やかな砂色と杏色",
    colors: ["#f6f0e5", "#eee1cf", "#b38b64"],
  },
  {
    id: "moon-water",
    name: "月夜の水面",
    description: "静かな藍と月明かり",
    colors: ["#202d3a", "#293c4b", "#89aeb5"],
  },
  {
    id: "sky-breeze",
    name: "空のそよ風",
    description: "澄んだ空と薄雲の青",
    colors: ["#e8f1f7", "#d8e8f3", "#739bb7"],
  },
  {
    id: "sakura-haze",
    name: "桜かすみ",
    description: "やさしい桜色と白",
    colors: ["#f7edef", "#f1dfe3", "#b9838e"],
  },
  {
    id: "birch-morning",
    name: "白樺の朝",
    description: "清らかな白と淡い木肌",
    colors: ["#f3f2ed", "#e6e2d7", "#9a927e"],
  },
];
