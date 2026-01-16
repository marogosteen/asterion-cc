# {{PROJECT_NAME}}

{{PROJECT_SUMMARY}}

## 技術スタック

- 言語: {{LANGUAGE}}
- フレームワーク: {{FRAMEWORK}}

## コマンド

```bash
# ビルド・テスト・リントの実行
./.asterion/init.sh
```

個別コマンドは `.asterion/init.sh` を参照。

## プロジェクト構造

```
{{PROJECT_NAME}}/
├── CLAUDE.md        # Claude Code 用設定（このファイル）
├── .asterion/
│   ├── features.json    # 機能リスト（asterion が status を管理）
│   ├── progress.md      # セッション間の進捗記録
│   ├── init.sh          # 初期化・検証スクリプト
└── src/
    └── ...
```

## 作業ルール

### asterion から呼ばれた場合

asterion が prompt で実装すべき機能を `iteration_id` と共に指定する。
Claude Code は以下を行う:

1. `./.asterion/init.sh` を実行し、変更前の状態が正常か確認する
2. 指定された機能のみを実装する
3. `./.asterion/init.sh` を再度実行し、ビルド・テスト・リントを通す
4. `.asterion/progress.md` に進捗メモを追記する
5. 変更をコミットする

### 厳守事項

- `.asterion/features.json` を**一切変更しない**（status は asterion が管理）
- 指定された機能以外を実装しない
- テストが通らない状態でコミットしない
- `.asterion/init.sh` が成功する状態を維持する

## コードスタイル

{{CODE_STYLE_NOTES}}

## Git ワークフロー

- コミットメッセージは簡潔に内容を記述
- AI 生成を示すコメントは含めない
