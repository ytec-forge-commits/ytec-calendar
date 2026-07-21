# Koyomado 固有規則

- 正本ソースはこのディレクトリ。Windows専用のTauri 2アプリとして維持する。
- 予定・外観設定は実行ファイル横の `data/calendar-data.json`、ウィンドウ位置は `data/window-state.json` に保存する。
- 保存形式を変更するときは `version`、旧版読込、移行、更新前バックアップを維持する。
- 実データをテスト、スクリーンショット、Gitへ含めない。画面確認には合成予定だけを使う。
- 外部API、認証、アクセス解析、クラウド同期、印刷、PDF機能は、明示依頼なしに追加しない。
- UI書体は同梱したLINE Seed JPへ統一し、予定ごとのフォント装飾機能は追加しない。
- 配布はインストーラーではなくポータブルZIPを正本とする。アップデート時は利用者の `data` フォルダーを上書きしない。
- 完了前に `npm run lint`、`npm test`、`npm run build`、`npm run tauri:build`、可能ならWindows実機操作を確認する。
