import { useCallback, useEffect, useMemo, useState, type CSSProperties, type FormEvent } from "react";
import {
  copyEventContent,
  eventsForDate,
  formatEventTime,
  getHolidayMap,
  getMonthCells,
  isValidTimeRange,
  longDateLabel,
  monthTitle,
  pasteEventContent,
  shiftMonth,
  toDateKey,
  upcomingEvents,
} from "./lib/calendar";
import { getAutoStartState, getDataDirectory, loadAppData, saveAppData, setAutoStartState, setSidebarWindowMode } from "./lib/store";
import {
  DEFAULT_EVENT_STYLE,
  THEMES,
  type AppData,
  type CalendarEvent,
  type EventContent,
  type EventStyle,
  type ThemeId,
} from "./types";

const WEEKDAYS = ["日", "月", "火", "水", "木", "金", "土"];
const EVENT_COLORS = ["#78a88f", "#83a9c2", "#b49ac7", "#d2a36f", "#d7867f", "#92a86c"];

function styleForEvent(style: EventStyle): CSSProperties {
  return {
    "--event-color": style.color,
  } as CSSProperties;
}

function createDraft(date: string, event?: CalendarEvent): CalendarEvent {
  if (event) return structuredClone(event);
  const now = new Date().toISOString();
  return {
    id: crypto.randomUUID(),
    title: "",
    date,
    allDay: false,
    startTime: "09:00",
    endTime: "10:00",
    location: "",
    notes: "",
    style: structuredClone(DEFAULT_EVENT_STYLE),
    createdAt: now,
    updatedAt: now,
  };
}

