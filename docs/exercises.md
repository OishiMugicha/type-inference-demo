# テストの実行方法

全段階のテストを有効化しています。`cargo test --workspace --locked` で言語処理とWasm接続のテストを実行できます。

## 段階を選んで実行

リポジトリのルートから実行してください。

```sh
# lexerのテストをすべて実行
cargo test -p tiny-ml-core --test lexer

# parserのうち、手書きTokenだけを使う1件を実行
cargo test -p tiny-ml-core --test parser literal_from_handwritten_tokens

# parser全体。lexerを使うケースも含む
cargo test -p tiny-ml-core --test parser

# 型推論・評価器。手書きASTとソース入力の両方を検証
cargo test -p tiny-ml-core --test evaluation
cargo test -p tiny-ml-core --test inference

# core全体を実行
cargo test -p tiny-ml-core
```

すべて通常のテストとCIで検証されます。ASTを直接作るテストにより、型推論と評価器はlexer・parserから独立して検証できます。

## テストの内容

| ファイル | 確認すること |
| --- | --- |
| `lexer.rs` | Tokenと位置、予約語の境界、空白、不正文字、整数範囲 |
| `parser.rs` | 手書きToken、優先順位、左結合、単項マイナス、let・関数・if、構文エラー |
| `inference.rs` | 基本型、主型、高階関数、let多相、環境の型変数、無限型、型エラー |
| `evaluation.rs` | 値呼び、評価順、静的スコープ、高階関数、条件分岐、整数エラー |

`support/mod.rs` は手書きASTと型の比較用ヘルパーです。型変数名だけ異なる主型は同じものとして比較します。型推論や単一化の実装はありません。

これらは出発点となる代表例です。実装中に見つけた境界条件を追加して、自分のテストを育ててください。

## Webで確かめる

`npm run dev` を起動し、Rustを変更するたびに `npm run wasm:dev` を実行してブラウザを再読み込みします。

Token・AST・推論された型・評価結果が表示されます。型エラーの場合も評価器は独立して実行されます。

通常のRustエラーは画面に表示されます。処理が戻らない場合は「中止」、panicでWorkerが使えなくなった場合は「再接続」を使ってください。panic後の再接続が難しい場合はページを再読み込みできます。
