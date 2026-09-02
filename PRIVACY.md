# Koyomado プライバシーポリシー

最終更新日: 2026年9月2日

Koyomadoは、予定を利用者のWindows PC内で管理するカレンダーです。Microsoft Store版とポータブル版を用意し、Googleカレンダー連携は任意で、利用者が設定画面で有効にした場合だけ外部通信を行います。

## ローカルで保存する情報

- Microsoft Store版では、予定、繰り返し条件、表示テーマ、Google連携の設定をMSIXのアプリ専用領域 `%LOCALAPPDATA%\Packages\Y-TEC.Koyomado_y7q84f7nwz24j\LocalState\Koyomado\data\calendar-data.json` へ保存します。ウィンドウ位置とサイズは同じ場所の `window-state.json` へ保存します。
- ポータブル版では、従来どおりKoyomadoの実行ファイル横にある `data/calendar-data.json` と `data/window-state.json` へ保存します。
- Google OAuthクライアントJSONから読み込んだクライアントID、デスクトップアプリ用クライアントシークレット、プロジェクトIDは `calendar-data.json` へ保存します。このファイルは暗号化しません。
- Googleの更新トークンはJSONへ保存せず、Windows資格情報マネージャーへ保存します。そのため、Koyomadoのフォルダーを別のPCへ移動した場合は、そのPCで再認証が必要です。

## Google連携時にアクセスする情報

利用者がGoogle連携を有効にして明示的にアカウントを接続した場合、KoyomadoはGoogleのOAuth 2.0とGoogle Calendar APIを利用して次の情報へアクセスします。

- 接続したGoogleアカウントの識別情報（メールアドレス、表示名）
- 利用者が選んだGoogleカレンダーの一覧情報と予定
- KoyomadoとGoogleカレンダー間で追加、更新、削除する予定

これらの情報は、カレンダー同期機能を提供する目的だけに使用します。Y-TECのサーバー、広告事業者、分析サービス、その他の第三者へ送信・販売・共有しません。人がGoogleカレンダーの内容を閲覧する仕組みもありません。

Google Workspace APIから受け取った情報の利用は、[Google API Services User Data Policy](https://developers.google.com/terms/api-services-user-data-policy)およびLimited Use要件に従います。

## 通信先

Google連携を有効にした場合だけ、Googleが提供する認証・アカウント・Calendar APIのエンドポイントへHTTPSで通信します。Koyomado独自の中継サーバーは使用しません。Google連携を無効にしている間は、Koyomadoの機能による外部通信を行いません。

## 接続解除と削除

設定画面でGoogleアカウントの接続を解除すると、Googleへの認可取り消しを試み、Windows資格情報マネージャーの更新トークンとKoyomado内の同期リンクを削除します。Googleから取り込んだ予定は、利用者が確認できるようローカル予定として残します。Google側の予定を削除したい場合は、接続解除の前にGoogleカレンダーまたはKoyomadoで削除してください。

Koyomadoのローカルデータをすべて削除する場合は、Koyomadoを終了してから、Microsoft Store版は `%LOCALAPPDATA%\Packages\Y-TEC.Koyomado_y7q84f7nwz24j\LocalState\Koyomado\data`、ポータブル版は実行ファイル横の `data` フォルダーを削除してください。二つの版を利用した場合は保存先が別々に残るため、必要な予定をバックアップしてから各保存先を確認してください。Store版のアンインストールではアプリ専用`LocalState`が削除される可能性があるため、残したい予定はアンインストール前に別の場所へバックアップしてください。

## 問い合わせ

不具合やプライバシーに関する連絡は、[Y-TEC Forgeのお問い合わせ](https://ytec.cloudfree.jp/forge/contact/)またはKoyomadoのGitHubリポジトリをご利用ください。予定内容、OAuthクライアントJSON、認証トークンなどの秘密情報は公開Issueへ貼り付けないでください。
