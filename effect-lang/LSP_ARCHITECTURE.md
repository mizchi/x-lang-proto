# LSP-First Architecture for EffectLang

## 設計思想

**エディタファースト開発体験**を重視し、LSPを通じて高度な言語機能を提供する。コマンドラインツールは二次的な位置づけ。

## アーキテクチャ概要

### 核となるコンポーネント

```
effect-lang/
├── src/
│   ├── bin/
│   │   ├── lsp.rs              # LSPサーバーエントリーポイント
│   │   └── cli.rs              # CLIツール（LSPサーバー機能のラッパー）
│   ├── lsp/                    # LSP実装
│   │   ├── server.rs           # Tower-LSPサーバー
│   │   ├── handlers/           # LSPハンドラー群
│   │   ├── diagnostics.rs      # 診断・エラー報告
│   │   └── capabilities.rs     # サーバー機能定義
│   ├── analysis/               # 言語解析エンジン
│   │   ├── database.rs         # Salsa incremental DB
│   │   ├── parser.rs           # インクリメンタルパーサー
│   │   ├── syntax/             # 構文解析
│   │   ├── types/              # 型システム
│   │   └── effects/            # エフェクトシステム
│   ├── core/                   # 言語コア
│   │   ├── ast.rs              # 抽象構文木
│   │   ├── span.rs             # ソース位置情報
│   │   └── symbol.rs           # シンボルテーブル
│   └── lib.rs
└── tree-sitter-effect/         # Tree-sitter grammar
```

## LSP機能マトリックス

### 基本機能
| 機能 | 実装状況 | 優先度 | 説明 |
|------|----------|-------|------|
| **Syntax Highlighting** | 🟡 計画中 | P0 | Tree-sitterベース |
| **Diagnostics** | 🟡 計画中 | P0 | リアルタイムエラー・警告 |
| **Hover Information** | 🟡 計画中 | P0 | 型情報・エフェクト情報 |
| **Go to Definition** | 🟡 計画中 | P0 | シンボル定義ジャンプ |
| **Find References** | 🟡 計画中 | P1 | 参照検索 |
| **Auto-completion** | 🟡 計画中 | P0 | インテリジェント補完 |

### 高度な機能
| 機能 | 実装状況 | 優先度 | 説明 |
|------|----------|-------|------|
| **Effect Inference Display** | 🟡 計画中 | P0 | エフェクト推論結果表示 |
| **Effect Handler Suggestions** | 🟡 計画中 | P1 | ハンドラー自動提案 |
| **Type Hole Completion** | 🟡 計画中 | P1 | `?`による型駆動開発 |
| **Refactoring** | 🟡 計画中 | P2 | 安全なリファクタリング |
| **Effect Visualization** | 🟡 計画中 | P2 | エフェクトフローの可視化 |
| **Live Error Squigles** | 🟡 計画中 | P0 | リアルタイム型エラー |

## インクリメンタル解析設計

### Salsa Database Schema

```rust
// 入力クエリ（ファイルシステムから）
#[salsa::input]
struct SourceFile {
    #[return_ref]
    path: PathBuf,
    #[return_ref] 
    content: String,
}

// 派生クエリ（解析結果）
#[salsa::tracked]
fn parse_file(db: &dyn Db, file: SourceFile) -> ParseResult {
    // インクリメンタルパース
}

#[salsa::tracked]
fn type_check_file(db: &dyn Db, file: SourceFile) -> TypeCheckResult {
    // 型チェック（パース結果に依存）
}

#[salsa::tracked]
fn infer_effects(db: &dyn Db, file: SourceFile) -> EffectInferenceResult {
    // エフェクト推論（型チェック結果に依存）
}
```

### Red-Green Tree（Rowan）

```rust
// 構文木ノード定義
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum SyntaxKind {
    // リテラル
    INTEGER, STRING, BOOL,
    // 識別子
    IDENT,
    // 式
    LET_EXPR, LAMBDA_EXPR, APP_EXPR, MATCH_EXPR,
    // 型
    TYPE_ANNO, EFFECT_ANNO,
    // その他
    ERROR, WHITESPACE, COMMENT,
}

// CST（Concrete Syntax Tree）から
// AST（Abstract Syntax Tree）への変換を効率的に実現
```

## LSPサーバー実装

### Tower-LSP ベースサーバー

```rust
#[tower_lsp::async_trait]
impl LanguageServer for EffectLanguageServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        // クライアント機能の確認
        // サーバー機能の宣言
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions::default()),
                definition_provider: Some(OneOf::Left(true)),
                // ... その他の機能
            },
            server_info: Some(ServerInfo {
                name: "effect-lang-lsp".to_string(),
                version: Some("0.1.0".to_string()),
            }),
        })
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        // インクリメンタル更新
        // 再解析のトリガー
        // 診断の更新
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        // カーソル位置の型・エフェクト情報を取得
        // マークダウン形式で返却
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        // コンテキスト考慮した補完候補
        // エフェクトハンドラーの提案
        // 型駆動補完
    }
}
```

## エラー報告戦略

### 段階的エラー報告

1. **構文エラー** (即座)
   ```
   Error: Expected '=' after let binding
    --> example.eff:3:10
    |
   3 | let x : Int 42
    |          ^ Expected '='
   ```

2. **型エラー** (型チェック後)
   ```
   Error: Type mismatch
    --> example.eff:5:8
    |
   5 |   add "hello" 42
    |       ^^^^^^^ Expected Int, found String
    |
   = Note: Function 'add' expects: Int -> Int -> Int
   ```

