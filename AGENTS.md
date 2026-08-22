# Koyomado 固有規則

- 正本ソースはこのディレクトリ。Windows専用のTauri 2アプリとして維持する。
- 予定・外観設定は実行ファイル横の `data/calendar-data.json`、ウィンドウ位置は `data/window-state.json` に保存する。
- 保存形式を変更するときは `version`、旧版読込、移行、更新前バックアップを維持する。
- 実データをテスト、スクリーンショット、Gitへ含めない。画面確認には合成予定だけを使う。
- Googleカレンダー連携以外の外部API、認証、アクセス解析、クラウド同期、印刷、PDF機能は、明示依頼なしに追加しない。
- Googleカレンダー連携は任意機能とし、OFFの間は外部通信しない。利用者自身のDesktop OAuthクライアントを読み込み、最大3アカウント、アカウントごとに1カレンダーを同期する。
- Googleの更新トークンは `calendar-data.json` へ保存せず、アカウントID単位でWindows資格情報マネージャーへ保存する。OAuthクライアントJSON、更新トークン、実予定をテスト、ログ、Git、Issueへ含めない。
- UI書体は同梱したLINE Seed JPへ統一し、予定ごとのフォント装飾機能は追加しない。
- 配布はインストーラーではなくポータブルZIPを正本とする。アップデート時は利用者の `data` フォルダーを上書きしない。
- 完了前に `npm run lint`、`npm test`、`npm run build`、`cargo test --locked`、`cargo clippy --locked --all-targets -- -D warnings`、`npm run tauri:build`、可能ならWindows実機操作を確認する。
- 保存形式version 3では繰り返し条件、例外予定、Google同期リンク、競合情報、表示方法を保持する。version 1 / 2から移行するときは `calendar-data.v1.backup.json` / `calendar-data.v2.backup.json` を残す。
- 実Google APIの結合確認は、利用者のテスト用OAuthクライアントと合成予定だけで行う。認証情報がない状態でライブ同期を確認済みと報告しない。
