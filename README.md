# Koyomado

[English](README.en.md) | 日本語

Koyomadoは、Windowsのデスクトップへウィジェットのように置いて使える、シンプルなカレンダーです。一般利用向けのMicrosoft Store版と、USBメモリやGoogle Driveへフォルダーごと持ち運べるポータブル版を用意します。

- 公式ページ: https://ytec.cloudfree.jp/forge/projects/koyomado/
- 対応OS: Windows 10 / 11（64bit）
- ライセンス: Apache License 2.0

## 主な機能

- 月カレンダーで予定を確認、追加、編集、削除
- 開始日・終了日を指定した連休、出張、日をまたぐ時刻付き予定
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
- 80～130%を5%刻みで調整できる表示倍率スライダー
- Windows起動時の自動起動
- タスクバーのみ、タスクトレイのみ、両方の3種類から選べる表示方法
- 予定ごとのポップアップ通知、5種類の標準通知音、音量・再生秒数の調整
- MP3 / M4A / AAC / WAV / OGG / Opus / FLAC / MIDIから選べるユーザー通知音
- 任意で有効にできるGoogleカレンダー双方向同期（最大3アカウント）

印刷、PDF出力、広告、アクセス解析、Koyomado独自のクラウドサーバーは含みません。

## 初回の使い方

1. Microsoft Store版はStoreからインストールします。ポータブル版は配布ZIPを今後使う場所へ展開します。
2. Store版はスタートメニュー、ポータブル版は`koyomado.exe`から起動します。
3. 右上の歯車から、表示方法とWindows自動起動を好みに合わせて設定します。
4. ウィンドウを好きな位置とサイズへ調整します。同じモニター構成では、次回からその位置で開きます。

標準設定ではタスクバーだけに表示し、閉じるボタンでアプリを終了します。「タスクトレイのみ」または「両方」を選んだ場合は、閉じる・最小化で画面を隠し、トレイメニューの「終了」で完全に終了できます。

Microsoft Store版の自動起動はWindowsのStartupTaskを使用します。Windows側で無効化した場合は、「設定」-「アプリ」-「スタートアップ」から有効にしてください。ポータブル版は従来どおり、Windowsサインイン後に実行ファイルが利用可能になるまで最大5分待つ方式です。ポータブル版のフォルダーを移動するときは、移動前にOFF、移動後にONにし直してください。

## 開始・終了日時と複数日の予定

予定には開始日と終了日を指定できます。終日をONにすると、9月1日から3日までの休みや出張のような期間予定として、期間中の各日に表示します。終日をOFFにすると開始・終了時刻も指定でき、開始時刻を変更したときは終了時刻を1時間後へ自動設定します。終了日と終了時刻は、その後に自由に変更できます。

複数日の予定をコピー・貼り付け、ドラッグ移動、Ctrl＋ドラッグでコピーした場合は、元の期間を保ったまま新しい開始日へ移します。繰り返し予定でも各回の期間を保ちます。

## 繰り返し予定と記念日

予定の追加・編集画面で「繰り返し」を有効にすると、間隔と終了条件を指定できます。

- 毎日: 1日ごと、2日ごとなど
- 毎週: 曜日を複数選択可能
- 毎月: 同じ日付、または第何週の同じ曜日
- 毎年: 誕生日や記念日に利用
- 終了: なし、指定日まで、指定回数

カレンダー上の繰り返し予定を編集・削除するときは、「この予定のみ」と「繰り返し全体」を選べます。1回分だけをドラッグした場合は、その回だけを別の日へ移動します。Ctrlキーを押しながらドラッグした場合は、独立した予定としてコピーします。

## Googleカレンダー連携

Google連携は初期状態でOFFです。利用者自身がGoogle Cloudでデスクトップアプリ用OAuthクライアントを作成し、ダウンロードしたJSONをKoyomadoへ読み込んだ場合だけ通信します。APIキーではなくOAuth 2.0クライアントを使用し、Y-TEC共通OAuthクライアントは採用しないため、他利用者のAPI利用がY-TECへ集約されることはありません。

