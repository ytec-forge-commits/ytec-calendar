# Contributing to Koyomado

Koyomadoへの改善提案や不具合報告を歓迎します。

## Issue

不具合では、Koyomadoのバージョン、Windowsのバージョン、再現手順、期待した結果、実際の結果を記載してください。スクリーンショットやテストデータへ、個人情報、予定内容、Googleの認証情報を含めないでください。

## Development

必要な環境はNode.js、npm、Rust stable、Tauri 2が要求するWindows開発環境です。

```powershell
npm ci
npm run lint
npm test
npm run build
Push-Location src-tauri
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
Pop-Location
npm run tauri:build
```

予定や設定は実行ファイル横の `data` に保存されます。開発・テストでは実際の予定を使用せず、合成データだけを使用してください。

## Pull request

- 変更理由と利用者への影響を説明してください。
- UI変更では、可能なら個人情報を含まないスクリーンショットを添付してください。
- 保存形式を変更する場合は、旧版読込、移行前バックアップ、回帰テストを含めてください。
- 新しい外部通信や依存関係を追加する場合は、目的、送信する情報、ライセンス、無効化方法を記載してください。

明示しない限り、提出されたContributionにはApache License 2.0が適用されます。
