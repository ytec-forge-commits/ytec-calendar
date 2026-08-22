import { useCallback, useEffect, useMemo, useRef, useState, type CSSProperties, type DragEvent as ReactDragEvent, type FormEvent, type MouseEvent as ReactMouseEvent } from "react";
import {
  copyEventContent,
  duplicateEventToDate,
  eventsForDate,
  formatEventTime,
  getHolidayMap,
  getMonthCells,
  getTodayView,
  isRecurringEvent,
  isValidTimeRange,
  longDateLabel,
  monthTitle,
  moveEventToDate,
  pasteEventContent,
  recurrenceLabel,
  shiftMonth,
  toDateKey,
  upcomingEvents,
} from "./lib/calendar";
import { getAutoStartState, getDataDirectory, loadAppData, saveAppData, setAutoStartState, setSidebarWindowMode, setWindowDisplayMode, syncGoogleCalendars } from "./lib/store";
import {
  createEmptyEvent,
  DEFAULT_SIMPLE_RECURRENCE,
  THEMES,
  type AppData,
  type CalendarEvent,
  type EventContent,
  type EventStyle,
  type GoogleAccount,
  type ThemeId,
} from "./types";
import koyomadoLogo from "./assets/koyomado-logo.png";
import { GoogleSettings } from "./GoogleSettings";

const WEEKDAYS = ["日", "月", "火", "水", "木", "金", "土"];
const EVENT_COLORS = ["#78a88f", "#83a9c2", "#b49ac7", "#d2a36f", "#d7867f", "#92a86c"];
const CALENDAR_EVENT_DRAG_TYPE = "application/x-koyomado-event";

type CalendarContextMenu =
  | { kind: "event"; event: CalendarEvent; x: number; y: number }
  | { kind: "day"; date: string; x: number; y: number };

interface CalendarDragState {
  eventId: string;
  event: CalendarEvent;
  sourceDate: string;
  targetDate: string | null;
  copy: boolean;
}

type RecurrenceEditScope = "occurrence" | "series";

function styleForEvent(style: EventStyle): CSSProperties {
  return {
    "--event-color": style.color,
  } as CSSProperties;
}

function createDraft(date: string, event?: CalendarEvent): CalendarEvent {
  if (event) return structuredClone(event);
  const now = new Date().toISOString();
  return createEmptyEvent(crypto.randomUUID(), date, now);
}