接続できるGoogleアカウントは3件までです。アカウントごとに同期するカレンダーを1つ選び、同期のON/OFFを切り替えられます。「新しい予定の既定の保存先」では、1件、複数件、すべて、またはローカルのみを選べます。新規予定では既定の保存先が自動選択されますが、予定ごとに解除・変更できます。

同期は、起動・再表示・約60秒ごと・予定変更直後・「今すぐ同期」で行います。予定名、開始・終了日時、終日、場所、メモ、繰り返し、リマインダーを双方向に反映します。新しくKoyomadoで作る予定は「上の通知時刻をGoogleにも保存する」が初期選択です。予定ごとにGoogle側の既定リマインダーへ切り替えることもできますが、その場合はKoyomadoで指定した通知時刻ではなく、対象カレンダーの既定通知がGoogle側で使われます。KoyomadoとGoogleで同じ予定を同時に変更した場合は、内容を勝手に上書きせず、競合した2件を残して画面で知らせます。

Google OAuthクライアントの作成から接続までの実画面付き手順は、配布ZIPと公式ページの操作説明書に掲載します。常用時は、利用者自身のOAuthプロジェクトを個人利用としてGoogle Auth Platformの「In production」へ切り替えます。個人利用の通常手順ではOAuth検証申請、公開ホームページ、プライバシーポリシー、承認済みドメインの登録は行いません。初回認証時に未確認アプリの警告が表示される場合がありますが、自分で作成したプロジェクトであることを確認して進みます。未検証プロジェクトには100新規ユーザーの上限がありますが、利用者ごとに自分のプロジェクトを作成し、最大3アカウントだけを接続するKoyomadoの方式では通常影響しません。Google Cloudの「Testing」のままでは認証が原則7日で期限切れになるため、常用しません。KoyomadoからGoogle Cloud側の公開状態を判定・変更することはできません。

新しいOAuthクライアントの完全なシークレットを含むJSONは、作成時にだけダウンロードできます。Koyomadoへ読み込んだ後も元のJSONを公開せず安全な場所へバックアップしてください。紛失した場合は、Google Auth Platformでクライアントシークレットをローテーションし、新しいJSONをダウンロードする方法が最も簡単です。ローテーションできない場合は、新しいDesktopクライアントを作成します。

予定内容と同期設定は利用中の版のJSONへ保存しますが、Googleの更新トークンはWindows資格情報マネージャーへ保存します。別のPCへポータブル版を移した場合、そのPCでGoogleアカウントを再認証してください。詳しくは[プライバシーポリシー](PRIVACY.md)を確認してください。

## データ保存と配布方法

| 配布方法 | データ保存先 | 更新方法 | 署名 |
| --- | --- | --- | --- |
| Microsoft Store版 | `%LOCALAPPDATA%\Packages\Y-TEC.Koyomado_y7q84f7nwz24j\LocalState\Koyomado\data` | Microsoft Store | Microsoft Storeによる配布署名 |
| ポータブル版 | `koyomado.exe`と同じ場所の`data` | ZIPを展開し、既存`data`を引き継ぐ | Y-TEC自己署名＋公開SHA-256 |

二つの版は保存先が別です。切り替える場合は両方を終了し、Store版を一度起動して終了してから、元の`data`フォルダーの中身を新しい保存先へコピーしてください。移行元は削除せず、予定・設定・通知音を確認できるまでバックアップとして残します。同じGoogle予定への重複同期や二重通知を避けるため、Store版とポータブル版を同時起動しないでください。同一PCではWindows資格情報マネージャーのGoogle更新トークンを共有するため、一方で接続解除すると他方でも再認証が必要になります。

Store版のデータはMSIXの`LocalState`にあり、Store更新では通常維持されますが、アプリをアンインストールすると削除される可能性があります。アンインストールやPC初期化の前に、Koyomadoを終了して上記の`data`フォルダーを別の場所へコピーしてください。ポータブル版の`data`はZIP更新で上書きしない限り実行ファイル横へ残ります。

### dataフォルダーの内容

