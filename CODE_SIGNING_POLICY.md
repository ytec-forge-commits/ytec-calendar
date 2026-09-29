# Code signing policy

KoyomadoのWindows配布物は、Microsoft Store版と直接配布のポータブル版で署名経路を分離します。二つの経路を同じ「署名済み」という表現だけで案内せず、実際の署名者、更新方法、SHA-256を公開ページへ明記します。

## Microsoft Store版

- Partner CenterのKoyomado製品へ、割り当て済みPackage Identityを持つ未署名MSIXを提出します。
- Store提出用MSIXをY-TECの自己署名証明書では署名しません。
- 認定後にMicrosoft Storeが配布署名を適用し、更新もMicrosoft Storeから配信します。
- Store提出用パッケージと直接配布ZIPは、同じアプリ版数へ対応付けます。MSIXのPackage versionは四区分数値で、第四区分を`0`にします。

## 直接配布ポータブル版

- 管理されたWindows端末の`CurrentUser\My`証明書ストアにある、Y-TEC自己署名コード署名証明書を使用します。
- 既存証明書を再利用するときも、Subject=Issuerの自己署名、Code Signing EKU、秘密鍵あり、秘密鍵が非エクスポートであることを署名前に機械確認します。確認できない証明書は使用しません。
- 秘密鍵は非エクスポートとし、PFX、秘密鍵、パスワードをワークスペース、Git、GitHub Actions、Release、公式サイト、ログへ保存しません。
- ビルド済みEXEの署名用コピーだけを署名し、署名者と暗号署名を検証してからZIPへ収録します。
- RFC 3161タイムスタンプを付け、署名時刻を検証できる状態にします。
- 公開用`.cer`は公開鍵だけを含む検証補助物です。利用者のTrusted RootまたはTrusted Publishersへ証明書を自動登録しません。
- 自己署名はSmartScreenやセキュリティ製品の警告をなくす保証ではありません。ダウンロード元、署名者、公開SHA-256を併記します。

## Team roles

- Committers and reviewers: [ytec-forge-commits organization members](https://github.com/orgs/ytec-forge-commits/people)
- Release approvers: [ytec-forge-commits organization owners](https://github.com/orgs/ytec-forge-commits/people?query=role%3Aowner)
- Store submission: KoyomadoへアクセスできるPartner Centerアカウント
- Direct signing: Y-TECが管理するローカル署名端末

外部からのPull Requestは、リポジトリ管理者が内容とCI結果を確認してから取り込みます。署名、Store提出、GitHub Release公開、公式サイト更新は、候補成果物と最終検証が一致した後に行います。

## Privacy

Koyomadoは、利用者が設定画面でGoogleカレンダー連携を明示的に有効にした場合を除き、他のネットワークシステムへ情報を送信しません。Google連携時の取扱いは[プライバシーポリシー](PRIVACY.md)に記載しています。OAuthクライアントJSON、トークン、実予定をMSIX、ZIP、Artifact、ログへ含めません。

## Release process

1. `main`ブランチの候補でlint、TypeScriptテスト、Webビルド、Rust test、Clippy、Windowsネイティブビルドを実行します。
2. 同じソースからStore向け未署名MSIXと、直接配布用の未署名EXEを生成します。
3. MSIXはIdentity、StartupTask、capability、必須ファイル、ライセンス表記を展開検証し、Partner Centerへ提出します。
4. 直接配布用EXEは管理された端末で自己署名し、署名者と署名内容を検証します。
5. 検証済みの署名EXE、操作説明書、ライセンス、更新履歴をポータブルZIPへまとめます。
6. 最終MSIX提出物、ZIP、操作説明書、公開鍵証明書ごとにSHA-256を生成します。ハッシュ生成後に成果物を書き換えません。
7. Store認定結果と直接配布物を確認し、GitHub Releaseと公式紹介ページへ署名方式、版数、SHA-256、保存先の違いを掲載します。

GitHub Actionsは、再現可能な未署名EXEとStore提出パッケージを生成する署名入力工程として使用できます。自己署名秘密鍵をActionsへ登録せず、未署名Artifactを公開用Releaseへ自動掲載しません。

将来SignPath Foundationまたは承認済みCA証明書を採用した場合も、上記の秘密情報保護、最終成果物検証、SHA-256、配布経路の分離を維持します。
