#!/bin/bash

# Git difftoolsとの統合スクリプト

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "Installing binary-ast-diff as Git difftool..."

# 1. プロジェクトをビルド
echo "Building project..."
cd "$PROJECT_DIR"
npm install
npm run build

# 2. CLIをグローバルに利用可能にする
echo "Making CLI globally available..."
npm link

# 3. Git設定を追加
echo "Configuring Git..."

# カスタムdifftoolを設定
git config --global difftool.binary-ast-diff.cmd "binary-ast-diff git-diff \"\$MERGED\" \"\$LOCAL\" \"\$REMOTE\""

# デフォルトdifftoolの設定（オプション）
read -p "Set binary-ast-diff as default difftool for .sexp files? (y/n): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    git config --global diff.sexp.textconv "binary-ast-diff parse"
    git config --global diff.sexp.tool "binary-ast-diff"
fi

# 4. .gitattributes の例を表示
echo ""
echo "Installation complete!"
echo ""
echo "Usage:"
echo "  git difftool --tool=binary-ast-diff file1.sexp file2.sexp"
echo ""
echo "For automatic detection, add this to your .gitattributes:"
echo "  *.sexp diff=sexp"
echo ""
echo "Examples:"
echo "  git difftool --tool=binary-ast-diff HEAD~1 HEAD -- example.sexp"
echo "  git log --oneline --ext-diff -- '*.sexp'"