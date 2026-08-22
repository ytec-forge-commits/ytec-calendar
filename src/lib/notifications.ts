import gentleChimesUrl from "../assets/sounds/gentle-chimes.ogg";
import deepDropUrl from "../assets/sounds/deep-drop.ogg";
import smallBellUrl from "../assets/sounds/small-bell.ogg";
import gentlePianoUrl from "../assets/sounds/gentle-piano.ogg";
import quietKalimbaUrl from "../assets/sounds/quiet-kalimba.ogg";
import { addDays, eventsForDate, toDateKey } from "./calendar";
import { loadCustomNotificationSound } from "./store";
import type { CalendarEvent, NotificationSettings, NotificationSoundId } from "../types";

export const MAX_REMINDER_MINUTES = 40_320;
export const MAX_REMINDERS = 5;
export const REMINDER_GRACE_MS = 2 * 60_000;
export const NOTIFICATION_SOUND_AUTO_STOP_MS = 12_000;
export type ReminderUnit = "minutes" | "hours" | "days";
const REMINDER_UNIT_FACTORS: Record<ReminderUnit, number> = {
  minutes: 1,
  hours: 60,
  days: 1_440,
};
const MAX_MIDI_TRACKS = 64;
const MAX_MIDI_EVENTS = 100_000;
const MAX_MIDI_NOTES = 2_048;

export interface NotificationSoundOption {
  id: NotificationSoundId;
  name: string;
  description: string;
  sourceUrl?: string;
}

export const BUILT_IN_NOTIFICATION_SOUNDS: NotificationSoundOption[] = [
  { id: "gentle-chimes", name: "やわらぎ", description: "澄んだチャイム", sourceUrl: gentleChimesUrl },
  { id: "deep-drop", name: "深い雫", description: "静かに響く低い音", sourceUrl: deepDropUrl },
  { id: "small-bell", name: "小鈴", description: "控えめな鈴の音", sourceUrl: smallBellUrl },
  { id: "gentle-piano", name: "朝露のピアノ", description: "やわらかな短いピアノ", sourceUrl: gentlePianoUrl },
  { id: "quiet-kalimba", name: "木漏れ日のカリンバ", description: "穏やかな木の音色", sourceUrl: quietKalimbaUrl },
];

export interface DueNotification {
  key: string;
  event: CalendarEvent;
  startsAt: Date;
  reminderMinutes: number;
}

function eventStart(event: CalendarEvent): Date | null {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(event.date);
  if (!match) return null;
  let hours = 0;
  let minutes = 0;
  if (!event.allDay) {
    const time = /^(\d{2}):(\d{2})$/.exec(event.startTime);
    if (!time) return null;
    hours = Number(time[1]);
    minutes = Number(time[2]);
  }
  const result = new Date(Number(match[1]), Number(match[2]) - 1, Number(match[3]), hours, minutes);
  return Number.isNaN(result.getTime()) ? null : result;
}

export function collectDueNotifications(
  events: CalendarEvent[],
  now = new Date(),
  graceMs = REMINDER_GRACE_MS,
): DueNotification[] {
  const result: DueNotification[] = [];
  const seenOccurrences = new Set<string>();
  const todayKey = toDateKey(now);
  for (let offset = 0; offset <= 28; offset += 1) {
    for (const event of eventsForDate(events, addDays(todayKey, offset))) {
      const occurrenceDate = event.occurrence?.originalDate ?? event.recurrenceException?.originalDate ?? event.date;
      const occurrenceKey = `${event.id}:${occurrenceDate}`;
      if (seenOccurrences.has(occurrenceKey)) continue;
      seenOccurrences.add(occurrenceKey);
      const startsAt = eventStart(event);
      if (!startsAt) continue;
      for (const reminderMinutes of event.reminders.popupMinutes) {
        const dueAt = startsAt.getTime() - reminderMinutes * 60_000;
        if (dueAt <= now.getTime() && dueAt > now.getTime() - graceMs) {
          result.push({
            key: `${occurrenceKey}:${reminderMinutes}`,
            event,
            startsAt,
            reminderMinutes,
          });
        }
      }
    }
  }
  return result.sort((left, right) => left.startsAt.getTime() - right.startsAt.getTime());
}