function App() {
  const [today, setToday] = useState(() => new Date());
  const todayKey = toDateKey(today);
  const [data, setData] = useState<AppData | null>(null);
  const [displayMonth, setDisplayMonth] = useState(() => new Date(today.getFullYear(), today.getMonth(), 1));
  const [selectedDate, setSelectedDate] = useState(todayKey);
  const [editing, setEditing] = useState<CalendarEvent | "new" | null>(null);
  const [copiedContent, setCopiedContent] = useState<EventContent | null>(null);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [contextMenu, setContextMenu] = useState<CalendarContextMenu | null>(null);
  const [agendaDate, setAgendaDate] = useState<string | null>(null);
  const [dragState, setDragState] = useState<CalendarDragState | null>(null);
  const [toast, setToast] = useState("");
  const [loadError, setLoadError] = useState("");
  const [syncBusy, setSyncBusy] = useState(false);
  const dataRef = useRef<AppData | null>(null);
  const interactionActiveRef = useRef(false);
  const syncBusyRef = useRef(false);

  useEffect(() => {
    loadAppData().then(setData).catch((error: unknown) => setLoadError(String(error)));
  }, []);

  useEffect(() => {
    dataRef.current = data;
  }, [data]);

  useEffect(() => {
    interactionActiveRef.current = Boolean(editing || settingsOpen || contextMenu || agendaDate || dragState);
  }, [agendaDate, contextMenu, dragState, editing, settingsOpen]);

  const runGoogleSync = useCallback(async (announce = false, allowActiveSettings = false) => {
    const current = dataRef.current;
    if (!current?.settings.google.enabled || !current.settings.google.client || !current.settings.google.accounts.some((account) => account.syncEnabled)) {
      if (announce) setToast("同期するGoogleアカウントがありません");
      return false;
    }
    if (syncBusyRef.current) {
      if (announce) setToast("Googleカレンダーと同期中です");
      return false;
    }
    if (interactionActiveRef.current && !(allowActiveSettings && settingsOpen)) {
      if (announce) setToast("予定の編集を閉じてから同期してください");
      return false;
    }

    syncBusyRef.current = true;
    setSyncBusy(true);
    try {
      const result = await syncGoogleCalendars();
      dataRef.current = result.data;
      setData(result.data);
      const changed = result.summary.pulled + result.summary.pushed + result.summary.deleted;
      if (result.summary.conflicts > 0) {
        setToast(`同期しました。${result.summary.conflicts}件の競合を2件に分けて残しました`);
      } else if (result.summary.warnings.length > 0) {
        setToast(`同期に注意事項があります: ${result.summary.warnings[0]}`);
      } else if (announce) {
        setToast(changed > 0 ? `Googleカレンダーと同期しました（変更 ${changed}件）` : "Googleカレンダーは最新です");
      }
      return true;
    } catch (error) {
      setToast(`Googleカレンダーと同期できませんでした: ${String(error)}`);
      return false;
    } finally {
      syncBusyRef.current = false;
      setSyncBusy(false);
    }
  }, [settingsOpen]);

  useEffect(() => {
    let midnightTimer = 0;
    const refreshToday = () => {
      const current = getTodayView();
      setToday((previous) => toDateKey(previous) === current.dateKey ? previous : current.date);
    };
    const scheduleMidnightRefresh = () => {
      const now = new Date();
      const nextDay = new Date(now.getFullYear(), now.getMonth(), now.getDate() + 1);
      midnightTimer = window.setTimeout(() => {
        refreshToday();
        scheduleMidnightRefresh();
      }, Math.max(1000, nextDay.getTime() - now.getTime() + 250));
    };
    const refreshWhenVisible = () => {
      if (!document.hidden) refreshToday();
    };

    scheduleMidnightRefresh();
    window.addEventListener("focus", refreshToday);
    document.addEventListener("visibilitychange", refreshWhenVisible);
    return () => {
      window.clearTimeout(midnightTimer);
      window.removeEventListener("focus", refreshToday);
      document.removeEventListener("visibilitychange", refreshWhenVisible);
    };
  }, []);

  useEffect(() => {
    if (!data) return;
    document.documentElement.dataset.theme = data.settings.theme;
  }, [data]);

  const sidebarCollapsed = data?.settings.sidebarCollapsed;
  useEffect(() => {
    if (sidebarCollapsed === undefined) return;
    void setSidebarWindowMode(sidebarCollapsed).catch((error: unknown) => {
      setToast(`ウィンドウ幅を調整できませんでした: ${String(error)}`);
    });
  }, [sidebarCollapsed]);

  const windowDisplayMode = data?.settings.windowDisplayMode;
  useEffect(() => {
    if (!windowDisplayMode) return;
    void setWindowDisplayMode(windowDisplayMode).catch((error: unknown) => {
      setToast(`表示先を変更できませんでした: ${String(error)}`);
    });
  }, [windowDisplayMode]);

  const googleSyncKey = data?.settings.google.enabled && data.settings.google.client
    ? data.settings.google.accounts.filter((account) => account.syncEnabled).map((account) => `${account.id}:${account.calendarId}`).join("|")
    : "";
  useEffect(() => {
    if (!googleSyncKey) return;
    const syncWhenAvailable = () => {
      if (!document.hidden) void runGoogleSync(false);
    };
    const initialTimer = window.setTimeout(syncWhenAvailable, 800);
    const interval = window.setInterval(syncWhenAvailable, 60_000);
    window.addEventListener("focus", syncWhenAvailable);
    document.addEventListener("visibilitychange", syncWhenAvailable);
    return () => {
      window.clearTimeout(initialTimer);
      window.clearInterval(interval);
      window.removeEventListener("focus", syncWhenAvailable);
      document.removeEventListener("visibilitychange", syncWhenAvailable);
    };
  }, [googleSyncKey, runGoogleSync]);

  useEffect(() => {
    if (!toast) return;
    const timer = window.setTimeout(() => setToast(""), 2600);
    return () => window.clearTimeout(timer);
  }, [toast]);

  useEffect(() => {
    if (!contextMenu) return;
    const closeMenu = () => setContextMenu(null);
    const closeOnEscape = (keyboardEvent: KeyboardEvent) => {
      if (keyboardEvent.key === "Escape") closeMenu();
    };
    window.addEventListener("click", closeMenu);
    window.addEventListener("keydown", closeOnEscape);
    window.addEventListener("resize", closeMenu);
    return () => {
      window.removeEventListener("click", closeMenu);
      window.removeEventListener("keydown", closeOnEscape);
      window.removeEventListener("resize", closeMenu);
    };
  }, [contextMenu]);

  const persist = useCallback(async (next: AppData, message?: string) => {
    if (syncBusyRef.current) {
      setToast("Googleカレンダーとの同期完了後にもう一度お試しください");
      return false;
    }
    try {
      await saveAppData(next);
      dataRef.current = next;
      setData(next);
      if (message) setToast(message);
      return true;
    } catch (error) {
      setToast(`保存できませんでした: ${String(error)}`);
      return false;
    }
  }, []);

  const monthCells = useMemo(() => getMonthCells(displayMonth, today), [displayMonth, today]);
  const holidayMap = useMemo(
    () => getHolidayMap(monthCells[0].date, monthCells[monthCells.length - 1].date),
    [monthCells],
  );
  const todayEvents = useMemo(() => (data ? eventsForDate(data.events, todayKey) : []), [data, todayKey]);
  const upcoming = useMemo(() => (data ? upcomingEvents(data.events, today, 7) : []), [data, today]);

  const openNewEvent = (date = selectedDate) => {
    if (syncBusyRef.current) {
      setToast("Googleカレンダーとの同期完了後に予定を追加できます");
      return;
    }
    setSelectedDate(date);
    setEditing("new");
  };

  const openEventEditor = (event: CalendarEvent) => {
    if (syncBusyRef.current) {
      setToast("Googleカレンダーとの同期完了後に予定を編集できます");
      return;
    }
    setSelectedDate(event.date);
    setEditing(event);
  };

  const saveEvent = async (event: CalendarEvent, scope: RecurrenceEditScope = "series") => {
    if (!data) return;
    const now = new Date().toISOString();
    const occurrence = event.occurrence ?? (event.recurrenceException
      ? { masterId: event.recurrenceException.masterId, originalDate: event.recurrenceException.originalDate }
      : null);
    const master = occurrence ? data.events.find((item) => item.id === occurrence.masterId) : undefined;
    let nextEvents: CalendarEvent[];
    let exists = data.events.some((item) => item.id === event.id);

    if (occurrence && master && scope === "occurrence") {
      const storedException = Boolean(event.recurrenceException && exists);
      const nextEvent: CalendarEvent = {
        ...event,
        id: storedException ? event.id : crypto.randomUUID(),
        annual: false,
        recurrence: null,
        recurrenceException: occurrence,
        occurrence: undefined,
        syncConflict: null,
        origin: master.origin,
        syncTargets: [...master.syncTargets],
        googleLinks: storedException
          ? event.googleLinks
          : master.googleLinks.map((link) => ({
              ...link,
              eventId: "",
              etag: "",
              googleUpdatedAt: "",
              localUpdatedAt: now,
              recurringEventId: link.eventId,
              originalStart: occurrence.originalDate,
            })),
        updatedAt: now,
      };
      nextEvents = storedException
        ? data.events.map((item) => item.id === event.id ? nextEvent : item)
        : [...data.events, nextEvent];
      exists = storedException;
    } else if (occurrence && master) {
      const nextMaster: CalendarEvent = {
        ...master,
        ...event,
        id: master.id,
        date: master.date,
        recurrenceException: null,
        occurrence: undefined,
        googleLinks: master.googleLinks,
        origin: master.origin,
        syncConflict: null,
        updatedAt: now,
      };
      nextEvents = data.events
        .filter((item) => item.id !== event.id || item.id === master.id)
        .map((item) => {
          if (item.id === master.id) return nextMaster;
          if (item.recurrenceException?.masterId === master.id) {
            return { ...item, syncTargets: [...nextMaster.syncTargets] };
          }
          return item;
        });
      exists = true;
    } else {
      const nextEvent = { ...event, occurrence: undefined, syncConflict: null, updatedAt: now };
      nextEvents = exists
        ? data.events.map((item) => item.id === event.id ? nextEvent : item)
        : [...data.events, nextEvent];
    }

    const next = { ...data, events: nextEvents };
    const saved = await persist(next, exists ? "予定を更新しました" : "予定を追加しました");
    if (!saved) return;
    setSelectedDate(event.date);
    setDisplayMonth(new Date(Number(event.date.slice(0, 4)), Number(event.date.slice(5, 7)) - 1, 1));
    setEditing(null);
    window.setTimeout(() => void runGoogleSync(false), 80);
  };

  const deleteEvent = async (event: CalendarEvent, scope: RecurrenceEditScope = "series") => {
    const deleteRange = event.occurrence || event.recurrenceException
      ? (scope === "occurrence" ? "この回だけ" : "シリーズごと")
      : "";
    if (!data || !window.confirm(`「${event.title}」を${deleteRange}削除しますか？\n削除した予定はデータ内に保管されます。`)) return;
    const now = new Date().toISOString();
    const occurrence = event.occurrence ?? (event.recurrenceException
      ? { masterId: event.recurrenceException.masterId, originalDate: event.recurrenceException.originalDate }
      : null);
    const master = occurrence ? data.events.find((item) => item.id === occurrence.masterId) : undefined;
    let events: CalendarEvent[];
    let deletedEvent = event;

    if (occurrence && master && scope === "occurrence") {
      const recurrence = master.recurrence
        ? { ...master.recurrence, excludedDates: [...new Set([...master.recurrence.excludedDates, occurrence.originalDate])] }
        : null;
      events = data.events
        .filter((item) => item.id !== event.id || item.id === master.id)
        .map((item) => item.id === master.id ? { ...item, recurrence } : item);
      deletedEvent = event.recurrenceException
        ? { ...event, occurrence: undefined }
        : {
            ...event,
            id: crypto.randomUUID(),
            annual: false,
            recurrence: null,
            recurrenceException: occurrence,
            occurrence: undefined,
            googleLinks: master.googleLinks.map((link) => ({
              ...link,
              eventId: "",
              recurringEventId: link.eventId,
              originalStart: occurrence.originalDate,
            })),
          };
    } else if (occurrence && master) {
      events = data.events.filter((item) => item.id !== master.id && item.recurrenceException?.masterId !== master.id);
      deletedEvent = master;
    } else {
      events = data.events.filter((item) => item.id !== event.id);
    }

    const next = {
      ...data,
      events,
      deletedEvents: [...data.deletedEvents, { ...deletedEvent, deletedAt: now }],
    };
    const saved = await persist(next, "予定を削除しました");
    if (!saved) return;
    setEditing(null);
    window.setTimeout(() => void runGoogleSync(false), 80);
  };

  const copyEvent = (event: CalendarEvent) => {
    setCopiedContent(copyEventContent(event));
    setEditing(null);
    setContextMenu(null);
    setToast(`「${event.title.trim()}」の内容をコピーしました`);
  };

  const pasteCopiedEvent = async (date: string) => {
    if (!copiedContent) return;
    setContextMenu(null);
    await saveEvent(pasteEventContent(createDraft(date), copiedContent));
  };

  const goToToday = () => {
    const current = getTodayView();
    setToday(current.date);
    setDisplayMonth(current.displayMonth);
    setSelectedDate(current.dateKey);
    setContextMenu(null);
  };

  const startEventDrag = (dragEvent: ReactDragEvent<HTMLButtonElement>, event: CalendarEvent) => {
    dragEvent.stopPropagation();
    dragEvent.dataTransfer.effectAllowed = "copyMove";
    dragEvent.dataTransfer.setData(CALENDAR_EVENT_DRAG_TYPE, event.id);
    dragEvent.dataTransfer.setData("text/plain", event.id);
    setContextMenu(null);
    setDragState({
      eventId: event.id,
      event: structuredClone(event),
      sourceDate: event.date,
      targetDate: null,
      copy: dragEvent.ctrlKey,
    });
  };

  const updateDragTarget = (dragEvent: ReactDragEvent<HTMLDivElement>, targetDate: string) => {
    if (!dragState) return;
    dragEvent.preventDefault();
    dragEvent.stopPropagation();
    if (targetDate === dragState.sourceDate) {
      dragEvent.dataTransfer.dropEffect = "none";
      if (dragState.targetDate !== null) setDragState((current) => current ? { ...current, targetDate: null } : null);
      return;
    }
    const copy = dragEvent.ctrlKey;
    dragEvent.dataTransfer.dropEffect = copy ? "copy" : "move";
    if (dragState.targetDate !== targetDate || dragState.copy !== copy) {
      setDragState((current) => current ? { ...current, targetDate, copy } : null);
    }
  };

  const leaveDragTarget = (dragEvent: ReactDragEvent<HTMLDivElement>, targetDate: string) => {
    if (!dragState || dragState.targetDate !== targetDate) return;
    const nextTarget = dragEvent.relatedTarget;
    if (nextTarget instanceof Node && dragEvent.currentTarget.contains(nextTarget)) return;
    setDragState((current) => current ? { ...current, targetDate: null } : null);
  };

  const dropEventOnDate = async (dropEvent: ReactDragEvent<HTMLDivElement>, targetDate: string) => {
    dropEvent.preventDefault();
    dropEvent.stopPropagation();
    if (!data) return;
    const eventId = dropEvent.dataTransfer.getData(CALENDAR_EVENT_DRAG_TYPE) || dragState?.eventId;
    const copy = dropEvent.ctrlKey;
    setDragState(null);
    if (!eventId) return;
    const source = dragState?.eventId === eventId
      ? dragState.event
      : data.events.find((event) => event.id === eventId);
    if (!source) {
      setToast("移動する予定を確認できませんでした");
      return;
    }
    if (source.date === targetDate) {
      setToast("別の日へドロップしてください");
      return;
    }

    const now = new Date().toISOString();
    const targetLabel = formatDayMenuDate(targetDate);
    const occurrence = source.occurrence ?? (source.recurrenceException
      ? { masterId: source.recurrenceException.masterId, originalDate: source.recurrenceException.originalDate }
      : null);
    let next: AppData;
    if (copy) {
      const copied = duplicateEventToDate(source, targetDate, crypto.randomUUID(), now);
      next = {
        ...data,
        events: [...data.events, {
          ...copied,
          annual: false,
          recurrence: null,
          recurrenceException: null,
        }],
      };
    } else if (occurrence) {
      const storedException = data.events.find((event) => event.id === source.id && event.recurrenceException);
      if (storedException) {
        next = {
          ...data,
          events: data.events.map((event) => event.id === storedException.id ? moveEventToDate(storedException, targetDate, now) : event),
        };
      } else {
        const master = data.events.find((event) => event.id === occurrence.masterId);
        if (!master) {
          setToast("繰り返し予定の元データを確認できませんでした");
          return;
        }
        const exception: CalendarEvent = {
          ...source,
          id: crypto.randomUUID(),
          date: targetDate,
          annual: false,
          recurrence: null,
          recurrenceException: occurrence,
          occurrence: undefined,
          googleLinks: master.googleLinks.map((link) => ({
            ...link,
            eventId: "",
            etag: "",
            googleUpdatedAt: "",
            localUpdatedAt: now,
            recurringEventId: link.eventId,
            originalStart: occurrence.originalDate,
          })),
          createdAt: now,
          updatedAt: now,
        };
        next = { ...data, events: [...data.events, exception] };
      }
    } else {
      next = { ...data, events: data.events.map((event) => event.id === source.id ? moveEventToDate(event, targetDate, now) : event) };
    }
    const saved = await persist(next, `「${source.title}」を${targetLabel}へ${copy ? "コピー" : "移動"}しました`);
    if (saved) {
      setSelectedDate(targetDate);
      window.setTimeout(() => void runGoogleSync(false), 80);
    }
  };

  const openContextMenu = (
    mouseEvent: ReactMouseEvent,
    target: { kind: "event"; event: CalendarEvent } | { kind: "day"; date: string },
  ) => {
    mouseEvent.preventDefault();
    mouseEvent.stopPropagation();
    const menuWidth = 224;
    const menuHeight = target.kind === "event"
      ? (target.event.occurrence || target.event.recurrenceException ? 220 : 165)
      : 142;
    setContextMenu({
      ...target,
      x: Math.max(8, Math.min(mouseEvent.clientX, window.innerWidth - menuWidth - 8)),
      y: Math.max(8, Math.min(mouseEvent.clientY, window.innerHeight - menuHeight - 8)),
    });
  };

  const selectDate = (date: string, events: CalendarEvent[]) => {
    setSelectedDate(date);
    setContextMenu(null);
    if (events.length > 0) {
      setAgendaDate(date);
    } else {
      openNewEvent(date);
    }
  };

  const updateSettings = async (settings: AppData["settings"]) => {
    if (!data) return;
    const connectedAccountIds = new Set(settings.google.accounts.map((account) => account.id));
    const removeDisconnectedLinks = (event: CalendarEvent): CalendarEvent => {
      const origin = event.origin.kind === "google" && !connectedAccountIds.has(event.origin.accountId)
        ? { kind: "local" as const }
        : event.origin;
      return {
        ...event,
        origin,
        syncTargets: event.syncTargets.filter((accountId) => connectedAccountIds.has(accountId)),
        googleLinks: event.googleLinks.filter((link) => connectedAccountIds.has(link.accountId)),
        syncConflict: event.syncConflict && connectedAccountIds.has(event.syncConflict.accountId) ? event.syncConflict : null,
      };
    };
    await persist({
      ...data,
      events: data.events.map(removeDisconnectedLinks),
      deletedEvents: data.deletedEvents.map((event) => ({ ...removeDisconnectedLinks(event), deletedAt: event.deletedAt })),
      settings,
    }, "設定を保存しました");
  };

  if (loadError) {
    return (
      <main className="fatal-state">
        <p className="eyebrow">koyomado</p>
        <h1>カレンダーを開けませんでした</h1>
        <p>{loadError}</p>
        <p>アプリのフォルダーへ書き込めるか確認して、もう一度起動してください。</p>
      </main>
    );
  }

  if (!data) {
    return (
      <main className="loading-state" aria-live="polite">
        <span className="loading-orb" />
        <p>カレンダーを整えています…</p>
      </main>
    );
  }

  return (
    <div className="app-shell">
      <header className="topbar">
        <div className="brand-block">
          <img className="brand-mark" src={koyomadoLogo} alt="" aria-hidden="true" />
          <div>
            <h1>koyomado</h1>
          </div>
        </div>

        <nav className="month-nav" aria-label="月の移動">
          <button className="icon-button" onClick={() => setDisplayMonth(shiftMonth(displayMonth, -1))} aria-label="前の月">‹</button>
          <button className="today-button" onClick={goToToday}>今日</button>
          <button className="icon-button" onClick={() => setDisplayMonth(shiftMonth(displayMonth, 1))} aria-label="次の月">›</button>
          <h2>{monthTitle(displayMonth)}</h2>
        </nav>

        <div className="top-actions">
          {googleSyncKey && (
            <button className="secondary-button sync-button" onClick={() => void runGoogleSync(true)} disabled={syncBusy} aria-label="Googleカレンダーと同期" title="Googleカレンダーと同期">
              <span className={syncBusy ? "sync-icon spinning" : "sync-icon"} aria-hidden="true">↻</span><span className="action-label">同期</span>
            </button>
          )}
          <button
            className="secondary-button sidebar-toggle-button"
            onClick={() => void updateSettings({ ...data.settings, sidebarCollapsed: !data.settings.sidebarCollapsed })}
            aria-label={data.settings.sidebarCollapsed ? "サイドバーを開く" : "サイドバーを折りたたむ"}
            title={data.settings.sidebarCollapsed ? "サイドバーを開く" : "サイドバーを折りたたむ"}
          >
            <span aria-hidden="true">{data.settings.sidebarCollapsed ? "▥" : "▤"}</span>
          </button>
          <button className="secondary-button settings-button" onClick={() => {
            if (syncBusyRef.current) return setToast("Googleカレンダーとの同期完了後に設定を開けます");
            setSettingsOpen(true);
          }} aria-label="表示と起動の設定">
            <span aria-hidden="true">⚙</span><span className="action-label">設定</span>
          </button>
          <button className="primary-button" onClick={() => openNewEvent()} aria-label="予定を追加">
            <span aria-hidden="true">＋</span><span className="action-label">予定を追加</span>
          </button>
        </div>
      </header>

      <main className={data.settings.sidebarCollapsed ? "workspace sidebar-collapsed" : "workspace"}>
        <aside className="sidebar">
          <section className="today-card">
            <p className="section-kicker">TODAY</p>
            <div className="today-heading">
              <span className="today-number">{today.getDate()}</span>
              <div>
                <h2>{longDateLabel(today).replace(`${today.getDate()}日`, "・")}</h2>
                <p>{todayEvents.length ? `${todayEvents.length}件の予定` : "ゆとりのある一日です"}</p>
              </div>
            </div>
            <button className="sidebar-add" onClick={() => openNewEvent(todayKey)}>今日の予定を追加 <span>＋</span></button>
          </section>

          <section className="upcoming-section">
            <div className="section-title-row">
              <h2>これから7日間</h2>
              <span>{upcoming.length}件</span>
            </div>
            {upcoming.length === 0 ? (
              <div className="empty-upcoming"><span aria-hidden="true">○</span><p>直近の予定はありません</p></div>
            ) : (
              <div className="upcoming-list">
                {upcoming.map((event) => (
                  <button key={`${event.id}:${event.occurrence?.originalDate ?? event.date}`} className="upcoming-item" onClick={() => openEventEditor(event)}>
                    <span className="upcoming-dot" style={{ background: event.style.color }} />
                    <span className="upcoming-date">{Number(event.date.slice(5, 7))}/{Number(event.date.slice(8, 10))}</span>
                    <span className="upcoming-info"><strong>{event.title}</strong><small>{isRecurringEvent(event) ? `${recurrenceLabel(event.recurrence)}・${formatEventTime(event)}` : formatEventTime(event)}</small></span>
                  </button>
                ))}
              </div>
            )}
          </section>

          <section className="theme-shortcut">
            <div>
              <p className="section-kicker">BACKGROUND</p>
              <h2>{THEMES.find((theme) => theme.id === data.settings.theme)?.name}</h2>
            </div>
            <div className="mini-swatches" role="group" aria-label="背景テーマを選択">
              {THEMES.map((theme) => (
                <button
                  key={theme.id}
                  className={theme.id === data.settings.theme ? "mini-swatch active" : "mini-swatch"}
                  style={{ background: theme.colors[2] }}
                  onClick={() => updateSettings({ ...data.settings, theme: theme.id })}
                  aria-label={`${theme.name}に変更`}
                  title={theme.name}
                />
              ))}
            </div>
          </section>
        </aside>

        <section className="calendar-panel" aria-label={`${monthTitle(displayMonth)}のカレンダー`}>
          <p id="calendar-drag-help" className="sr-only">予定を別の日へドラッグすると移動します。Ctrlキーを押しながらドラッグするとコピーします。</p>
          <div className="weekday-row">
            {WEEKDAYS.map((day, index) => <div key={day} className={index === 0 ? "sunday" : index === 6 ? "saturday" : ""}>{day}</div>)}
          </div>
          <div className="month-grid">
            {monthCells.map((cell) => {
              const events = eventsForDate(data.events, cell.dateKey);
              const selected = selectedDate === cell.dateKey;
              const holidayName = holidayMap.get(cell.dateKey);
              const visibleEvents = events.length > 2 || (holidayName && events.length > 1) ? events.slice(0, 1) : events;
              const dayOfWeek = cell.date.getDay();
              const dayType = holidayName || dayOfWeek === 0 ? " holiday" : dayOfWeek === 6 ? " saturday-day" : "";
              const isDropTarget = dragState?.targetDate === cell.dateKey;
              return (
                <div
                  key={cell.dateKey}
                  className={`day-cell${cell.isCurrentMonth ? "" : " outside"}${cell.isToday ? " today" : ""}${selected ? " selected" : ""}${dayType}${isDropTarget ? ` drop-target ${dragState.copy ? "drop-copy-target" : "drop-move-target"}` : ""}`}
                  data-drop-label={isDropTarget ? (dragState.copy ? "コピー" : "移動") : undefined}
                  onClick={() => selectDate(cell.dateKey, events)}
                  onContextMenu={(mouseEvent) => openContextMenu(mouseEvent, { kind: "day", date: cell.dateKey })}
                  onDragEnter={(dragEvent) => updateDragTarget(dragEvent, cell.dateKey)}
                  onDragOver={(dragEvent) => updateDragTarget(dragEvent, cell.dateKey)}
                  onDragLeave={(dragEvent) => leaveDragTarget(dragEvent, cell.dateKey)}
                  onDrop={(dropEvent) => void dropEventOnDate(dropEvent, cell.dateKey)}
                >
                  <div className="day-cell-head">
                    <button
                      className="date-number"
                      onClick={(clickEvent) => { clickEvent.stopPropagation(); selectDate(cell.dateKey, events); }}
                      aria-label={`${cell.date.getMonth() + 1}月${cell.date.getDate()}日を選択`}
                    >
                      {cell.date.getDate()}
                    </button>
                    <button className="cell-add" onClick={(event) => { event.stopPropagation(); openNewEvent(cell.dateKey); }} aria-label={`${cell.date.getMonth() + 1}月${cell.date.getDate()}日に予定を追加`}>＋</button>
                  </div>
                  {holidayName && <span className="holiday-name">{holidayName}</span>}
                  <div className="event-stack">
                    {visibleEvents.map((event) => (
                      <button
                        key={`${event.id}:${event.occurrence?.originalDate ?? event.date}`}
                        className={dragState?.eventId === event.id ? "event-chip dragging" : "event-chip"}
                        style={styleForEvent(event.style)}
                        draggable
                        onDragStart={(dragEvent) => startEventDrag(dragEvent, event)}
                        onDragEnd={() => setDragState(null)}
                        onClick={(clickEvent) => { clickEvent.stopPropagation(); openEventEditor(event); }}
                        onContextMenu={(mouseEvent) => openContextMenu(mouseEvent, { kind: "event", event })}
                        aria-describedby="calendar-drag-help"
                        title={`${isRecurringEvent(event) ? `${recurrenceLabel(event.recurrence)} ` : ""}${formatEventTime(event)} ${event.title}\nドラッグで移動 / Ctrl＋ドラッグでコピー`}
                      >
                        {isRecurringEvent(event) && <span className="annual-indicator" aria-label="繰り返し予定">↻</span>}
                        {!event.allDay && <span className="event-time">{event.startTime}</span>}
                        <span className="event-title">{event.title}</span>
                      </button>
                    ))}
                    {events.length > visibleEvents.length && <button className="more-events" onClick={(clickEvent) => { clickEvent.stopPropagation(); selectDate(cell.dateKey, events); }}>ほか{events.length - visibleEvents.length}件</button>}
                  </div>
                </div>
              );
            })}
          </div>
        </section>
      </main>

      {editing && (
        <EventEditor
          key={editing === "new" ? `new-${selectedDate}` : editing.id}
          date={selectedDate}
          event={editing === "new" ? undefined : editing}
          copiedContent={copiedContent}
          googleAccounts={data.settings.google.enabled ? data.settings.google.accounts.filter((account) => account.syncEnabled) : []}
          onClose={() => setEditing(null)}
          onSave={saveEvent}
          onDelete={deleteEvent}
          onCopy={copyEvent}
        />
      )}
      {settingsOpen && (
        <SettingsDialog
          settings={data.settings}
          onChange={updateSettings}
          onSync={() => runGoogleSync(true, true)}
          syncBusy={syncBusy}
          onClose={() => setSettingsOpen(false)}
        />
      )}
      {agendaDate && (
        <DayAgendaDialog
          date={agendaDate}
          events={eventsForDate(data.events, agendaDate)}
          onClose={() => setAgendaDate(null)}
          onAdd={() => { setAgendaDate(null); openNewEvent(agendaDate); }}
          onEdit={(event) => { setAgendaDate(null); openEventEditor(event); }}
        />
      )}
      {contextMenu && (
        <div
          className="calendar-context-menu"
          role="menu"
          style={{ left: contextMenu.x, top: contextMenu.y }}
          onClick={(clickEvent) => clickEvent.stopPropagation()}
        >
          {contextMenu.kind === "event" ? (
            <>
              <p className="context-menu-title" title={contextMenu.event.title}>{contextMenu.event.title}</p>
              <button role="menuitem" onClick={() => copyEvent(contextMenu.event)}>内容をコピー</button>
              <button role="menuitem" onClick={() => { const event = contextMenu.event; setContextMenu(null); openEventEditor(event); }}>編集</button>
              {contextMenu.event.occurrence || contextMenu.event.recurrenceException ? (
                <>
                  <button className="context-menu-delete" role="menuitem" onClick={() => { const event = contextMenu.event; setContextMenu(null); void deleteEvent(event, "occurrence"); }}>この予定のみ削除</button>
                  <button className="context-menu-delete" role="menuitem" onClick={() => { const event = contextMenu.event; setContextMenu(null); void deleteEvent(event, "series"); }}>シリーズ全体を削除</button>
                </>
              ) : (
                <button className="context-menu-delete" role="menuitem" onClick={() => { const event = contextMenu.event; setContextMenu(null); void deleteEvent(event); }}>削除</button>
              )}
            </>
          ) : (
            <>
              <p className="context-menu-title">{formatDayMenuDate(contextMenu.date)}</p>
              <button role="menuitem" disabled={!copiedContent} onClick={() => void pasteCopiedEvent(contextMenu.date)}>
                {copiedContent ? "ここに貼り付け" : "コピーした予定はありません"}
              </button>
              <button role="menuitem" onClick={() => { const date = contextMenu.date; setContextMenu(null); openNewEvent(date); }}>予定を追加</button>
            </>
          )}
        </div>
      )}
      {dragState && (
        <div className="drag-instruction" role="status">
          {dragState.copy ? "コピー先の日付へドロップ（Ctrlを離すと移動）" : "移動先の日付へドロップ（Ctrlを押すとコピー）"}
        </div>
      )}
      {toast && <div className="toast" role="status">{toast}</div>}
    </div>
  );
}

