# Tiny ML — 型推論の実験室

『プログラミング言語の基礎概念』を読んだ後に、lexerから型推論・評価器までを学べるRust製の小さなML処理系です。

lexer・parser・let多相を持つ型推論・値呼びの評価器を実装しています。Web画面から式を入力し、Token・AST・型・値を確認できます。

## 起動する

Rust stable（rustup）、Node.js 24、npm、wasm-pack 0.15.0を使用します。

```sh
rustup show
cargo install wasm-pack --version 0.15.0 --locked
npm ci
npm run dev
```

表示されたローカルURLを開き、「実行する」を押してください。Token・AST・型・値の各欄に結果が表示されます。

`rust-toolchain.toml` にWasmターゲット・rustfmt・Clippyを指定しています。初回の `rustup show` で必要なコンポーネントが取得されます。

## 実装の入口

| 入口 | ファイル | 入出力 |
| --- | --- | --- |
| `lex` | `crates/core/src/lexer.rs` | `&str → LangResult<Vec<Token>>` |
| `parse` | `crates/core/src/parser.rs` | `&[Token] → LangResult<Expr>` |
| `infer` | `crates/core/src/infer.rs` | `&Expr → LangResult<Type>` |
| `eval` | `crates/core/src/eval.rs` | `&Expr → LangResult<Value>` |

[言語仕様](docs/language.md)に対応した実装とテストを用意しています。

1. lexerで文字列を位置付きのToken列に変換する。
2. 再帰下降parserで演算子の優先順位を反映したASTを構築する。
3. 型推論で単一化・出現検査・let束縛の一般化を行い、主型を求める。
4. 評価器で定義時の環境を捕捉するクロージャと、範囲検査付きの整数演算を評価する。

型推論と評価器は独立して動作します。[テストの実行方法](docs/exercises.md)には、段階ごとのコマンドをまとめています。

## 編集と検証

```sh
# Rustを編集したら、別ターミナルでWasmを再ビルド
npm run wasm:dev
# 完了後、ブラウザを再読み込み

# Rust全体の整形・静的解析・テスト
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
crates/core/       言語のデータ型・lexer・parser・型推論・評価器・テスト
crates/wasm/       Rustから表示用JSONへの変換と段階間の接続
web/src/           TypeScriptの画面とWeb Worker
docs/              言語仕様とテストの実行方法
tests/             Playwrightのブラウザ検証
.github/workflows/ 検証とGitHub Pages公開
```

Wasmの `analyze(source)` はJSON文字列を返します。`tokens`・`ast`・`inferred_type`・`value` の各フィールドが、`success`（`output`）か `unimplemented` / `error` / `skipped`（`message`）を持ちます。対応するTypeScript型は `web/src/protocol.ts` にあります。

Token・ASTはRustのDebug形式、型・値はDisplay形式の文字列です。`i64`をJavaScriptの数値に変換せず、精度を保ちます。型変数は `'t0` のように表示し、クロージャの値は `<fun>` と表示します。画面では出力をテキストとして扱います。

lexer成功後にparserが動き、parser成功後に型推論・評価器がそれぞれ動きます。型エラーでも評価器は実行します。型エラーの例で型と値の両方にエラーが出ることがあります。先に成功した結果は、後段の通常エラーで失われません。

処理はWorker内で実行します。「中止」はWorkerを終了し、新しいWasmを読み込みます。Rustのpanicや読み込み失敗の場合は「再接続」でやり直せます。1回の要求につき全段階をまとめて返すため、無限ループ中の途中結果は表示しません。coreの通常エラーには `Diagnostic` を使ってください。

## GitHub Pagesに公開する

1. このプロジェクトをGitHubのリポジトリへ追加する。`Cargo.lock` と `package-lock.json` もコミットする。
2. リポジトリの **Settings → Pages → Build and deployment → Source** を **GitHub Actions** にする。
3. `main` にpushする。Actionsの「Check and deploy Pages」が検証・ビルド・公開する。

PRでは検証だけを実行します。手動実行は `main` を選ぶと公開まで進みます。生成物の `web/pkg`・`web/dist` はコミット不要です。

公開URLが `https://<owner>.github.io/<repo>/` の場合は、ワークフローがリポジトリ名から `BASE_PATH` を設定します。`<owner>.github.io` リポジトリの場合は `/` にします。カスタムドメインは未設定です。ローカルでサブパスを試す場合は次のようにします。

```sh
BASE_PATH=/type-inference-demo/ npm run build
BASE_PATH=/type-inference-demo/ npm run preview
```

ローカルで実装しただけではGitHubリポジトリの作成や公開は行われません。

## 参考

- [wasm-pack: build](https://rustwasm.github.io/docs/wasm-pack/commands/build.html) — `--target web` と生成物
- [Vite: Deploying a Static Site](https://vite.dev/guide/static-deploy.html) — サブパスと静的サイトの公開
- [GitHub Pages: custom workflows](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages) — Pagesへのデプロイ

`tiny-exp`のコードは流用していません。
