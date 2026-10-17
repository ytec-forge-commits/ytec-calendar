import { useEffect, useRef, useState } from "react";
import { applyBackupRestore, chooseBackupFile, chooseStorageDirectory, configureAutoBackup, exportBackup, getAutoBackup, isTauriRuntime, previewBackupRestore, runDueBackup, type AutoBackupSettings, type BackupRestorePreview } from "./lib/store";
import type { AppData } from "./types";

interface Props { onChanged: (data: AppData) => void; onBusyChange: (busy: boolean) => void }
type FeedbackArea = "manual" | "restore" | "automatic";
export function BackupSettings({ onChanged, onBusyChange }: Props) {
  const [destination, setDestination] = useState("");
  const [archive, setArchive] = useState("");
  const [target, setTarget] = useState("");
  const [preview, setPreview] = useState<BackupRestorePreview | null>(null);
  const [auto, setAuto] = useState<AutoBackupSettings | null>(null);
  const [busy, setBusy] = useState(false);
  const [status, setStatus] = useState("");
  const [error, setError] = useState("");
  const [feedbackArea, setFeedbackArea] = useState<FeedbackArea>("manual");
  const feedbackRef = useRef<HTMLDivElement>(null);
  useEffect(() => { getAutoBackup().then(setAuto).catch(e => { setFeedbackArea("automatic"); setError(String(e)); }); }, []);
  useEffect(() => {
    if (busy || status || error) feedbackRef.current?.scrollIntoView({ block: "nearest" });
  }, [busy, status, error, feedbackArea]);
  const run = async (action: () => Promise<void>, area: FeedbackArea = "manual") => {
    setFeedbackArea(area); setBusy(true); onBusyChange(true); setError(""); setStatus("");
    try { await action(); } catch (e) { setPreview(null); setError(`${String(e)} 元の予定データは削除していません。${area === "restore" ? "途中の復元フォルダーが残った場合は別の空フォルダーでやり直してください。" : ""}`); }
    finally { setBusy(false); onBusyChange(false); }
  };
  const native = isTauriRuntime();
  const feedback = (area: FeedbackArea) => feedbackArea === area && <div ref={feedbackRef} className="backup-feedback">
    {busy && <p role="status">操作を処理中です。完了までお待ちください。</p>}
    {status && <p role="status" className="backup-status">{status}</p>}
    {error && <p role="alert" className="form-error">{error}</p>}
  </div>;
  return <section className="settings-section data-location backup-settings">
    <h3>ZIPバックアップ・復元</h3>
    <p>予定・削除済み予定・外観設定・通知音を1つのZIPに保存します。再インストールや別PCへの引越しに利用できます。</p>
    <p className="native-note">ZIPは暗号化されません。予定・アカウント情報が含まれるため、ご自身で管理してください。OAuthクライアント・ログイン用トークン・PC固有の保存先指定・自動バックアップ設定・過去の履歴ファイルは含みません。</p>
    {!native ? <p>ZIP操作はWindowsアプリ版で利用できます。</p> : <fieldset disabled={busy}>
      <legend>手動バックアップ</legend>
      <label className="field"><span>保存するZIPのパス</span><input value={destination} placeholder="例：C:\Backups\koyomado-2026-10-17.zip" onChange={e => setDestination(e.target.value)} /></label>
      <div className="storage-actions">
        <button className="secondary-button" type="button" onClick={() => void run(async () => { const path = await chooseBackupFile(true); if (path) setDestination(path); })}>ZIPの保存先を選択</button>
        <button className="primary-button" type="button" disabled={!destination.trim()} onClick={() => void run(async () => { const path = await exportBackup(destination.trim()); setStatus(`ZIPを保存・照合しました：${path}`); })}>ZIPを保存する</button>
      </div>
      <small>アプリのdataフォルダー外へ保存してください。同名ZIPには上書きしません。</small>
      {feedback("manual")}
      <h4>ZIPから復元</h4>
      <label className="field"><span>復元するZIPのパス</span><input value={archive} onChange={e => { setArchive(e.target.value); setPreview(null); }} /></label>
      <button className="secondary-button" type="button" onClick={() => void run(async () => { const path = await chooseBackupFile(false); if (path) { setArchive(path); setPreview(null); } }, "restore")}>復元するZIPを選択</button>
      <label className="field"><span>復元先の空フォルダー</span><input value={target} onChange={e => { setTarget(e.target.value); setPreview(null); }} /></label>
      <div className="storage-actions">
        <button className="secondary-button" type="button" onClick={() => void run(async () => { const path = await chooseStorageDirectory(); if (path) { setTarget(path); setPreview(null); } }, "restore")}>復元先フォルダーを選択</button>
        <button className="secondary-button" type="button" disabled={!archive.trim() || !target.trim()} onClick={() => void run(async () => setPreview(await previewBackupRestore(archive.trim(), target.trim())), "restore")}>ZIPの内容を確認</button>
      </div>
      {preview && <div className="storage-preview">
        <h4>復元内容の確認</h4>
        <p>作成日時：{new Date(preview.createdAt).toLocaleString("ja-JP")}<br />予定 {preview.eventCount}件 ／ 削除済み {preview.deletedEventCount}件 ／ 通知音 {preview.soundCount}件 ／ Googleアカウント {preview.googleAccountCount}件</p>
        <code>{preview.targetDirectory}</code>
        <p>空フォルダーへ復元・照合してから表示を切り替えます。現在のデータとZIPは残し、予定を統合しません。</p>
        <p>Google連携は一旦OFFになります。OAuth設定の読込・再認証後、最初の同期は受信と照合だけを行い、Google側へ追加・更新・削除しません。紐付く予定はGoogle側の最新を優先し、照合できない復元予定はローカルに残します。元の内容はZIPから確認できます。</p>
        <div className="storage-actions"><button className="primary-button" type="button" onClick={() => void run(async () => onChanged(await applyBackupRestore(preview.token)), "restore")}>確認したZIPを復元して切り替える</button><button className="secondary-button" type="button" onClick={() => setPreview(null)}>復元をキャンセル</button></div>
      </div>}
      {feedback("restore")}
      <h4>自動バックアップ</h4>
      {auto && <>
        <label><input type="checkbox" checked={auto.enabled} onChange={e => setAuto({ ...auto, enabled: e.target.checked })} /> 自動バックアップを有効にする</label>
        <label className="field"><span>自動バックアップ先フォルダー</span><input value={auto.directory} onChange={e => setAuto({ ...auto, directory: e.target.value })} /></label>
        <button className="secondary-button" type="button" onClick={() => void run(async () => { const path = await chooseStorageDirectory(); if (path) setAuto({ ...auto, directory: path }); }, "automatic")}>自動バックアップ先を選択</button>
        <div className="storage-actions">
          <label className="field"><span>作成間隔</span><select value={auto.frequency} onChange={e => setAuto({ ...auto, frequency: e.target.value as AutoBackupSettings["frequency"] })}><option value="daily">毎日</option><option value="weekly">毎週（月曜区切り）</option><option value="monthly">毎月（1日区切り）</option></select></label>
          <label className="field"><span>残す世代数（1～100）</span><input type="number" min="1" max="100" value={auto.generations} onChange={e => setAuto({ ...auto, generations: Number(e.target.value) })} /></label>
        </div>
        <button className="primary-button" type="button" onClick={() => void run(async () => { await configureAutoBackup(auto); await runDueBackup(); setAuto(await getAutoBackup()); setStatus("自動バックアップ設定を保存しました。対象期間のバックアップを確認しました。"); }, "automatic")}>自動バックアップ設定を保存</button>
        {auto.lastSuccess && <p>最終成功：{new Date(auto.lastSuccess).toLocaleString("ja-JP")}</p>}
        {auto.lastError && <p className="form-error">{auto.lastError}</p>}
      </>}
      <p className="native-note">起動中に各期間1回、終了中の分は次回起動時に作成します。新しいZIPの保存・照合成功後に、この設定で作成した古い世代だけを整理します。手動ZIP・別の設定のZIPには触れません。予定データとは別のフォルダーを指定してください。</p>
      {feedback("automatic")}
    </fieldset>}
  </section>;
}