export function reminderLabel(minutes: number): string {
  if (minutes === 0) return "開始時刻";
  if (minutes % 10_080 === 0) return `${minutes / 10_080}週間前`;
  if (minutes % 1_440 === 0) return `${minutes / 1_440}日前`;
  if (minutes % 60 === 0) return `${minutes / 60}時間前`;
  return `${minutes}分前`;
}

export function reminderInputParts(minutes: number): { amount: number; unit: ReminderUnit } {
  const normalized = Math.max(0, Math.min(MAX_REMINDER_MINUTES, Math.round(minutes) || 0));
  if (normalized > 0 && normalized % REMINDER_UNIT_FACTORS.days === 0) {
    return { amount: normalized / REMINDER_UNIT_FACTORS.days, unit: "days" };
  }
  if (normalized > 0 && normalized % REMINDER_UNIT_FACTORS.hours === 0) {
    return { amount: normalized / REMINDER_UNIT_FACTORS.hours, unit: "hours" };
  }
  return { amount: normalized, unit: "minutes" };
}

export function reminderMinutesFromInput(amount: number, unit: ReminderUnit): number {
  const factor = REMINDER_UNIT_FACTORS[unit];
  return Math.max(0, Math.min(MAX_REMINDER_MINUTES, Math.round(amount || 0) * factor));
}

export function maxReminderInputAmount(unit: ReminderUnit): number {
  return Math.floor(MAX_REMINDER_MINUTES / REMINDER_UNIT_FACTORS[unit]);
}

export interface NotificationPlayback {
  stop: () => void;
}

export function scheduleNotificationPlaybackStop(
  playback: NotificationPlayback,
  onStopped: () => void,
  delayMs = NOTIFICATION_SOUND_AUTO_STOP_MS,
): () => void {
  let active = true;
  const timer = globalThis.setTimeout(() => {
    if (!active) return;
    active = false;
    playback.stop();
    onStopped();
  }, delayMs);
  return () => {
    if (!active) return;
    active = false;
    globalThis.clearTimeout(timer);
  };
}

interface MidiNote {
  startSeconds: number;
  durationSeconds: number;
  note: number;
  velocity: number;
}

interface ParsedMidi {
  notes: MidiNote[];
  durationSeconds: number;
}

function readVariableLength(bytes: Uint8Array, cursor: { value: number }): number {
  let result = 0;
  for (let count = 0; count < 4; count += 1) {
    if (cursor.value >= bytes.length) throw new Error("MIDIデータが途中で終了しています");
    const value = bytes[cursor.value++];
    result = (result << 7) | (value & 0x7f);
    if ((value & 0x80) === 0) return result;
  }
  throw new Error("MIDIの可変長データが正しくありません");
}

function readUint32(bytes: Uint8Array, offset: number): number {
  return ((bytes[offset] << 24) | (bytes[offset + 1] << 16) | (bytes[offset + 2] << 8) | bytes[offset + 3]) >>> 0;
}