function App() {
  const today = useMemo(() => new Date(), []);
  const todayKey = toDateKey(today);
  const [data, setData] = useState<AppData | null>(null);
  const [displayMonth, setDisplayMonth] = useState(() => new Date(today.getFullYear(), today.getMonth(), 1));
  const [selectedDate, setSelectedDate] = useState(todayKey);
  const [editing, setEditing] = useState<CalendarEvent | "new" | null>(null);
  const [copiedContent, setCopiedContent] = useState<EventContent | null>(null);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [toast, setToast] = useState("");
  const [loadError, setLoadError] = useState("");

  useEffect(() => {
    loadAppData().then(setData).catch((error: unknown) => setLoadError(String(error)));
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

  useEffect(() => {
    if (!toast) return;
    const timer = window.setTimeout(() => setToast(""), 2600);
    return () => window.clearTimeout(timer);
  }, [toast]);

  const persist = useCallback(async (next: AppData, message?: string) => {
    try {
      await saveAppData(next);
      setData(next);
      if (message) setToast(message);
    } catch (error) {
      setToast(`保存できませんでした: ${String(error)}`);
    }
  }, []);

  const monthCells = useMemo(() => getMonthCells(displayMonth, today), [displayMonth, today]);
  const holidayMap = useMemo(
    () => getHolidayMap(monthCells[0].date, monthCells[monthCells.length - 1].date),
    [monthCells],
  );
  const todayEvents = useMemo(() => (data ? eventsForDate(data.events, todayKey) : []), [data, todayKey]);
  const upcoming = useMemo(() => (data ? upcomingEvents(data.events, today, 7).slice(0, 7) : []), [data, today]);

  const openNewEvent = (date = selectedDate) => {
    setSelectedDate(date);
    setEditing("new");
  };

  const saveEvent = async (event: CalendarEvent) => {
    if (!data) return;
    const exists = data.events.some((item) => item.id === event.id);
    const nextEvent = { ...event, updatedAt: new Date().toISOString() };
    const next = {
      ...data,
      events: exists ? data.events.map((item) => (item.id === event.id ? nextEvent : item)) : [...data.events, nextEvent],
    };
    await persist(next, exists ? "予定を更新しました" : "予定を追加しました");
    setSelectedDate(event.date);
    setDisplayMonth(new Date(Number(event.date.slice(0, 4)), Number(event.date.slice(5, 7)) - 1, 1));
    setEditing(null);
  };

  const deleteEvent = async (event: CalendarEvent) => {
    if (!data || !window.confirm(`「${event.title}」を削除しますか？\n削除した予定はデータ内に保管されます。`)) return;
    const next = {
      ...data,
      events: data.events.filter((item) => item.id !== event.id),
      deletedEvents: [...data.deletedEvents, { ...event, deletedAt: new Date().toISOString() }],
    };
    await persist(next, "予定を削除しました");
    setEditing(null);
  };

  const copyEvent = (event: CalendarEvent) => {
    setCopiedContent(copyEventContent(event));
    setEditing(null);
    setToast(`「${event.title.trim()}」の内容をコピーしました`);
  };

  const updateSettings = async (settings: AppData["settings"]) => {
    if (!data) return;
    await persist({ ...data, settings }, "表示設定を保存しました");
  };

  if (loadError) {
    return (
      <main className="fatal-state">
        <p className="eyebrow">Y-TEC Calendar</p>
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
          <div className="brand-mark" aria-hidden="true"><span>Y</span></div>
          <div>
            <p className="eyebrow">Y-TEC</p>
            <h1>Calendar</h1>
          </div>
        </div>

        <nav className="month-nav" aria-label="月の移動">
          <button className="icon-button" onClick={() => setDisplayMonth(shiftMonth(displayMonth, -1))} aria-label="前の月">‹</button>
          <button className="today-button" onClick={() => { setDisplayMonth(new Date(today.getFullYear(), today.getMonth(), 1)); setSelectedDate(todayKey); }}>今日</button>
          <button className="icon-button" onClick={() => setDisplayMonth(shiftMonth(displayMonth, 1))} aria-label="次の月">›</button>
          <h2>{monthTitle(displayMonth)}</h2>
        </nav>

        <div className="top-actions">
          <button
            className="secondary-button sidebar-toggle-button"
            onClick={() => void updateSettings({ ...data.settings, sidebarCollapsed: !data.settings.sidebarCollapsed })}
            aria-label={data.settings.sidebarCollapsed ? "サイドバーを開く" : "サイドバーを折りたたむ"}
            title={data.settings.sidebarCollapsed ? "サイドバーを開く" : "サイドバーを折りたたむ"}
          >
            <span aria-hidden="true">{data.settings.sidebarCollapsed ? "▥" : "▤"}</span>
          </button>
          <button className="secondary-button settings-button" onClick={() => setSettingsOpen(true)} aria-label="表示と起動の設定">
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
                  <button key={event.id} className="upcoming-item" onClick={() => { setSelectedDate(event.date); setEditing(event); }}>
                    <span className="upcoming-dot" style={{ background: event.style.color }} />
                    <span className="upcoming-date">{Number(event.date.slice(5, 7))}/{Number(event.date.slice(8, 10))}</span>
                    <span className="upcoming-info"><strong>{event.title}</strong><small>{formatEventTime(event)}</small></span>
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
          <div className="weekday-row">
            {WEEKDAYS.map((day, index) => <div key={day} className={index === 0 ? "sunday" : index === 6 ? "saturday" : ""}>{day}</div>)}
          </div>
          <div className="month-grid">
            {monthCells.map((cell) => {
              const events = eventsForDate(data.events, cell.dateKey);
              const selected = selectedDate === cell.dateKey;
              const holidayName = holidayMap.get(cell.dateKey);
              const dayOfWeek = cell.date.getDay();
              const dayType = holidayName || dayOfWeek === 0 ? " holiday" : dayOfWeek === 6 ? " saturday-day" : "";
              return (
                <div
                  key={cell.dateKey}
                  className={`day-cell${cell.isCurrentMonth ? "" : " outside"}${cell.isToday ? " today" : ""}${selected ? " selected" : ""}${dayType}`}
                  onClick={() => setSelectedDate(cell.dateKey)}
                >
                  <div className="day-cell-head">
                    <button
                      className="date-number"
                      onClick={() => setSelectedDate(cell.dateKey)}
                      aria-label={`${cell.date.getMonth() + 1}月${cell.date.getDate()}日を選択`}
                    >
                      {cell.date.getDate()}
                    </button>
                    {holidayName && <span className="holiday-name" title={holidayName}>{holidayName}</span>}
                    <button className="cell-add" onClick={(event) => { event.stopPropagation(); openNewEvent(cell.dateKey); }} aria-label={`${cell.date.getMonth() + 1}月${cell.date.getDate()}日に予定を追加`}>＋</button>
                  </div>
                  <div className="event-stack">
                    {events.slice(0, 3).map((event) => (
                      <button
                        key={event.id}
                        className="event-chip"
                        style={styleForEvent(event.style)}
                        onClick={(clickEvent) => { clickEvent.stopPropagation(); setEditing(event); setSelectedDate(event.date); }}
                        title={`${formatEventTime(event)} ${event.title}`}
                      >
                        {!event.allDay && <span className="event-time">{event.startTime}</span>}
                        <span className="event-title">{event.title}</span>
                      </button>
                    ))}
                    {events.length > 3 && <button className="more-events" onClick={() => setSelectedDate(cell.dateKey)}>ほか{events.length - 3}件</button>}
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
          onClose={() => setSettingsOpen(false)}
        />
      )}
      {toast && <div className="toast" role="status">{toast}</div>}
    </div>
  );
}

interface EventEditorProps {
  date: string;
  event?: CalendarEvent;
  copiedContent: EventContent | null;
  onClose: () => void;
  onSave: (event: CalendarEvent) => Promise<void>;
  onDelete: (event: CalendarEvent) => Promise<void>;
  onCopy: (event: CalendarEvent) => void;
}

function EventEditor({ date, event, copiedContent, onClose, onSave, onDelete, onCopy }: EventEditorProps) {
  const [draft, setDraft] = useState(() => createDraft(date, event));
  const [error, setError] = useState("");

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

  const submit = (submitEvent: FormEvent) => {
    submitEvent.preventDefault();
    const title = draft.title.trim();
    if (!title) return setError("予定名を入力してください。");
    if (!draft.date) return setError("日付を選択してください。");
    if (!draft.allDay && !isValidTimeRange(draft.startTime, draft.endTime)) return setError("終了時刻は開始時刻より後にしてください。");
    setError("");
    void onSave({ ...draft, title, location: draft.location.trim(), notes: draft.notes.trim() });
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

  return (
    <div className="modal-backdrop" onMouseDown={(mouseEvent) => { if (mouseEvent.target === mouseEvent.currentTarget) onClose(); }}>
      <form className="dialog event-dialog" onSubmit={submit} aria-modal="true" role="dialog" aria-labelledby="event-dialog-title">
        <header className="dialog-header">
          <div><p className="section-kicker">SCHEDULE</p><h2 id="event-dialog-title">{event ? "予定を編集" : "予定を追加"}</h2></div>
          <button type="button" className="close-button" onClick={onClose} aria-label="閉じる">×</button>
        </header>

        <div className="form-body">
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
            <label className="field"><span>日付</span><input type="date" value={draft.date} onChange={(changeEvent) => patchDraft("date", changeEvent.target.value)} /></label>
            <label className="toggle-field"><input type="checkbox" checked={draft.allDay} onChange={(changeEvent) => patchDraft("allDay", changeEvent.target.checked)} /><span className="toggle-track" /><span>終日</span></label>
            {!draft.allDay && (
              <div className="time-fields">
                <label className="field"><span>開始</span><input type="time" value={draft.startTime} onChange={(changeEvent) => patchDraft("startTime", changeEvent.target.value)} /></label>
                <span className="time-separator">→</span>
                <label className="field"><span>終了</span><input type="time" value={draft.endTime} onChange={(changeEvent) => patchDraft("endTime", changeEvent.target.value)} /></label>
              </div>
            )}
          </div>

          <label className="field"><span>場所 <small>任意</small></span><input value={draft.location} onChange={(changeEvent) => patchDraft("location", changeEvent.target.value)} placeholder="会議室、訪問先など" maxLength={100} /></label>
          <label className="field"><span>メモ <small>任意</small></span><textarea value={draft.notes} onChange={(changeEvent) => patchDraft("notes", changeEvent.target.value)} placeholder="補足や持ち物など" rows={3} maxLength={1000} /></label>

          <fieldset className="decoration-panel color-panel">
            <legend>予定の色</legend>
            <div className="color-row">
              <span>背景色</span><div className="color-options">{EVENT_COLORS.map((color) => <button type="button" key={color} className={draft.style.color === color ? "color-dot active" : "color-dot"} style={{ background: color }} onClick={() => patchStyle("color", color)} aria-label={`背景色 ${color}`} />)}</div>
            </div>
            <div className="event-preview" style={styleForEvent(draft.style)}><span>{draft.allDay ? "終日" : draft.startTime}</span>{draft.title || "予定のプレビュー"}</div>
          </fieldset>
          {error && <p className="form-error" role="alert">{error}</p>}
        </div>

        <footer className="dialog-footer">
          <div className="footer-left">
            {event && <button type="button" className="danger-button" onClick={() => void onDelete(event)}>削除</button>}
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
  onClose: () => void;
}

function SettingsDialog({ settings, onChange, onClose }: SettingsDialogProps) {
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
            <div className="settings-heading"><div><h3>背景テーマ</h3><p>5つの落ち着いた配色から選べます。</p></div></div>
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
            <div className="settings-row">
              <div><h3>Windows起動時に自動起動</h3><p>最後に閉じた位置とサイズでカレンダーを開きます。</p></div>
              <button className={`switch ${autoStart ? "on" : ""}`} onClick={() => void toggleAutoStart()} disabled={autoStartBusy || autoStart === null} role="switch" aria-checked={Boolean(autoStart)}><span /></button>
            </div>
            {autoStart === null && !autoStartBusy && <p className="native-note">Webプレビューでは変更できません。Windowsアプリ版で設定してください。</p>}
          </section>

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
