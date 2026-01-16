#!/bin/bash
set -euo pipefail

echo "=== asterion 初期化 ==="

# ============================================================
# プロジェクト固有の初期化スクリプト
# 技術スタックに合わせてカスタマイズしてください
# ============================================================

# 例: Rust プロジェクト
# ----------------------
# cargo fmt --check
# cargo clippy -- -D warnings
# cargo test
# cargo build --release

# 例: Node.js プロジェクト
# ------------------------
# npm ci
# npm run lint
# npm test
# npm run build

# 例: Python プロジェクト
# -----------------------
# pip install -r requirements.txt
# ruff check .
# pytest
# python -m build

echo "=== 初期化完了 ==="
echo ""
echo "TODO: このスクリプトをプロジェクトに合わせてカスタマイズしてください"
echo ".asterion/init.sh を編集して、ビルド・テスト・リントコマンドを追加"