- `data/calendar-data.json`: 予定、繰り返し、外観、Google連携設定
- `data/calendar-data.backup.json`: 直前の保存内容
- `data/calendar-data.v1.backup.json` / `calendar-data.v2.backup.json` / `calendar-data.v3.backup.json` / `calendar-data.v4.backup.json`: 旧形式からの移行前データ（移行時のみ）
- `data/notification-sounds/`: ユーザーが選んだ通知音（設定した場合のみ）
- `data/window-state.json`: モニター構成ごとの位置とサイズ（直近12構成）
- `data/window-state.backup.json`: 直前の位置情報
- `data/window-state.v1.backup.json`: 旧位置形式からの移行前データ（移行時のみ）

保存形式version 1 / 2 / 3 / 4は初回起動時にversion 5へ自動移行し、移行前ファイルを残します。予定の削除はソフトデリートで、削除済みデータを `deletedEvents` に保持します。

## 予定の通知

予定の追加・編集画面で、開始時刻の0分前から最大28日前まで、予定ごとにポップアップ通知を設定できます。10分前・30分前・1時間前・3時間前・6時間前・12時間前・1日前は、よく使う通知時間からクリックだけで複数選択・解除できます。入力単位は「分前」「時間前」「日前」から自由に指定することもできます。内部ではGoogleカレンダー互換の分数として保存・同期します。終日予定は開始日の0:00が基準です。Google側で既定の30分前通知を使うと、終日予定では前日の23:30になります。通知はKoyomadoが起動している間だけ動作します。通知音は初期設定の12秒を含む3～60秒から選んだ長さで自動停止し、通知ポップアップの「OK（音を止める）」を押した場合はすぐ停止します。

設定画面では「やわらぎ」「深い雫」「小鈴」「朝露のピアノ」「木漏れ日のカリンバ」の5種類、音なし、またはユーザー音源を選択できます。ユーザー音源は15MBまでのMP3 / M4A / AAC / WAV / OGG / Opus / FLAC / MIDIに対応します。MIDIは内蔵の穏やかな音色で再生するため、元ファイルの音源や音色とは異なる場合があります。実際に再生できる形式はWindows WebView2のコーデックにも依存します。

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
npm run test:windows-signing-policy
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test-partner-center-submission.ps1 -SkipArtifactHashCheck
Push-Location src-tauri
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
Pop-Location
npm run tauri:build
```

実際のGoogleアカウントを使う結合テストは、利用者自身のテスト用OAuthクライアントと合成予定だけで行います。OAuthクライアントJSONやトークンはGit、ログ、Issueへ含めないでください。

### Microsoft Store向けMSIX

Windows SDKのMakeAppxを使用します。Partner Centerで割り当てられたPackage Identityを推測せず、`packaging/msix/AppxManifest.xml.in`の値と提出先を一致させてください。

```powershell
npm run tauri:build
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/package-msix.ps1 -SkipBuild -PackageVersion 1.0.0.0 -CreateUpload
npm run test:store-submission
```

Store提出用MSIXはローカルで自己署名せず、Partner Centerの署名を使用します。`release/koyomado-v<version>-store-x64.msixupload`を生成します。

### 自己署名ポータブルZIP

自己署名証明書はWindowsの`CurrentUser\My`へ非エクスポート鍵として作成し、PFXや秘密鍵をワークスペースへ置きません。最終リリースは、ビルド後に署名用コピーを作り、そのコピーだけを署名・検証・ZIP化します。

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/new-self-signed-code-signing-certificate.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/package-self-signed-direct.ps1 -CertificateThumbprint <表示されたThumbprint>
```

`release/koyomado-v<version>-windows-portable.zip`、公開鍵だけを含む`.cer`、操作説明書、`SHA256SUMS.txt`を生成します。署名にはRFC 3161タイムスタンプを付けます。自己署名はSmartScreen警告をなくす保証ではありません。利用者のTrusted Rootへ証明書を自動登録しません。

開発確認用の未署名ZIPだけを作る場合は次を使えます。未署名ZIPを正式公開物として扱いません。

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

Microsoft Store版と自己署名ポータブル版を同じ表現で案内せず、署名方式、更新方法、保存先をGitHub Releaseと公式ページへ明記します。公開ファイルはSHA-256を照合してください。コード署名の運用は[Code signing policy](CODE_SIGNING_POLICY.md)に従います。

Copyright 2026 Y-TEC. Licensed under the Apache License, Version 2.0.