3. **エフェクトエラー** (エフェクト推論後)
   ```
   Error: Unhandled effect
    --> example.eff:8:12
    |
   8 |     print "Hello"
    |     ^^^^^ Effect 'IO' is not handled
    |
   = Help: Consider adding an IO handler or declare function as <IO>
   ```

### インテリジェントエラー回復

```rust
// パーサーエラー回復戦略
impl Parser {
    fn recover_from_error(&mut self, expected: &[TokenKind]) -> Option<SyntaxNode> {
        // 1. 同期トークンまでスキップ
        // 2. エラーノードとして部分的にパース
        // 3. 可能な限り後続の解析を継続
    }
}
```

## リアルタイム機能

### 型情報のホバー表示

```rust
async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
    let position = params.text_document_position_params.position;
    let uri = params.text_document_position_params.text_document.uri;
    
    let analysis = self.analysis_host.snapshot();
    let file_id = analysis.file_id_for_uri(&uri)?;
    
    if let Some(info) = analysis.hover_info(file_id, position)? {
        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!(
                    "```effectlang\n{}\n```\n\n{}",
                    info.type_signature,
                    info.documentation.unwrap_or_default()
                ),
            }),
            range: Some(info.range),
        }))
    } else {
        Ok(None)
    }
}
```

### エフェクト推論の可視化

```rust
// エフェクト情報をInlay Hintsとして表示
async fn inlay_hint(&self, params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
    let uri = params.text_document.uri;
    let analysis = self.analysis_host.snapshot();
    let file_id = analysis.file_id_for_uri(&uri)?;
    
    let mut hints = Vec::new();
    
    // 関数のエフェクト推論結果をヒントとして表示
    for function in analysis.functions_in_file(file_id)? {
        if let Some(inferred_effects) = analysis.inferred_effects(function)? {
            if !inferred_effects.is_empty() {
                hints.push(InlayHint {
                    position: function.name_range().end,
                    label: InlayHintLabel::String(format!(" <{}>", inferred_effects)),
                    kind: Some(InlayHintKind::TYPE),
                    tooltip: Some(InlayHintTooltip::String(
                        "Inferred effects".to_string()
                    )),
                    padding_left: Some(true),
                    padding_right: Some(false),
                    data: None,
                });
            }
        }
    }
    
    Ok(Some(hints))
}
```

## パフォーマンス最適化

### キャッシュ戦略

```rust
// Salsa による自動キャッシュ
#[salsa::tracked]
fn expensive_analysis(db: &dyn Db, file: SourceFile) -> AnalysisResult {
    // 高コストな解析
    // 入力が変わらない限り結果がキャッシュされる
}

// 手動キャッシュが必要な場合
struct AnalysisHost {
    symbol_cache: DashMap<FileId, Arc<SymbolTable>>,
    type_cache: DashMap<(FileId, Position), Arc<TypeInfo>>,
}
```

### 並行処理

```rust
// 複数ファイルの並行解析
async fn analyze_workspace(&self, workspace: &Workspace) -> Result<()> {
    let files: Vec<_> = workspace.all_files().collect();
    
    // ファイル間依存関係を考慮した並行処理
    let results = stream::iter(files)
        .map(|file| self.analyze_file(file))
        .buffer_unordered(num_cpus::get())
        .collect::<Vec<_>>()
        .await;
    
    // 結果をマージして全体解析を完了
    self.merge_analysis_results(results).await
}
```

## デバッグ・観測可能性

### 構造化ログ

```rust
use tracing::{info, warn, error, span, Level};

#[tracing::instrument(skip(self))]
async fn type_check_file(&self, file_id: FileId) -> Result<TypeCheckResult> {
    let _span = span!(Level::INFO, "type_check", file = %file_id).entered();
    
    info!("Starting type check");
    let result = self.perform_type_check(file_id).await?;
    info!(functions = result.functions.len(), "Type check completed");
    
    Ok(result)
}
```

### LSPクライアント向けカスタム通知

```rust
// カスタム通知でエフェクト解析進捗を報告
#[derive(Serialize)]
struct EffectAnalysisProgress {
    file: String,
    functions_analyzed: usize,
    total_functions: usize,
    effects_inferred: Vec<String>,
}

// クライアントに通知送信
self.client.send_notification::<custom::EffectAnalysisProgress>(
    EffectAnalysisProgress {
        file: file_path.to_string(),
        functions_analyzed: 42,
        total_functions: 100,
        effects_inferred: vec!["IO".to_string(), "State[Int]".to_string()],
    }
).await?;
```

## エディタ統合戦略

### VS Code Extension

```typescript
// VS Code拡張の主要機能
class EffectLangExtension {
    // エフェクトの可視化
    private registerEffectDecorations() {
        // エフェクト推論結果をハイライト表示
    }
    
    // 型ホールの補完
    private registerTypeHoleCommands() {
        // `?` での型駆動開発サポート
    }
    
    // エフェクトハンドラー自動生成
    private registerEffectHandlerCodeActions() {
        // 未処理エフェクトの自動ハンドラー生成
    }
}
```

### Neovim/Helix対応

```lua
-- Neovim設定例
require('lspconfig').effect_lang.setup({
    cmd = { 'effect-lsp' },
    filetypes = { 'effect', 'eff' },
    root_dir = require('lspconfig.util').root_pattern('effect.toml', '.git'),
    settings = {
        effectLang = {
            enableInlayHints = true,
            enableEffectVisualization = true,
        }
    }
})
```

この設計により、**エディタファーストの開発体験**と**高度な言語機能**を両立する実装を実現します。