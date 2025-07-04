# x言語 AST直接操作アーキテクチャ設計

## 概要

x言語は従来のテキストベース編集を完全に排除し、AST（抽象構文木）に対する直接操作を基盤とした言語です。これにより、以下を実現します：

- **零遅延型チェック**: ASTレベルでの増分型推論
- **構造保証**: 構文エラーの不存在
- **高速編集**: O(log n)の編集操作
- **AI最適化**: 自然言語からの直接AST生成

## アーキテクチャ概要

```
┌─────────────────────────────────────────────────────────────┐
│                    x-editor (Language Service)             │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │ AST Engine  │  │ Query       │  │ Incremental         │  │
│  │ - Tree Ops  │  │ System      │  │ Type Checker        │  │
│  │ - Indexing  │  │ - Path      │  │ - Salsa Cache       │  │
│  │ - Versioning│  │ - Pattern   │  │ - Constraint Solver │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │ Operations  │  │ Validation  │  │ Change Tracking     │  │
│  │ - Insert    │  │ - Immediate │  │ - Undo/Redo         │  │
│  │ - Delete    │  │ - Semantic  │  │ - History           │  │
│  │ - Replace   │  │ - Type Safe │  │ - Diff              │  │
│  │ - Move      │  └─────────────┘  └─────────────────────┘  │
│  │ - Batch     │                                           │
│  └─────────────┘                                           │
└─────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                 x-parser (AST Foundation)                   │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │ AST Nodes   │  │ Spans       │  │ Symbol Resolution   │  │
│  │ - Immutable │  │ - Source    │  │ - Scopes            │  │
│  │ - Typed     │  │ - Positions │  │ - Bindings          │  │
│  │ - Persistent│  │ - Ranges    │  │ - References        │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────┐
│               x-checker (Type System)                       │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │ Type        │  │ Effect      │  │ Constraint          │  │
│  │ Inference   │  │ System      │  │ Solver              │  │
│  │ - HM        │  │ - Algebraic │  │ - Unification       │  │
│  │ - Gradual   │  │ - Handlers  │  │ - Incremental       │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## 核心設計原則

### 1. 永続化データ構造
AST nodeは完全にimmutableとし、変更は新しいtreeを生成します。これにより：
- Structural sharing による memory 効率化
- Undo/Redo の高速実装
- 並行アクセスの安全性

### 2. インデックス化クエリシステム
全ASTノードに対して複数のインデックスを維持：
- **Type Index**: ノード型による高速検索
- **Symbol Index**: 識別子による参照解決
- **Position Index**: 空間的近接性による検索
- **Dependency Index**: 依存関係グラフ

### 3. 増分型チェック
Salsa frameworkを活用した完全増分型チェック：
- ノード単位の型推論キャッシュ
- 依存関係追跡による最小再計算
- 制約解決の差分更新

### 4. 操作原子性
全ての編集操作は原子的かつ復元可能：
- Transactional な batch 操作
- 操作前の自動バリデーション
- 失敗時の自動ロールバック

## 主要コンポーネント

### AST Engine
```rust
pub struct AstEngine {
    /// 現在のAST（ルートノード）
    current_ast: Arc<CompilationUnit>,
    /// ノード高速検索用インデックス
    indices: IndexCollection,
    /// 変更履歴とバージョン管理
    version_history: VersionHistory,
    /// 型チェッカーへのハンドル
    type_checker: Arc<dyn IncrementalTypeChecker>,
}

impl AstEngine {
    /// 高効率クエリ実行
    pub fn query(&self, query: &AstQuery) -> QueryResult;
    
    /// 原子的操作実行
    pub fn execute_operation(&mut self, op: Operation) -> Result<EditResult>;
    
    /// バッチ操作（複数操作の原子的実行）
    pub fn execute_batch(&mut self, ops: Vec<Operation>) -> Result<BatchResult>;
}
```

### Query System
```rust
pub enum AstQuery {
    // 基本検索
    FindByType { node_type: AstNodeType },
    FindByPattern { pattern: QueryPattern },
    FindReferences { symbol: SymbolId },
    
    // 構造ナビゲーション
    GetParent { node_id: NodeId },
    GetChildren { node_id: NodeId },
    GetSiblings { node_id: NodeId },
    
    // セマンティック検索
    FindDefinition { symbol: SymbolId },
    FindUsages { symbol: SymbolId },
    FindTypeReferences { type_id: TypeId },
    
    // 範囲検索
    NodesInRange { start: Position, end: Position },
    ContainingNode { position: Position },
    
    // 複合クエリ
    And { queries: Vec<AstQuery> },
    Or { queries: Vec<AstQuery> },
    Filter { base: Box<AstQuery>, predicate: Predicate },
}
```

### Operation System
```rust
pub enum Operation {
    // 基本操作
    Insert { parent: NodeId, index: usize, node: AstNode },
    Delete { node: NodeId },
    Replace { node: NodeId, new_node: AstNode },
    Move { node: NodeId, new_parent: NodeId, index: usize },
    
    // 高レベル操作
    Rename { symbol: SymbolId, new_name: String },
    ExtractMethod { range: Range<NodeId>, method_name: String },
    InlineVariable { symbol: SymbolId },
    ChangeSignature { function: NodeId, new_signature: FunctionSignature },
    
    // メタ操作
    Batch { operations: Vec<Operation> },
    Transaction { operations: Vec<Operation> },
}
```

## パフォーマンス特性

### 時間計算量
- **Query実行**: O(log n) (インデックス使用)
- **編集操作**: O(log n) (persistent tree)
- **型チェック**: O(変更部分のみ) (増分処理)
- **参照解決**: O(1) (pre-computed index)

### 空間計算量
- **AST保存**: O(n) with structural sharing
- **インデックス**: O(n) per index type
- **型情報**: O(n) with sharing
- **履歴管理**: O(変更履歴) with compression

## 実装戦略

### Phase 1: 基盤
1. Persistent AST 実装 (im crate活用)
2. 基本インデックスシステム
3. 単純なクエリエンジン
4. 基本編集操作

### Phase 2: 最適化
1. 高度なインデックス戦略
2. 増分型チェック統合
3. バッチ操作サポート
4. メモリ最適化

### Phase 3: 高級機能
1. 複雑なリファクタリング
2. AI支援コード生成
3. リアルタイム協調編集
4. 高度な型推論

この設計により、x言語は従来の言語を遥かに上回る編集効率と開発体験を提供できます。