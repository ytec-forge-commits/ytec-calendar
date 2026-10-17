import { useEffect, useState } from "react";
import { applyStorageDirectory, chooseStorageDirectory, getStorageInfo, isTauriRuntime, previewStorageDirectory, type StorageInfo, type StoragePreview } from "./lib/store";
import type { AppData } from "./types";

interface Props {
  onChanged: (data: AppData) => void;
  onBusyChange: (busy: boolean) => void;
}

export function StorageSettings({ onChanged, onBusyChange }: Props) {
  const [info, setInfo] = useState<StorageInfo | null>(null);
  const [directory, setDirectory] = useState("");
  const [preview, setPreview] = useState<StoragePreview | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const native = isTauriRuntime();
  useEffect(() => { getStorageInfo().then(setInfo).catch((e: unknown) => setError(String(e))); }, []);

  const run = async (action: () => Promise<void>) => {
    setError(""); setBusy(true); onBusyChange(true);
    try { await action(); }
    catch (e) {
      setPreview(null);
      setError(`${String(e)} 元のデータは削除していません。現在の保存先を確認してください。コピー途中のファイルが残った場合は別の空フォルダーを選んでください。`);
      void getStorageInfo().then(setInfo).catch(() => undefined);
    }
    finally { setBusy(false); onBusyChange(false); }
  };
  const inspect = async (path: string) => {
    setPreview(null);
    const result = await previewStorageDirectory(path.trim());
    setDirectory(result.targetDirectory); setPreview(result);
  };
  const select = () => run(async () => {
    const selected = await chooseStorageDirectory();
    if (selected) { setDirectory(selected); await inspect(selected); }
  });
  const apply = (mode: "copyCurrent" | "useExisting") => run(async () => {
    if (!preview) return;
    const data = await applyStorageDirectory(preview.token, mode);
    onChanged(data);
  });

  return <section className="settings-section data-location">
    <h3>予定データの保存先</h3>
    <p>Google Driveなど、このPCから読み書きできるフォルダーを指定できます。</p>
    <small>現在の保存先</small><code>{info?.directory ?? "確認中…"}</code>
    {info && !info.available && <p className="form-error" role="alert">現在の保存先を利用できません。接続・同期状態を確認するか、既存データのある別の保存先を指定してください。</p>}
    {!native ? <p className="native-note">保存先の変更はWindowsアプリ版で利用できます。</p> : <>
      <label className="field"><span>保存先フォルダーのパス</span><input value={directory} disabled={busy} placeholder="例：G:\マイドライブ\Koyomado\data" onChange={e => { setDirectory(e.target.value); setPreview(null); }} /></label>
      <div className="storage-actions">
        <button type="button" className="secondary-button" disabled={busy} onClick={() => void select()}>フォルダーを選択</button>
        <button type="button" className="secondary-button" disabled={busy || !directory.trim()} onClick={() => void run(() => inspect(directory))}>このフォルダーを確認</button>
        {info && <button type="button" className="text-button" disabled={busy} onClick={() => { setDirectory(info.defaultDirectory); setPreview(null); }}>標準の保存先を指定</button>}
      </div>
      <p className="native-note">新規保存は空フォルダー、既存ポータブル版を参照する場合はcalendar-data.jsonのあるdataフォルダーを選択してください。</p>
      {preview && <div className="storage-preview">
        <h4>切替内容の確認</h4>
        <small>選択先</small><code>{preview.targetDirectory}</code>
        <p>現在の予定：{preview.sourceEventCount === null ? "利用できません" : `${preview.sourceEventCount}件`} ／ 選択先：{preview.targetEventCount === null ? "新規の空フォルダー" : `${preview.targetEventCount}件`}</p>
        {preview.targetEventCount === null ? <>
          <p>予定・バックアップ・履歴・通知音をコピーし、照合してから切り替えます。元のデータは残します。</p>
          <button type="button" className="primary-button" disabled={busy || preview.sourceEventCount === null} onClick={() => void apply("copyCurrent")}>現在のデータをコピーして切り替える</button>
        </> : <>
          <p>選択先にある予定へ表示を切り替えます。現在の予定とは統合せず、両方のデータを残します。</p>
          <button type="button" className="primary-button" disabled={busy} onClick={() => void apply("useExisting")}>選択先の既存データを使う</button>
        </>}
        <button type="button" className="secondary-button" disabled={busy} onClick={() => setPreview(null)}>切替をキャンセル</button>
      </div>}
    </>}
    {busy && <p role="status">保存先を確認・切替中です。コピーと照合が完了するまでお待ちください。</p>}
    {error && <p className="form-error" role="alert">{error}</p>}
    <p className="native-note">保存先の指定はこのPC、ウィンドウ位置は標準の保存先に残ります。Driveの同期完了を確認してください。設定・履歴にも予定情報が含まれます。</p>
  </section>;
}
