# Koyomado

Koyomadoは、Windowsのデスクトップへウィジェットのように置いて使える、シンプルなポータブル型カレンダーです。予定と設定は実行ファイル横へ保存され、フォルダーごとUSBメモリやGoogle Driveへ移動できます。

- 公式ページ: https://ytec.cloudfree.jp/ytb/koyomado/
- 対応OS: Windows 10 / 11（64bit）
- ライセンス: Apache License 2.0

## 主な機能

- 月カレンダーで予定を確認、追加、編集、削除
- 予定の右クリックコピー・貼り付け・削除
- ドラッグで移動、Ctrlキーを押しながらドラッグでコピー
- 毎日、毎週、毎月、毎年の繰り返し予定
- 毎年同じ月日の誕生日・記念日
- 繰り返し予定の1回だけ、または全体の編集・削除
- 予定のある日を選んだときの一覧ポップアップ
- 土曜、日曜、日本の祝日の色分けと祝日名表示（1970～2050年を内蔵）
- 同じ日に5件以上、月に10件以上あっても崩れない省略表示と一覧確認
- LINE Seed JPで統一した表示、予定6色、背景テーマ8種
- 開閉状態を記憶する折りたたみ式サイドバー
- モニター構成ごとのウィンドウ位置とサイズの保存
- Windows起動時の自動起動
- タスクバーのみ、タスクトレイのみ、両方の3種類から選べる表示方法
- 任意で有効にできるGoogleカレンダー双方向同期（最大3アカウント）

印刷、PDF出力、広告、アクセス解析、Koyomado独自のクラウドサーバーは含みません。

## 初回の使い方

1. 配布ZIPを今後使う場所へ展開します。
2. `koyomado.exe` を起動します。
3. 右上の歯車から、表示方法とWindows自動起動を好みに合わせて設定します。
4. ウィンドウを好きな位置とサイズへ調整します。同じモニター構成では、次回からその位置で開きます。

標準設定ではタスクバーだけに表示し、閉じるボタンでアプリを終了します。「タスクトレイのみ」または「両方」を選んだ場合は、閉じる・最小化で画面を隠し、トレイメニューの「終了」で完全に終了できます。

自動起動をONにした後でアプリのフォルダーを移動するときは、移動前にOFF、移動後にONにし直してください。

## 繰り返し予定と記念日

予定の追加・編集画面で「繰り返し」を有効にすると、間隔と終了条件を指定できます。

- 毎日: 1日ごと、2日ごとなど
- 毎週: 曜日を複数選択可能
- 毎月: 同じ日付、または第何週の同じ曜日
- 毎年: 誕生日や記念日に利用
- 終了: なし、指定日まで、指定回数

カレンダー上の繰り返し予定を編集・削除するときは、「この予定のみ」と「繰り返し全体」を選べます。1回分だけをドラッグした場合は、その回だけを別の日へ移動します。Ctrlキーを押しながらドラッグした場合は、独立した予定としてコピーします。

## Googleカレンダー連携

Google連携は初期状態でOFFです。利用者自身がGoogle Cloudでデスクトップアプリ用OAuthクライアントを作成し、ダウンロードしたJSONをKoyomadoへ読み込んだ場合だけ通信します。APIキーではなくOAuth 2.0クライアントを使用するため、Y-TECへAPI利用料が請求される共通キーはありません。

接続できるGoogleアカウントは3件までです。アカウントごとに同期するカレンダーを1つ選び、同期のON/OFFを切り替えられます。Koyomadoで予定を追加するときは、ローカルだけ、特定アカウント、接続中の全アカウントから保存先を選べます。

同期は、起動・再表示・約60秒ごと・予定変更直後・「今すぐ同期」で行います。KoyomadoとGoogleで同じ予定を同時に変更した場合は、内容を勝手に上書きせず、競合した2件を残して画面で知らせます。

Google OAuthクライアントの作成から接続までの画面付き手順は、配布ZIPと公式ページの操作説明書に掲載します。Google Cloudの「テスト」状態では認証が7日で期限切れになるため、個人利用でも公開ステータスと警告の意味を説明書で確認してください。

予定内容と同期設定は実行ファイル横のJSONに保存しますが、Googleの更新トークンはWindows資格情報マネージャーへ保存します。別のPCへフォルダーを移した場合、そのPCでGoogleアカウントを再認証してください。詳しくは[プライバシーポリシー](PRIVACY.md)を確認してください。

## ポータブルデータ

- `data/calendar-data.json`: 予定、繰り返し、外観、Google連携設定
- `data/calendar-data.backup.json`: 直前の保存内容
- `data/calendar-data.v1.backup.json` / `calendar-data.v2.backup.json`: 旧形式からの移行前データ（移行時のみ）
- `data/window-state.json`: モニター構成ごとの位置とサイズ（直近12構成）
- `data/window-state.backup.json`: 直前の位置情報
- `data/window-state.v1.backup.json`: 旧位置形式からの移行前データ（移行時のみ）

保存形式version 1 / 2は初回起動時にversion 3へ自動移行し、移行前ファイルを残します。予定の削除はソフトデリートで、削除済みデータを `deletedEvents` に保持します。

Google Drive上で使う場合、同じフォルダーを複数PCから同時に起動しないでください。Google Driveそのもののファイル競合はKoyomadoでは解決できません。

## 開発

必要な環境はNode.js、npm、Rust stable、Tauri 2が要求するWindows開発環境です。

```powershell
npm ci
npm run tauri:dev
```

Web画面だけを確認するときは `npm run dev` を使えます。ブラウザー版ではlocalStorageへ合成データを保存し、Google連携、自動起動、タスクトレイなどのWindows機能は動きません。

### 検証

```powershell
npm run lint
npm test
npm run build
Push-Location src-tauri
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
Pop-Location
npm run tauri:build
```

実際のGoogleアカウントを使う結合テストは、利用者自身のテスト用OAuthクライアントと合成予定だけで行います。OAuthクライアントJSONやトークンはGit、ログ、Issueへ含めないでください。

### ポータブルZIP

```powershell
npm run tauri:portable
```

`release/koyomado-v<version>-windows-portable.zip` を生成します。更新時は既存の `data` フォルダーを残し、実行ファイルと同梱文書だけを差し替えてください。

## セキュリティと公開

- [プライバシーポリシー](PRIVACY.md)
- [セキュリティポリシー](SECURITY.md)
- [Code signing policy](CODE_SIGNING_POLICY.md)
- [Contribution guide](CONTRIBUTING.md)
- [第三者ライセンス](THIRD_PARTY_NOTICES.md)

配布物が署名済みか未署名かは、GitHub Releaseと公式ページへ明記します。未署名版ではSHA-256を照合してください。コード署名の運用は[Code signing policy](CODE_SIGNING_POLICY.md)に従います。

Copyright 2026 Y-TEC. Licensed under the Apache License, Version 2.0.
