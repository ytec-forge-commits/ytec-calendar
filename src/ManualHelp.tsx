import { useState } from "react";
import { openUserManual } from "./lib/store";

export function ManualHelp() {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const open = async () => {
    setBusy(true);
    setError("");
    try {
      await openUserManual();
    } catch (reason) {
      setError(String(reason));
    } finally {
      setBusy(false);
    }
  };
  return (
    <section className="settings-section">
      <div className="settings-heading"><div><h3>操作説明書</h3><p>同梱のPDFを既定のアプリで開きます。ネット接続は不要です。</p></div></div>
      <button type="button" className="secondary-button" disabled={busy} onClick={() => void open()}>{busy ? "開いています…" : "操作説明書を開く"}</button>
      {error && <p className="form-error" role="alert">{error}</p>}
    </section>
  );
}
