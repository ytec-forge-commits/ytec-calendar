# Third-party notices

Koyomadoは、次のオープンソースソフトウェアとフォントを利用しています。

## 実行時の主な依存関係

- React / React DOM — MIT License
- Tauri / Tauri API — Apache-2.0 OR MIT License
- auto-launch — MIT License
- rrule — BSD 3-Clause License
- holiday-japanese — Copyright 2024 GAHOJIN, Inc., Apache License 2.0
- chrono — MIT OR Apache-2.0 License
- oauth2 — MIT OR Apache-2.0 License
- reqwest — MIT OR Apache-2.0 License
- serde / serde_json — MIT OR Apache-2.0 License
- url — MIT OR Apache-2.0 License
- windows-sys — MIT OR Apache-2.0 License
- windows — MIT OR Apache-2.0 License
- scheduler — MIT License
- tslib — 0BSD License
- LINE Seed JP — Copyright LY Corporation, SIL Open Font License 1.1

## 同梱通知音

次の標準通知音はCC0 1.0（パブリックドメイン提供）素材です。Koyomadoでは一部を音量調整・約10.5秒への切り出し・フェード処理しています。

- 「やわらぎ」「深い雫」「小鈴」— Robin Lamb, [UI Sound Effects](https://opengameart.org/content/ui-sound-effects-button-clicks-user-feedback-notifications)
- 「朝露のピアノ」— jestar, [A Simple Trifle](https://opengameart.org/content/a-simple-trifle)
- 「木漏れ日のカリンバ」— extenz, [Short kalimba loop](https://opengameart.org/content/short-kalimba-loop)

出典と編集内容の詳細は `third_party/opengameart-cc0-notification-sounds/NOTICE.txt` に記録しています。

## 開発時の主な依存関係

- Vite — MIT License
- Vitest — MIT License
- ESLint and related plugins — MIT License
- TypeScript — Apache License 2.0

LINE Seed JPのライセンス全文は `src/assets/fonts/OFL.txt` に同梱し、ポータブルZIPでは `LINE_Seed_JP_OFL.txt` として収録します。

正確な依存バージョンと推移的依存関係は `package-lock.json` と `src-tauri/Cargo.lock` に記録されています。配布物には `third_party/dependency-licenses/` の版・取得元・SPDX表現・著作権通知・ライセンス本文・ハッシュ対応表を同梱します。npmの実行時依存7件と、Windows対象のCargo依存305件（ビルド時のみの依存も保守的に含む）が対象です。台帳掲載だけで全項目が最終EXEへ含まれることを意味しません。依存の版・コード・既存条件は変更していません。

MPL-2.0のコンポーネントは元のファイル単位の条件を維持し、Apache-2.0へ変更しません。未改変の対応ソース5件を `third_party/mpl-source/` に同梱し、取得URLとCargo.lockに照合したSHA-256を同ディレクトリの `index.json` に示します。

画像・ロゴ・説明用スクリーンショットの由来と確認範囲は `ASSET_PROVENANCE.md` と `docs/asset-manifest.json` を参照してください。Google Cloud画面の第三者UI・商標は本体のApache-2.0で再許諾しません。Googleによる推奨・提携を示すものではありません。

Koyomado本体はApache License 2.0で提供します。詳しくは `LICENSE.txt` と `NOTICE` を確認してください。