function formatDayMenuDate(date: string): string {
  const [year, month, day] = date.split("-").map(Number);
  const weekday = WEEKDAYS[new Date(year, month - 1, day).getDay()];
  return `${month}月${day}日（${weekday}）`;
}

interface DayAgendaDialogProps {
  date: string;
  events: CalendarEvent[];
  onClose: () => void;
  onAdd: () => void;
  onEdit: (event: CalendarEvent) => void;
}

function DayAgendaDialog({ date, events, onClose, onAdd, onEdit }: DayAgendaDialogProps) {
  useEffect(() => {
    const closeOnEscape = (keyboardEvent: KeyboardEvent) => {
      if (keyboardEvent.key === "Escape") onClose();
    };
    window.addEventListener("keydown", closeOnEscape);
    return () => window.removeEventListener("keydown", closeOnEscape);
  }, [onClose]);

  return (
    <div className="modal-backdrop" onMouseDown={(mouseEvent) => { if (mouseEvent.target === mouseEvent.currentTarget) onClose(); }}>
      <section className="dialog agenda-dialog" aria-modal="true" role="dialog" aria-labelledby="agenda-dialog-title">
        <header className="dialog-header">
          <div><p className="section-kicker">DAY SCHEDULE</p><h2 id="agenda-dialog-title">{formatDayMenuDate(date)}の予定</h2></div>
          <button type="button" className="close-button" onClick={onClose} aria-label="閉じる">×</button>
        </header>
        <div className="agenda-list">
          {events.map((event) => (
            <button key={event.id} className="agenda-event" onClick={() => onEdit(event)}>
              <span className="agenda-event-dot" style={{ background: event.style.color }} />
              <span className="agenda-event-time">{formatEventTime(event)}</span>
              <span className="agenda-event-details">
                <strong title={event.title}>{event.title}</strong>
                {event.location && <small title={event.location}>{event.location}</small>}
                {isRecurringEvent(event) && <span className="annual-badge">{recurrenceLabel(event.recurrence)}</span>}
              </span>
              <span className="agenda-event-arrow" aria-hidden="true">›</span>
            </button>
          ))}
        </div>
        <footer className="dialog-footer"><span /><button type="button" className="primary-button" onClick={onAdd}>この日に予定を追加</button></footer>
      </section>
    </div>
  );
}