export function parseMidi(bytes: Uint8Array): ParsedMidi {
  if (bytes.length < 14 || new TextDecoder().decode(bytes.slice(0, 4)) !== "MThd") {
    throw new Error("MIDIファイルのヘッダーを確認できません");
  }
  const headerLength = readUint32(bytes, 4);
  if (headerLength < 6 || bytes.length < 8 + headerLength) throw new Error("MIDIヘッダーが正しくありません");
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const trackCount = view.getUint16(10);
  const division = view.getUint16(12);
  if (trackCount === 0 || trackCount > MAX_MIDI_TRACKS) throw new Error("MIDIのトラック数が対応範囲を超えています");
  if ((division & 0x8000) !== 0 || division === 0) throw new Error("SMPTE時間形式のMIDIには対応していません");

  const rawNotes: Array<{ startTick: number; endTick: number; note: number; velocity: number }> = [];
  const pushNote = (note: { startTick: number; endTick: number; note: number; velocity: number }) => {
    if (rawNotes.length >= MAX_MIDI_NOTES) throw new Error("MIDIの音符数が通知音の対応範囲を超えています");
    rawNotes.push(note);
  };
  const tempos: Array<{ tick: number; microseconds: number }> = [{ tick: 0, microseconds: 500_000 }];
  let offset = 8 + headerLength;
  let latestTick = 0;
  let eventCount = 0;
  for (let trackIndex = 0; trackIndex < trackCount; trackIndex += 1) {
    if (offset + 8 > bytes.length || new TextDecoder().decode(bytes.slice(offset, offset + 4)) !== "MTrk") {
      throw new Error("MIDIトラックを確認できません");
    }
    const trackEnd = offset + 8 + readUint32(bytes, offset + 4);
    if (trackEnd > bytes.length) throw new Error("MIDIトラックが途中で終了しています");
    const cursor = { value: offset + 8 };
    const active = new Map<string, Array<{ tick: number; velocity: number }>>();
    let tick = 0;
    let runningStatus = 0;
    while (cursor.value < trackEnd) {
      eventCount += 1;
      if (eventCount > MAX_MIDI_EVENTS) throw new Error("MIDIのイベント数が通知音の対応範囲を超えています");
      tick += readVariableLength(bytes, cursor);
      latestTick = Math.max(latestTick, tick);
      let status = bytes[cursor.value++];
      let firstData: number | undefined;
      if (status < 0x80) {
        if (!runningStatus) throw new Error("MIDIのランニングステータスが正しくありません");
        firstData = status;
        status = runningStatus;
      } else if (status < 0xf0) {
        runningStatus = status;
      }
      if (status === 0xff) {
        if (cursor.value >= trackEnd) throw new Error("MIDIメタイベントが途中で終了しています");
        const type = bytes[cursor.value++];
        const length = readVariableLength(bytes, cursor);
        if (cursor.value + length > trackEnd) throw new Error("MIDIメタイベントが途中で終了しています");
        if (type === 0x51 && length === 3) {
          tempos.push({
            tick,
            microseconds: (bytes[cursor.value] << 16) | (bytes[cursor.value + 1] << 8) | bytes[cursor.value + 2],
          });
        }
        cursor.value += length;
        continue;
      }
      if (status === 0xf0 || status === 0xf7) {
        const length = readVariableLength(bytes, cursor);
        cursor.value += length;
        if (cursor.value > trackEnd) throw new Error("MIDI SysExが途中で終了しています");
        continue;
      }
      const command = status & 0xf0;
      const channel = status & 0x0f;
      const data1 = firstData ?? bytes[cursor.value++];
      const hasSecond = command !== 0xc0 && command !== 0xd0;
      const data2 = hasSecond ? bytes[cursor.value++] : 0;
      if (cursor.value > trackEnd) throw new Error("MIDIイベントが途中で終了しています");
      if (command !== 0x80 && command !== 0x90) continue;
      const key = `${channel}:${data1}`;
      if (command === 0x90 && data2 > 0) {
        const notes = active.get(key) ?? [];
        notes.push({ tick, velocity: data2 });
        active.set(key, notes);
      } else {
        const notes = active.get(key);
        const started = notes?.shift();
        if (started) pushNote({ startTick: started.tick, endTick: Math.max(tick, started.tick + 1), note: data1, velocity: started.velocity });
      }
    }
    for (const [key, notes] of active) {
      const note = Number(key.split(":")[1]);
      for (const started of notes) pushNote({ startTick: started.tick, endTick: Math.max(latestTick, started.tick + division / 4), note, velocity: started.velocity });
    }
    offset = trackEnd;
  }
  if (!rawNotes.length) throw new Error("再生できる音符がMIDIに含まれていません");

  const tempoMap = [...tempos]
    .sort((left, right) => left.tick - right.tick)
    .filter((tempo, index, list) => index === list.length - 1 || tempo.tick !== list[index + 1].tick);
  const secondsAtTick = (targetTick: number) => {
    let seconds = 0;
    let previousTick = 0;
    let microseconds = 500_000;
    for (const tempo of tempoMap) {
      if (tempo.tick > targetTick) break;
      seconds += ((tempo.tick - previousTick) * microseconds) / division / 1_000_000;
      previousTick = tempo.tick;
      microseconds = tempo.microseconds;
    }
    return seconds + ((targetTick - previousTick) * microseconds) / division / 1_000_000;
  };
  const notes = rawNotes.map((note) => {
    const startSeconds = secondsAtTick(note.startTick);
    const endSeconds = secondsAtTick(note.endTick);
    return { startSeconds, durationSeconds: Math.max(0.04, endSeconds - startSeconds), note: note.note, velocity: note.velocity };
  });
  const durationSeconds = Math.max(0.5, ...notes.map((note) => note.startSeconds + note.durationSeconds)) + 0.25;
  return { notes, durationSeconds };
}

