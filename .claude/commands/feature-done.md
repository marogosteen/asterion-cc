---
description: 機能完了マーク - テスト確認後、status を completed に更新してコミット
allowed-tools: Read, Write, Edit, Bash(cat:*), Bash(git status), Bash(git log:*), Bash(git add:*), Bash(git commit:*), Bash(npm:*), Bash(yarn:*), Bash(pnpm:*), Bash(cargo:*), Bash(python:*), Bash(./init.sh)
---

# Feature Done - 機能完了マーク

実装した機能を完了としてマークし、進捗を記録します。

## 前提条件

- `/feature-next` で機能を実装済みであること
- テストが成功していること

## 実行手順

### 1. テストの最終確認

完了マークをつける前に、もう一度テストを実行してください。

- `features.json` の該当機能の steps を確認
- 全ての手順が成功することを確認

**テストが失敗した場合は、完了マークをつけずに問題を修正してください。**

### 2. features.json の更新

完了した機能の `status` を `"completed"` に変更してください。

変更前：
```json
{
  "category": "functional",
  "description": "機能の説明",
  "steps": ["..."],
  "status": "in_progress"
}
```

変更後：
```json
{
  "category": "functional",
  "description": "機能の説明",
  "steps": ["..."],
  "status": "completed"
}
```

### 3. claude-progress.txt の更新

今回の作業内容を追記してください：

```
## YYYY-MM-DD セッション

### 完了した機能
- [機能の description]

### 発生した問題
- （問題があれば記載、なければ「特になし」）

### 次回やること
- 次の未完了機能の実装

### 注意点
- （次のセッションで注意すべき点があれば記載）
```

### 4. Git コミット

変更をコミットしてください。

**コミット規約：**
- AI 生成を示すコメントは含めない（Co-Authored-By: Claude 等は禁止）
- コミットメッセージは簡潔に内容を記述

```bash
git add .
git commit -m "feat: [機能の説明]"
```

コミットには以下を含めます：
- 実装したコードの変更
- features.json（status: completed に更新）
- claude-progress.txt（進捗を追記）

### 5. 完了報告

ユーザーに完了を報告してください：

```
機能を完了としてマークしました:
- description: [機能の説明]
- コミット: [コミットハッシュ]

進捗状況:
- 完了: X / Y 機能
- 次の機能: [次の未完了機能の description]
```

## 注意事項

- **テストが通る状態でのみ**完了マークをつける
- 部分的な実装で完了としない
- 問題が発生した場合は `claude-progress.txt` に詳細を記録する
- 次のセッションで分かるように、注意点を残す
