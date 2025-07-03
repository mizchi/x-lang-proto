#!/bin/bash

# Git difftoolsとの統合を削除するスクリプト

set -e

echo "Uninstalling binary-ast-diff Git integration..."

# Git設定を削除
echo "Removing Git configuration..."
git config --global --unset difftool.binary-ast-diff.cmd || true
git config --global --unset diff.sexp.textconv || true  
git config --global --unset diff.sexp.tool || true

# グローバルリンクを削除
echo "Removing global npm link..."
npm unlink -g binary-ast-diff || true

echo "Uninstallation complete!"
echo ""
echo "Note: .gitattributes files are not automatically modified."
echo "Remove '*.sexp diff=sexp' lines manually if needed."