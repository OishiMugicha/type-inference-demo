# Tiny ML — 型推論の実験室

『プログラミング言語の基礎概念』を読んだ後に、lexerから型推論・評価器までをRustで自作するためのひな形です。

**言語処理の中身は未実装です。** 型定義、関数の入口、練習用テスト、Wasmとの接続、Web画面、GitHub Pages用ワークフローを用意しています。サンプルの結果は、各段階を実装すると表示されるようになります。

## 起動する

Rust stable（rustup）、Node.js 24、npm、wasm-pack 0.15.0を使用します。

```sh
rustup show
cargo install wasm-pack --version 0.15.0 --locked
npm ci
npm run dev
```

表示されたローカルURLを開き、「実行する」を押してください。初期状態ではToken欄に「未実装」、後続の欄に「未実行」と表示されれば接続成功です。型や値を計算する実装は含んでいません。

`rust-toolchain.toml` にWasmターゲット・rustfmt・Clippyを指定しています。初回の `rustup show` で必要なコンポーネントが取得されます。

## 自分で実装するところ

| 入口 | ファイル | 入出力 |
| --- | --- | --- |
| `lex` | `crates/core/src/lexer.rs` | `&str → LangResult<Vec<Token>>` |
| `parse` | `crates/core/src/parser.rs` | `&[Token] → LangResult<Expr>` |
| `infer` | `crates/core/src/infer.rs` | `&Expr → LangResult<Type>` |
| `eval` | `crates/core/src/eval.rs` | `&Expr → LangResult<Value>` |

関数の `Err(Diagnostic::unimplemented(...))` を自分の実装に置き換えます。内部の補助関数やモジュール分割は自由です。parser生成器や型推論の補助アルゴリズムは入れていません。

1. [言語仕様](docs/language.md)を読んで、lexerのテストから始める。
2. parserを実装し、Web上のToken・ASTを確認する。
3. 評価器または型推論へ進む。どちらもASTを直接作るテストがあり、独立して実装できる。
4. 各段階のテストが通ったら `#[ignore = "exercise: …"]` を外し、通常の検証対象にする。

[練習用テストの実行方法](docs/exercises.md)には、段階ごとのコマンドをまとめています。仕様と期待結果だけを示し、解法のヒントは載せていません。

## 編集と検証

```sh
# Rustを編集したら、別ターミナルでWasmを再ビルド
npm run wasm:dev
# 完了後、ブラウザを再読み込み

# 通常の基盤チェック。未実装の練習用テストはスキップ
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked

# Wasm・TypeScript・Webの公開用ビルド
npm run build
npm run preview

# ブラウザを初回インストールし、ビルドからブラウザ検証まで実行
npx playwright install chromium
npm run test:e2e
```

TypeScript/CSSはViteで更新されます。Rust変更の自動監視は含めていません。`npm run typecheck` 単体を実行する前にも、生成された `web/pkg` が必要です。

Wasmビルド用ツールのキャッシュは `target/wasm-pack-cache` に置きます。初回ビルドにはツール取得のためのネットワーク接続が必要です。既存キャッシュを使う場合は `WASM_PACK_CACHE` で指定できます。

ブラウザ検証は、標準で `/type-inference-demo/` のサブパスで実際のWasmを読み込みます。中止・再実行、読み込み失敗からの再接続、狭い画面も確認します。Linuxでブラウザ用ライブラリが不足する場合は、Playwrightの案内に従って `npx playwright install --with-deps chromium` を実行してください。

## 構成とWebの動作

```text
crates/core/       言語のデータ型・未実装の入口・練習用テスト
crates/wasm/       Rustから表示用JSONへの変換と段階間の接続
web/src/           TypeScriptの画面とWeb Worker
docs/              言語仕様と練習の進め方
tests/             Playwrightのブラウザ検証
.github/workflows/ 検証とGitHub Pages公開
```

Wasmの `analyze(source)` はJSON文字列を返します。`tokens`・`ast`・`inferred_type`・`value` の各フィールドが、`success`（`output`）か `unimplemented` / `error` / `skipped`（`message`）を持ちます。対応するTypeScript型は `web/src/protocol.ts` にあります。

Token・ASTはRustのDebug形式、型・値はDisplay形式の文字列です。`i64`をJavaScriptの数値に変換せず、精度を保ちます。型変数は `'t0` のように表示し、クロージャの値は `<fun>` と表示します。画面では出力をテキストとして扱います。

lexer成功後にparserが動き、parser成功後に型推論・評価器がそれぞれ動きます。型推論の未実装や型エラーでも評価器は実行します。型エラーの例で型と値の両方にエラーが出ることがあります。先に成功した結果は、後段の通常エラーで失われません。

処理はWorker内で実行します。「中止」はWorkerを終了し、新しいWasmを読み込みます。Rustのpanicや読み込み失敗の場合は「再接続」でやり直せます。1回の要求につき全段階をまとめて返すため、無限ループ中の途中結果は表示しません。coreの通常エラーには `Diagnostic` を使ってください。

## GitHub Pagesに公開する

1. このひな形をGitHubのリポジトリへ追加する。`Cargo.lock` と `package-lock.json` もコミットする。
2. リポジトリの **Settings → Pages → Build and deployment → Source** を **GitHub Actions** にする。
3. `main` にpushする。Actionsの「Check and deploy Pages」が検証・ビルド・公開する。

PRでは検証だけを実行します。手動実行は `main` を選ぶと公開まで進みます。生成物の `web/pkg`・`web/dist` はコミット不要です。

公開URLが `https://<owner>.github.io/<repo>/` の場合は、ワークフローがリポジトリ名から `BASE_PATH` を設定します。`<owner>.github.io` リポジトリの場合は `/` にします。カスタムドメインは未設定です。ローカルでサブパスを試す場合は次のようにします。

```sh
BASE_PATH=/type-inference-demo/ npm run build
BASE_PATH=/type-inference-demo/ npm run preview
```

このひな形を生成しただけではGitHubリポジトリの作成や公開は行われません。

## 参考

- [wasm-pack: build](https://rustwasm.github.io/docs/wasm-pack/commands/build.html) — `--target web` と生成物
- [Vite: Deploying a Static Site](https://vite.dev/guide/static-deploy.html) — サブパスと静的サイトの公開
- [GitHub Pages: custom workflows](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages) — Pagesへのデプロイ

`tiny-exp`のコードは流用していません。