interface EventEditorProps {
  date: string;
  event?: CalendarEvent;
  copiedContent: EventContent | null;
  googleAccounts: GoogleAccount[];
  onClose: () => void;
  onSave: (event: CalendarEvent, scope?: RecurrenceEditScope) => Promise<void>;
  onDelete: (event: CalendarEvent, scope?: RecurrenceEditScope) => Promise<void>;
  onCopy: (event: CalendarEvent) => void;
}

function EventEditor({ date, event, copiedContent, googleAccounts, onClose, onSave, onDelete, onCopy }: EventEditorProps) {
  const [draft, setDraft] = useState(() => createDraft(date, event));
  const [error, setError] = useState("");
  const isOccurrence = Boolean(event?.occurrence || event?.recurrenceException);
  const [recurrenceScope, setRecurrenceScope] = useState<RecurrenceEditScope>(isOccurrence ? "occurrence" : "series");

  useEffect(() => {
    const closeOnEscape = (keyboardEvent: KeyboardEvent) => {
      if (keyboardEvent.key === "Escape") onClose();
    };
    window.addEventListener("keydown", closeOnEscape);
    return () => window.removeEventListener("keydown", closeOnEscape);
  }, [onClose]);

  const patchDraft = <K extends keyof CalendarEvent>(key: K, value: CalendarEvent[K]) => {
    setDraft((current) => ({ ...current, [key]: value }));
  };
  const patchStyle = <K extends keyof EventStyle>(key: K, value: EventStyle[K]) => {
    setDraft((current) => ({ ...current, style: { ...current.style, [key]: value } }));
  };

  const setRecurrenceFrequency = (value: string) => {
    if (value === "none") {
      setDraft((current) => ({ ...current, annual: false, recurrence: null }));
      return;
    }
    if (value === "google") return;
    setDraft((current) => {
      const existing = current.recurrence?.kind === "simple"
        ? current.recurrence
        : structuredClone(DEFAULT_SIMPLE_RECURRENCE);
      return {
        ...current,
        annual: value === "yearly",
        recurrence: {
          ...existing,
          kind: "simple",
          frequency: value as typeof existing.frequency,
          weekDays: value === "weekly" && existing.weekDays.length === 0
            ? [new Date(`${current.date}T12:00:00`).getDay()]
            : existing.weekDays,
        },
      };
    });
  };

  const patchSimpleRecurrence = (patch: Partial<Extract<NonNullable<CalendarEvent["recurrence"]>, { kind: "simple" }>>) => {
    setDraft((current) => current.recurrence?.kind === "simple"
      ? { ...current, recurrence: { ...current.recurrence, ...patch } }
      : current);
  };

  const submit = (submitEvent: FormEvent) => {
    submitEvent.preventDefault();
    const title = draft.title.trim();
    if (!title) return setError("予定名を入力してください。");
    if (!draft.date) return setError("日付を選択してください。");
    if (!draft.allDay && !isValidTimeRange(draft.startTime, draft.endTime)) return setError("終了時刻は開始時刻より後にしてください。");
    setError("");
    void onSave({ ...draft, title, location: draft.location.trim(), notes: draft.notes.trim() }, recurrenceScope);
  };

  const copyCurrentContent = () => {
    const title = draft.title.trim();
    if (!title) return setError("コピーする予定名を入力してください。");
    setError("");
    onCopy({ ...draft, title, location: draft.location.trim(), notes: draft.notes.trim() });
  };

  const pasteCopiedContent = () => {
    if (!copiedContent) return;
    setDraft((current) => pasteEventContent(current, copiedContent));
    setError("");
  };

  const sourceGoogleAccountId = draft.origin.kind === "google" ? draft.origin.accountId : null;
  const selectedTargetCount = googleAccounts.filter((account) => draft.syncTargets.includes(account.id)).length;
  const toggleSyncTarget = (accountId: string, enabled: boolean) => {
    if (accountId === sourceGoogleAccountId) return;
    setDraft((current) => ({
      ...current,
      syncTargets: enabled
        ? [...new Set([...current.syncTargets, accountId])]
        : current.syncTargets.filter((target) => target !== accountId),
    }));
  };

  return (
    <div className="modal-backdrop" onMouseDown={(mouseEvent) => { if (mouseEvent.target === mouseEvent.currentTarget) onClose(); }}>
      <form className="dialog event-dialog" onSubmit={submit} aria-modal="true" role="dialog" aria-labelledby="event-dialog-title">
        <header className="dialog-header">
          <div><p className="section-kicker">SCHEDULE</p><h2 id="event-dialog-title">{event ? "予定を編集" : "予定を追加"}</h2></div>
          <button type="button" className="close-button" onClick={onClose} aria-label="閉じる">×</button>
        </header>

        <div className="form-body">
          {draft.syncConflict && (
            <div className="sync-conflict-notice" role="alert">
              <strong>Google同期で内容が競合しました</strong>
              <span>{draft.syncConflict.message}</span>
              <small>この予定を確認して保存すると、競合表示を解消します。</small>
            </div>
          )}
          {!event && copiedContent && (
            <div className="copy-paste-bar">
              <span className="copy-paste-details">
                <small>コピーした予定</small>
                <strong title={copiedContent.title}>{copiedContent.title}</strong>
              </span>
              <button type="button" className="secondary-button paste-button" onClick={pasteCopiedContent}>内容を貼り付け</button>
            </div>
          )}
          <label className="field title-field">
            <span>予定名 <b>必須</b></span>
            <input autoFocus value={draft.title} onChange={(changeEvent) => patchDraft("title", changeEvent.target.value)} placeholder="例：定例ミーティング" maxLength={80} />
          </label>

          <div className="form-row date-time-row">
            <label className="field"><span>日付</span><input type="date" value={draft.date} disabled={isOccurrence && recurrenceScope === "series"} onChange={(changeEvent) => patchDraft("date", changeEvent.target.value)} /></label>
            <label className="toggle-field"><input type="checkbox" checked={draft.allDay} onChange={(changeEvent) => patchDraft("allDay", changeEvent.target.checked)} /><span className="toggle-track" /><span>終日</span></label>
            {!draft.allDay && (
              <div className="time-fields">
                <label className="field"><span>開始</span><input type="time" value={draft.startTime} onChange={(changeEvent) => patchDraft("startTime", changeEvent.target.value)} /></label>
                <span className="time-separator">→</span>
                <label className="field"><span>終了</span><input type="time" value={draft.endTime} onChange={(changeEvent) => patchDraft("endTime", changeEvent.target.value)} /></label>
              </div>
            )}
          </div>

          {isOccurrence && (
            <fieldset className="recurrence-scope-panel">
              <legend>変更する範囲</legend>
              <label><input type="radio" name="recurrence-scope" checked={recurrenceScope === "occurrence"} onChange={() => setRecurrenceScope("occurrence")} />この予定のみ</label>
              <label><input type="radio" name="recurrence-scope" checked={recurrenceScope === "series"} onChange={() => setRecurrenceScope("series")} />繰り返し全体</label>
              {recurrenceScope === "series" && <small>日付の変更は下の周期・曜日で調整します。</small>}
            </fieldset>
          )}

          <fieldset className="recurrence-panel">
            <legend>繰り返し</legend>
            <div className="recurrence-grid">
              <label className="field">
                <span>繰り返し周期</span>
                <select
                  value={draft.recurrence?.kind === "google" ? "google" : draft.recurrence?.frequency ?? "none"}
                  onChange={(changeEvent) => setRecurrenceFrequency(changeEvent.target.value)}
                  disabled={isOccurrence && recurrenceScope === "occurrence"}
                >
                  <option value="none">繰り返さない</option>
                  <option value="daily">毎日</option>
                  <option value="weekly">毎週</option>
                  <option value="monthly">毎月</option>
                  <option value="yearly">毎年（誕生日・記念日）</option>
                  {draft.recurrence?.kind === "google" && <option value="google">Googleカレンダーの繰り返し</option>}
                </select>
              </label>

              {draft.recurrence?.kind === "simple" && !(isOccurrence && recurrenceScope === "occurrence") && (
                <>
                  <label className="field recurrence-interval">
                    <span>間隔</span>
                    <span className="number-with-unit">
                      <input type="number" min="1" max="99" value={draft.recurrence.interval} onChange={(changeEvent) => patchSimpleRecurrence({ interval: Math.max(1, Math.min(99, Number(changeEvent.target.value) || 1)) })} />
                      <small>{draft.recurrence.frequency === "daily" ? "日" : draft.recurrence.frequency === "weekly" ? "週" : draft.recurrence.frequency === "monthly" ? "か月" : "年"}ごと</small>
                    </span>
                  </label>

                  {draft.recurrence.frequency === "weekly" && (
                    <div className="weekday-picker" role="group" aria-label="繰り返す曜日">
                      <span>曜日</span>
                      <div>{WEEKDAYS.map((weekday, index) => {
                        const active = draft.recurrence?.kind === "simple" && draft.recurrence.weekDays.includes(index);
                        return <button key={weekday} type="button" className={active ? "active" : ""} onClick={() => {
                          if (draft.recurrence?.kind !== "simple") return;
                          const next = active ? draft.recurrence.weekDays.filter((day) => day !== index) : [...draft.recurrence.weekDays, index].sort();
                          if (next.length) patchSimpleRecurrence({ weekDays: next });
                        }}>{weekday}</button>;
                      })}</div>
                    </div>
                  )}

                  {draft.recurrence.frequency === "monthly" && (
                    <label className="field">
                      <span>毎月の基準</span>
                      <select value={draft.recurrence.monthlyMode} onChange={(changeEvent) => patchSimpleRecurrence({ monthlyMode: changeEvent.target.value as "day-of-month" | "weekday-of-month" })}>
                        <option value="day-of-month">同じ日付</option>
                        <option value="weekday-of-month">同じ第何曜日</option>
                      </select>
                    </label>
                  )}

                  <label className="field">
                    <span>終了</span>
                    <select value={draft.recurrence.end.type} onChange={(changeEvent) => {
                      const type = changeEvent.target.value;
                      patchSimpleRecurrence({ end: type === "until" ? { type, date: draft.date } : type === "count" ? { type, count: 10 } : { type: "never" } });
                    }}>
                      <option value="never">終了しない</option>
                      <option value="until">終了日を指定</option>
                      <option value="count">回数を指定</option>
                    </select>
                  </label>
                  {draft.recurrence.end.type === "until" && <label className="field"><span>終了日</span><input type="date" min={draft.date} value={draft.recurrence.end.date} onChange={(changeEvent) => patchSimpleRecurrence({ end: { type: "until", date: changeEvent.target.value } })} /></label>}
                  {draft.recurrence.end.type === "count" && <label className="field"><span>回数</span><span className="number-with-unit"><input type="number" min="1" max="999" value={draft.recurrence.end.count} onChange={(changeEvent) => patchSimpleRecurrence({ end: { type: "count", count: Math.max(1, Math.min(999, Number(changeEvent.target.value) || 1)) } })} /><small>回</small></span></label>}
                </>
              )}
            </div>
            <p className="recurrence-summary">{recurrenceLabel(draft.recurrence)}{isOccurrence && recurrenceScope === "occurrence" ? "のうち、この予定だけを変更します。" : "として保存します。"}</p>
            {draft.recurrence?.kind === "google" && <p className="native-note">この繰り返し規則はGoogleカレンダーから取得した内容を維持します。周期自体の変更はGoogleカレンダーで行えます。</p>}
          </fieldset>

          <label className="field"><span>場所 <small>任意</small></span><input value={draft.location} onChange={(changeEvent) => patchDraft("location", changeEvent.target.value)} placeholder="会議室、訪問先など" maxLength={100} /></label>
          <label className="field"><span>メモ <small>任意</small></span><textarea value={draft.notes} onChange={(changeEvent) => patchDraft("notes", changeEvent.target.value)} placeholder="補足や持ち物など" rows={3} maxLength={1000} /></label>

          {googleAccounts.length > 0 && (
            <fieldset className="sync-target-panel">
              <legend>Googleカレンダーへの保存先</legend>
              <div className="sync-target-heading">
                <span>ローカルには常に保存されます。</span>
                {!sourceGoogleAccountId && (
                  <button type="button" className="text-button" onClick={() => {
                    const allSelected = selectedTargetCount === googleAccounts.length;
                    setDraft((current) => ({ ...current, syncTargets: allSelected ? [] : googleAccounts.map((account) => account.id) }));
                  }}>{selectedTargetCount === googleAccounts.length ? "すべて解除" : "すべて選択"}</button>
                )}
              </div>
              <div className="sync-target-list">
                {googleAccounts.map((account) => {
                  const source = account.id === sourceGoogleAccountId;
                  return (
                    <label key={account.id}>
                      <input type="checkbox" checked={source || draft.syncTargets.includes(account.id)} disabled={source} onChange={(changeEvent) => toggleSyncTarget(account.id, changeEvent.target.checked)} />
                      <span><strong>{account.displayName || account.email}</strong><small>{account.calendarName || account.email}{source ? "・Googleから取得" : ""}</small></span>
                    </label>
                  );
                })}
              </div>
            </fieldset>
          )}

          <fieldset className="decoration-panel color-panel">
            <legend>予定の色</legend>
            <div className="color-row">
              <span>背景色</span><div className="color-options">{EVENT_COLORS.map((color) => <button type="button" key={color} className={draft.style.color === color ? "color-dot active" : "color-dot"} style={{ background: color }} onClick={() => patchStyle("color", color)} aria-label={`背景色 ${color}`} />)}</div>
            </div>
            <div className="event-preview" style={styleForEvent(draft.style)}><span>{draft.recurrence ? `${recurrenceLabel(draft.recurrence)}・${draft.allDay ? "終日" : draft.startTime}` : draft.allDay ? "終日" : draft.startTime}</span>{draft.title || "予定のプレビュー"}</div>
          </fieldset>
          {error && <p className="form-error" role="alert">{error}</p>}
        </div>

        <footer className="dialog-footer">
          <div className="footer-left">
            {event && <button type="button" className="danger-button" onClick={() => void onDelete(event, recurrenceScope)}>削除</button>}
            {event && <button type="button" className="secondary-button copy-button" onClick={copyCurrentContent}>内容をコピー</button>}
          </div>
          <div className="footer-actions"><button type="button" className="secondary-button" onClick={onClose}>キャンセル</button><button type="submit" className="primary-button">{event ? "変更を保存" : "予定を追加"}</button></div>
        </footer>
      </form>
    </div>
  );
}

