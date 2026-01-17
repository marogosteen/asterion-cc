# asterion

Claude Code を使った自律エージェントループ。features.json に定義された機能を順番に実装します。

## インストール

```bash
curl -fsSL https://raw.githubusercontent.com/marogosteen/asterion-cc/main/install.sh | sh
```

デフォルトでは `~/.local/bin` にインストールされます。PATH に追加してください:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

## アップデート

```bash
asterion --update
```

## 使い方

```bash
# 5回のイテレーションを実行
asterion 5

# ドライラン（実行せずに確認）
asterion 5 --dry-run

# 完了時にデスクトップ通知
asterion 5 --notify
```

## 必要なもの

- [Claude Code](https://claude.ai/claude-code) がインストールされていること
- `.asterion/features.json` に実装する機能が定義されていること

## ライセンス

MIT
