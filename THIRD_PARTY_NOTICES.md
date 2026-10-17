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
- zip 9.0.4 — MIT License, https://github.com/zip-rs/zip2 (ZIPバックアップ読書き、未改変、実行時に静的リンク)
- typed-path 0.12.3 — MIT OR Apache-2.0 License, https://github.com/chipsenkbeil/typed-path (zipの依存、MITを選択、未改変)
- flate2 1.1.9 — MIT OR Apache-2.0 License, https://github.com/rust-lang/flate2-rs (既存依存をZIP圧縮にも利用、MITを選択、未改変)
- sha2 0.10.9 — MIT OR Apache-2.0 License, https://github.com/RustCrypto/hashes (既存依存をZIPのSHA-256照合にも利用、MITを選択、未改変)
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

正確な依存バージョンと推移的依存関係は `package-lock.json` と `src-tauri/Cargo.lock` に記録されています。各依存関係のライセンス全文と著作権表示は、それぞれの配布元およびパッケージに含まれるライセンスファイルを参照してください。

Koyomado本体はApache License 2.0で提供します。詳しくは `LICENSE.txt` と `NOTICE` を確認してください。

## ZIPバックアップ関連ライセンス本文（2026-10-17確認）


### flate2-1.1.9-LICENSE-MIT.txt

Copyright (c) 2014-2026 Alex Crichton

Permission is hereby granted, free of charge, to any
person obtaining a copy of this software and associated
documentation files (the "Software"), to deal in the
Software without restriction, including without
limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software
is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice
shall be included in all copies or substantial portions
of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.


### sha2-0.10.9-LICENSE-MIT.txt

Copyright (c) 2006-2009 Graydon Hoare
Copyright (c) 2009-2013 Mozilla Foundation
Copyright (c) 2016 Artyom Pavlov

Permission is hereby granted, free of charge, to any
person obtaining a copy of this software and associated
documentation files (the "Software"), to deal in the
Software without restriction, including without
limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software
is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice
shall be included in all copies or substantial portions
of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.


### typed-path-0.12.3-LICENSE-MIT.txt

Permission is hereby granted, free of charge, to any
person obtaining a copy of this software and associated
documentation files (the "Software"), to deal in the
Software without restriction, including without
limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software
is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice
shall be included in all copies or substantial portions
of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.


### zip-9.0.4-LICENSE.txt

The MIT License (MIT)

Copyright (c) 2014 Mathijs van de Nes

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.


## 配布用ライセンス本文・対応ソース・素材の出典

正確な依存バージョンと推移的依存関係は `package-lock.json` と `src-tauri/Cargo.lock` に記録されています。配布物には `third_party/dependency-licenses/` の版・取得元・SPDX表現・著作権通知・ライセンス本文・ハッシュ対応表を同梱します。npmの実行時依存7件と、Windows対象のCargo依存307件（ビルド時のみの依存も保守的に含む）が対象です。台帳掲載だけで全項目が最終EXEへ含まれることを意味しません。1.1.0で追加したバックアップ用依存4件のライセンス全文は third_party/rust-backup/ と本文末尾にも収録しています。

MPL-2.0のコンポーネントは元のファイル単位の条件を維持し、Apache-2.0へ変更しません。未改変の対応ソース5件を `third_party/mpl-source/` に同梱し、取得URLとCargo.lockに照合したSHA-256を同ディレクトリの `index.json` に示します。

画像・ロゴ・説明用スクリーンショットの由来と確認範囲は `ASSET_PROVENANCE.md` と `docs/asset-manifest.json` を参照してください。Google Cloud画面の第三者UI・商標は本体のApache-2.0で再許諾しません。Googleによる推奨・提携を示すものではありません。
