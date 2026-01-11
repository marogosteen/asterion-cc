# Project Name

Project Summary

## プロジェクト概要

Project Name の開発を行う。
Anthropic のブログ記事「Effective harnesses for long-running agents」（./docs/long-running-agents-guide.md）のパターンを使用して実装する。

## 技術スタック

## コマンド例

```bash
# ビルド
cargo build

# テスト
cargo test

# リンター
cargo clippy -- -D warnings

# フォーマット
cargo fmt

# 初期化スクリプト
./init.sh
```

## プロジェクト構造

```
project-name/
├── features.json    # 機能リスト（status: pending/in_progress/completed で進捗管理）
├── progress.md      # セッション間の進捗記録
├── init.sh          # 初期化・検証スクリプト
└── CLAUDE.md        # Claude Code 用設定（このファイル）
```

## 作業ルール

### セッション開始時

1. `pwd` で作業ディレクトリを確認する
2. `git log --oneline -10` で最近のコミットを確認する
3. `progress.md` を読み、前回の作業内容を把握する
4. `features.json` を読み、未完了（`status: "pending"`）の機能を確認する
5. `./init.sh` を実行して、ビルドと E2E 検証が通ることを確認する
6. **1つの機能だけ**に集中する
7. 実装を開始する機能の `status` を `"in_progress"` に更新する

### セッション終了時

1. 完了した機能の `status` を `"completed"` に更新する
2. `progress.md` に今回の作業内容を追記する
3. Git コミットする（1機能 = 1コミット）
4. 次回やることを `progress.md` に記載する

### 厳守事項

- features.json の機能を**削除・編集しない**（status フィールドのみ変更可）
- 一度に複数の機能を実装しない
- テストが通らない状態でコミットしない
- `init.sh` が成功する状態を維持する

## コードスタイル例

- `cargo fmt` に従う
- `cargo clippy -- -D warnings` を通す
- clippy::pedantic, clippy::nursery は有効化済み

## Git ワークフロー

- コミットメッセージは簡潔に内容を記述
- AI 生成を示すコメントは含めない
