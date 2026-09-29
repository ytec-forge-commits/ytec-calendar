import { useEffect, useRef, useState, type ChangeEvent } from "react";
import {
  connectGoogleAccount,
  disconnectGoogleAccount,
  getGoogleCredentialStatuses,
  listGoogleCalendars,
  parseGoogleOAuthClientJson,
} from "./lib/store";
import type {
  GoogleAccount,
  GoogleCalendarOption,
  GoogleIntegrationSettings,
} from "./types";

interface GoogleSettingsProps {
  google: GoogleIntegrationSettings;
  onChange: (google: GoogleIntegrationSettings) => Promise<void>;
  onSync: () => Promise<boolean>;
  syncBusy: boolean;
}

export function GoogleSettings({ google, onChange, onSync, syncBusy }: GoogleSettingsProps) {
  const [busy, setBusy] = useState("");
  const [defaultTargetsBusy, setDefaultTargetsBusy] = useState(false);
  const defaultTargetsSaving = useRef(false);
  const [status, setStatus] = useState("");
  const [credentialAvailability, setCredentialAvailability] = useState<Record<string, boolean>>({});
  const [calendarOptions, setCalendarOptions] = useState<Record<string, GoogleCalendarOption[]>>({});
  const activeAccounts = google.accounts.filter((account) => account.syncEnabled);
  const selectedDefaultCount = activeAccounts.filter((account) => google.defaultSyncTargets.includes(account.id)).length;

  useEffect(() => {
    if (google.accounts.length === 0) return;
    void getGoogleCredentialStatuses(google.accounts.map((account) => account.id))
      .then((items) => setCredentialAvailability(Object.fromEntries(items.map((item) => [item.accountId, item.available]))))
      .catch((error: unknown) => setStatus(`認証状態を確認できませんでした: ${String(error)}`));
  }, [google.accounts]);

  const importClient = async (changeEvent: ChangeEvent<HTMLInputElement>) => {
    const file = changeEvent.target.files?.[0];
    changeEvent.target.value = "";
    if (!file) return;
    setBusy("client");
    try {
      const client = parseGoogleOAuthClientJson(await file.text());
      await onChange({ ...google, enabled: true, client });
      setStatus("OAuthクライアント設定を読み込みました。続けてGoogleアカウントを接続してください。");
    } catch (error) {
      setStatus(String(error));
    } finally {
      setBusy("");
    }
  };

  const connectAccount = async () => {
    if (!google.client) return setStatus("先にOAuthクライアントJSONを読み込んでください。");
    if (google.accounts.length >= 3) return setStatus("Googleアカウントは3件まで接続できます。");
    setBusy("connect");
    setStatus("ブラウザーでGoogleアカウントを選び、アクセスを許可してください…");
    try {
      const result = await connectGoogleAccount(google.client);
      const accounts = [...google.accounts.filter((account) => account.id !== result.account.id), result.account];
      await onChange({ ...google, enabled: true, accounts });
      setCalendarOptions((current) => ({ ...current, [result.account.id]: result.calendars }));
      setCredentialAvailability((current) => ({ ...current, [result.account.id]: true }));
      setStatus(`${result.account.email} を接続しました。`);
    } catch (error) {
      setStatus(`Googleアカウントを接続できませんでした: ${String(error)}`);
    } finally {
      setBusy("");
    }
  };

  const reauthenticate = async (account: GoogleAccount) => {
    if (!google.client) return setStatus("OAuthクライアント設定を読み込み直してください。");
    setBusy(`reauth-${account.id}`);
    setStatus("ブラウザーで同じGoogleアカウントを選んでください…");
    try {
      const result = await connectGoogleAccount(google.client);
      if (result.account.id !== account.id) {
        await disconnectGoogleAccount(result.account.id).catch(() => undefined);
        setStatus(`別のアカウント（${result.account.email}）が選択されました。${account.email} を選び直してください。`);
        return;
      }
      const restored = { ...result.account, calendarId: account.calendarId, calendarName: account.calendarName, syncToken: "" };
      await onChange({ ...google, enabled: true, accounts: google.accounts.map((item) => item.id === account.id ? restored : item) });
      setCalendarOptions((current) => ({ ...current, [account.id]: result.calendars }));
      setCredentialAvailability((current) => ({ ...current, [account.id]: true }));
      setStatus(`${account.email} を再認証しました。`);
    } catch (error) {
      setStatus(`再認証できませんでした: ${String(error)}`);
    } finally {
      setBusy("");
    }
  };

  const loadCalendars = async (account: GoogleAccount) => {
    if (!google.client || calendarOptions[account.id]) return;
    setBusy(`calendars-${account.id}`);
    try {
      const calendars = await listGoogleCalendars(google.client, account.id);
      setCalendarOptions((current) => ({ ...current, [account.id]: calendars }));
    } catch (error) {
      setStatus(`カレンダー一覧を取得できませんでした: ${String(error)}`);
    } finally {
      setBusy("");
    }
  };

  const patchAccount = async (accountId: string, patch: Partial<GoogleAccount>) => {
    const defaultSyncTargets = patch.syncEnabled === false
      ? google.defaultSyncTargets.filter((target) => target !== accountId)
      : google.defaultSyncTargets;
    await onChange({
      ...google,
      accounts: google.accounts.map((account) => account.id === accountId ? { ...account, ...patch } : account),
      defaultSyncTargets,
    });
  };

  const saveDefaultSyncTargets = async (defaultSyncTargets: string[]) => {
    if (defaultTargetsSaving.current) return;
    defaultTargetsSaving.current = true;
    setDefaultTargetsBusy(true);
    try {
      await onChange({ ...google, defaultSyncTargets });
    } catch (error) {
      setStatus(`既定の保存先を変更できませんでした: ${String(error)}`);
    } finally {
      defaultTargetsSaving.current = false;
      setDefaultTargetsBusy(false);
    }
  };

  const toggleDefaultSyncTarget = async (accountId: string, enabled: boolean) => {
    await saveDefaultSyncTargets(enabled
      ? [...new Set([...google.defaultSyncTargets, accountId])]
      : google.defaultSyncTargets.filter((target) => target !== accountId));
  };

  const disconnect = async (account: GoogleAccount) => {
    if (!window.confirm(`${account.email} との接続を解除しますか？\n取り込んだ予定はKoyomadoのローカル予定として残します。`)) return;
    setBusy(`disconnect-${account.id}`);
    try {
      const result = await disconnectGoogleAccount(account.id);
      await onChange({
        ...google,
        accounts: google.accounts.filter((item) => item.id !== account.id),
        defaultSyncTargets: google.defaultSyncTargets.filter((target) => target !== account.id),
      });
      setStatus(result.message);
    } catch (error) {
      setStatus(`接続を解除できませんでした: ${String(error)}`);
    } finally {
      setBusy("");
    }
  };

  return (
    <section className="settings-section google-settings">
      <div className="settings-row">
        <div>
          <h3>Googleカレンダー連携</h3>
          <p>任意機能です。接続した場合だけGoogleへ通信します。予定と認証情報をY-TECへ送信することはありません。</p>
        </div>
        <button className={`switch ${google.enabled ? "on" : ""}`} onClick={() => void onChange({ ...google, enabled: !google.enabled })} role="switch" aria-checked={google.enabled}><span /></button>
      </div>

      {google.enabled && (
        <div className="google-settings-content">
          <div className="oauth-client-card">
            <div>
              <strong>利用者自身のOAuthクライアント</strong>
              <small>{google.client ? `設定済み${google.client.projectId ? `（${google.client.projectId}）` : ""}` : "Google Cloudで作成したデスクトップアプリ用JSONが必要です"}</small>
            </div>
            <label className="secondary-button file-button">
              {google.client ? "JSONを読み直す" : "JSONを選択"}
              <input type="file" accept="application/json,.json" onChange={(event) => void importClient(event)} disabled={Boolean(busy)} />
            </label>
          </div>
          <p className="native-note oauth-publishing-note">常用する場合は、利用者自身のGoogle Auth Platformで公開ステータスを「In production」にしてください。個人利用ではOAuth審査やWebサイト登録は不要です。Testingのままでは認証が原則7日で切れます。</p>

          <div className="google-account-heading">
            <span><strong>接続アカウント</strong><small>{google.accounts.length}/3件</small></span>
            <span className="google-heading-actions">
              {google.accounts.some((account) => account.syncEnabled) && <button type="button" className="secondary-button" onClick={() => void onSync()} disabled={Boolean(busy) || syncBusy}>{syncBusy ? "同期中…" : "↻ 今すぐ同期"}</button>}
              <button type="button" className="secondary-button" onClick={() => void connectAccount()} disabled={!google.client || Boolean(busy) || syncBusy || google.accounts.length >= 3}>＋ アカウントを接続</button>
            </span>
          </div>

          {google.accounts.length === 0 ? <p className="google-empty">接続済みのGoogleアカウントはありません。</p> : (
            <div className="google-account-list">
              {google.accounts.map((account) => {
                const credentialAvailable = credentialAvailability[account.id] ?? !account.needsReauth;
                const calendars = calendarOptions[account.id];
                return (
                  <article key={account.id} className="google-account-card">
                    <div className="google-account-title">
                      <span><strong>{account.displayName || account.email}</strong><small>{account.email}</small></span>
                      <span className={credentialAvailable ? "connection-badge connected" : "connection-badge warning"}>{credentialAvailable ? "接続済み" : "再認証が必要"}</span>
                    </div>
                    <label className="field">
                      <span>同期するカレンダー</span>
                      <select value={account.calendarId} onFocus={() => void loadCalendars(account)} onChange={(event) => {
                        const selected = calendarOptions[account.id]?.find((calendar) => calendar.id === event.target.value);
                        void patchAccount(account.id, { calendarId: event.target.value, calendarName: selected?.name ?? event.target.selectedOptions[0]?.text ?? "", syncToken: "" });
                      }} disabled={!credentialAvailable || Boolean(account.lastSyncAt) || busy === `calendars-${account.id}`}>
                        {!calendars?.some((calendar) => calendar.id === account.calendarId) && <option value={account.calendarId}>{account.calendarName || account.calendarId}</option>}
                        {calendars?.map((calendar) => <option key={calendar.id} value={calendar.id}>{calendar.name}{calendar.primary ? "（メイン）" : ""}</option>)}
                      </select>
                    </label>
                    {account.lastSyncAt && <small className="calendar-lock-note">同期開始後に対象カレンダーを変える場合は、いったん接続を解除して接続し直してください。</small>}
                    <div className="google-account-actions">
                      <label className="toggle-field"><input type="checkbox" checked={account.syncEnabled} onChange={(event) => void patchAccount(account.id, { syncEnabled: event.target.checked })} /><span className="toggle-track" /><span>このアカウントと同期</span></label>
                      <span>
                        {!credentialAvailable && <button type="button" className="secondary-button" onClick={() => void reauthenticate(account)} disabled={Boolean(busy) || syncBusy}>再認証</button>}
                        <button type="button" className="danger-button" onClick={() => void disconnect(account)} disabled={Boolean(busy) || syncBusy}>接続解除</button>
                      </span>
                    </div>
                    {account.lastSyncAt && <small className="last-sync">最終同期: {new Date(account.lastSyncAt).toLocaleString("ja-JP")}</small>}
                    {account.lastError && <p className="google-error">{account.lastError}</p>}
                  </article>
                );
              })}
            </div>
          )}
          {google.accounts.length > 0 && (
            <div className="default-sync-targets">
              <div className="default-sync-heading">
                <span>
                  <strong>新しい予定の既定の保存先</strong>
                  <small>新規予定で自動選択します。予定ごとに解除・変更できます。</small>
                </span>
                {activeAccounts.length > 0 && (
                  <button
                    type="button"
                    className="text-button"
                    disabled={defaultTargetsBusy || Boolean(busy) || syncBusy}
                    onClick={() => void saveDefaultSyncTargets(
                      selectedDefaultCount === activeAccounts.length ? [] : activeAccounts.map((account) => account.id),
                    )}
                  >
                    {selectedDefaultCount === activeAccounts.length ? "すべて解除" : "すべて選択"}
                  </button>
                )}
              </div>
              {activeAccounts.length === 0 ? (
                <p className="google-empty">「このアカウントと同期」をONにすると既定の保存先へ選べます。</p>
              ) : (
                <div className="sync-target-list default-sync-list">
                  {activeAccounts.map((account) => (
                    <label key={account.id}>
                      <input
                        type="checkbox"
                        checked={google.defaultSyncTargets.includes(account.id)}
                        disabled={defaultTargetsBusy || Boolean(busy) || syncBusy}
                        onChange={(event) => void toggleDefaultSyncTarget(account.id, event.target.checked)}
                      />
                      <span>
                        <strong>{account.displayName || account.email}</strong>
                        <small>{account.calendarName || account.email}</small>
                      </span>
                    </label>
                  ))}
                </div>
              )}
              <p className="default-sync-note">何も選ばない場合、新しい予定は従来どおりローカルだけへ保存します。</p>
            </div>
          )}
          <p className="native-note">認証用の更新トークンはWindows資格情報マネージャーへ保存されます。別のPCでは同じアカウントを再認証してください。</p>
        </div>
      )}
      {status && <p className="settings-status" role="status">{status}</p>}
    </section>
  );
}