interface SettingsDialogProps {
  settings: AppData["settings"];
  onChange: (settings: AppData["settings"]) => Promise<void>;
  onSync: () => Promise<boolean>;
  syncBusy: boolean;
  onClose: () => void;
}

function SettingsDialog({ settings, onChange, onSync, syncBusy, onClose }: SettingsDialogProps) {
  const [autoStart, setAutoStart] = useState<boolean | null>(null);
  const [autoStartBusy, setAutoStartBusy] = useState(true);
  const [dataDirectory, setDataDirectory] = useState("確認中…");
  const [status, setStatus] = useState("");

  useEffect(() => {
    Promise.all([getAutoStartState(), getDataDirectory()]).then(([enabled, directory]) => {
      setAutoStart(enabled);
      setDataDirectory(directory);
      setAutoStartBusy(false);
    }).catch((error: unknown) => {
      setAutoStartBusy(false);
      setStatus(`設定の確認に失敗しました: ${String(error)}`);
    });
  }, []);

  const toggleAutoStart = async () => {
    if (autoStart === null) return;
    setAutoStartBusy(true);
    try {
      await setAutoStartState(!autoStart);
      setAutoStart(!autoStart);
      setStatus(!autoStart ? "Windows起動時の自動起動を有効にしました。" : "自動起動を無効にしました。");
    } catch (error) {
      setStatus(`自動起動を変更できませんでした: ${String(error)}`);
    } finally {
      setAutoStartBusy(false);
    }
  };

  return (
    <div className="modal-backdrop" onMouseDown={(mouseEvent) => { if (mouseEvent.target === mouseEvent.currentTarget) onClose(); }}>
      <section className="dialog settings-dialog" aria-modal="true" role="dialog" aria-labelledby="settings-dialog-title">
        <header className="dialog-header"><div><p className="section-kicker">PREFERENCES</p><h2 id="settings-dialog-title">表示と起動の設定</h2></div><button className="close-button" onClick={onClose} aria-label="閉じる">×</button></header>
        <div className="settings-body">
          <section className="settings-section">
            <div className="settings-heading"><div><h3>背景テーマ</h3><p>8つの落ち着いた配色から選べます。</p></div></div>
            <div className="theme-grid">
              {THEMES.map((theme) => (
                <button key={theme.id} className={settings.theme === theme.id ? "theme-card active" : "theme-card"} onClick={() => void onChange({ ...settings, theme: theme.id as ThemeId })}>
                  <span className="theme-preview" style={{ background: `linear-gradient(135deg, ${theme.colors[0]}, ${theme.colors[1]})` }}><i style={{ background: theme.colors[2] }} /></span>
                  <span><strong>{theme.name}</strong><small>{theme.description}</small></span>
                  <b className="theme-check">✓</b>
                </button>
              ))}
            </div>
          </section>

          <section className="settings-section native-settings">
            <div className="settings-heading"><div><h3>ウィンドウの表示先</h3><p>最小化時と起動中に表示する場所を選びます。</p></div></div>
            <div className="display-mode-options" role="radiogroup" aria-label="ウィンドウの表示先">
              {([
                ["taskbar", "タスクバーのみ", "標準。閉じるとアプリを終了します。"],
                ["tray", "タスクトレイのみ", "閉じる・最小化でトレイへ隠します。"],
                ["both", "両方", "タスクバーとトレイの両方へ表示します。"],
              ] as const).map(([mode, label, description]) => (
                <button key={mode} type="button" role="radio" aria-checked={settings.windowDisplayMode === mode} className={settings.windowDisplayMode === mode ? "display-mode-card active" : "display-mode-card"} onClick={() => void onChange({ ...settings, windowDisplayMode: mode })}>
                  <span><strong>{label}</strong><small>{description}</small></span><b>✓</b>
                </button>
              ))}
            </div>
          </section>

          <section className="settings-section native-settings">
            <div className="settings-row">
              <div><h3>Windows起動時に自動起動</h3><p>最後に閉じた位置とサイズでカレンダーを開きます。</p></div>
              <button className={`switch ${autoStart ? "on" : ""}`} onClick={() => void toggleAutoStart()} disabled={autoStartBusy || autoStart === null} role="switch" aria-checked={Boolean(autoStart)}><span /></button>
            </div>
            {autoStart === null && !autoStartBusy && <p className="native-note">Webプレビューでは変更できません。Windowsアプリ版で設定してください。</p>}
          </section>

          <GoogleSettings
            google={settings.google}
            onChange={(google) => onChange({ ...settings, google })}
            onSync={onSync}
            syncBusy={syncBusy}
          />

          <section className="settings-section data-location">
            <h3>データの保存場所</h3>
            <p>アプリと一緒に移動できる場所へ保存しています。</p>
            <code>{dataDirectory}</code>
          </section>
          {status && <p className="settings-status" role="status">{status}</p>}
        </div>
        <footer className="dialog-footer"><span /><button className="primary-button" onClick={onClose}>完了</button></footer>
      </section>
    </div>
  );
}

export default App;