function playMidi(bytes: Uint8Array, volume: number): NotificationPlayback {
  const parsed = parseMidi(bytes);
  const context = new AudioContext();
  const master = context.createGain();
  master.gain.value = Math.max(0, Math.min(1, volume / 100)) * 0.42;
  master.connect(context.destination);
  let stopped = false;
  let timer = 0;
  const oscillators = new Set<OscillatorNode>();
  const schedule = () => {
    if (stopped) return;
    const base = context.currentTime + 0.04;
    for (const note of parsed.notes) {
      const oscillator = context.createOscillator();
      const gain = context.createGain();
      const start = base + note.startSeconds;
      const end = start + Math.min(note.durationSeconds, 4);
      oscillator.type = "sine";
      oscillator.frequency.value = 440 * 2 ** ((note.note - 69) / 12);
      gain.gain.setValueAtTime(0.0001, start);
      gain.gain.exponentialRampToValueAtTime(Math.max(0.002, note.velocity / 127), start + 0.018);
      gain.gain.exponentialRampToValueAtTime(0.0001, end + 0.18);
      oscillator.connect(gain).connect(master);
      oscillator.start(start);
      oscillator.stop(end + 0.2);
      oscillators.add(oscillator);
      oscillator.addEventListener("ended", () => oscillators.delete(oscillator));
    }
    timer = window.setTimeout(schedule, Math.max(600, parsed.durationSeconds * 1000));
  };
  schedule();
  return {
    stop: () => {
      stopped = true;
      window.clearTimeout(timer);
      oscillators.forEach((oscillator) => {
        try { oscillator.stop(); } catch { /* already stopped */ }
      });
      oscillators.clear();
      void context.close();
    },
  };
}

export async function playNotificationSound(settings: NotificationSettings): Promise<NotificationPlayback> {
  if (settings.soundId === "silent" || settings.volume <= 0) return { stop: () => undefined };
  if (settings.soundId === "custom") {
    const custom = settings.customSound;
    if (!custom) throw new Error("カスタム通知音が設定されていません");
    const bytes = await loadCustomNotificationSound(custom.storedFileName);
    if (custom.kind === "midi") return playMidi(bytes, settings.volume);
    const url = URL.createObjectURL(new Blob([Uint8Array.from(bytes).buffer], { type: custom.mimeType }));
    const audio = new Audio(url);
    audio.loop = true;
    audio.volume = Math.max(0, Math.min(1, settings.volume / 100));
    try {
      await audio.play();
    } catch (error) {
      URL.revokeObjectURL(url);
      throw new Error(`この音声を再生できません: ${String(error)}`, { cause: error });
    }
    return { stop: () => { audio.pause(); audio.currentTime = 0; URL.revokeObjectURL(url); } };
  }
  const option = BUILT_IN_NOTIFICATION_SOUNDS.find((sound) => sound.id === settings.soundId);
  if (!option?.sourceUrl) throw new Error("選択した標準通知音を確認できません");
  const audio = new Audio(option.sourceUrl);
  audio.loop = true;
  audio.volume = Math.max(0, Math.min(1, settings.volume / 100));
  await audio.play();
  return { stop: () => { audio.pause(); audio.currentTime = 0; } };
}
