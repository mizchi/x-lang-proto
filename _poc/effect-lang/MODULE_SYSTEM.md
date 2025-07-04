# EffectLang モジュールシステム設計

## 設計原則

### 1. **エフェクトファースト**
- エフェクトの可視性と管理を最優先
- モジュール境界でのエフェクト制御
- エフェクトの階層的管理

### 2. **LSP統合重視**
- インクリメンタル解析対応
- モジュール間ナビゲーション
- リアルタイム依存関係表示

### 3. **型安全性**
- モジュール境界での型チェック
- プライベート型の完全隠蔽
- 段階的型付け対応

## モジュール階層

```
project/
├── effect.toml              # プロジェクト設定
├── src/
│   ├── lib.eff             # ライブラリルート
│   ├── main.eff            # 実行可能ファイル
│   ├── core/               # コアモジュール群
│   │   ├── mod.eff         # モジュール定義
│   │   ├── types.eff       # 型定義
│   │   └── effects.eff     # エフェクト定義
│   └── utils/
│       ├── mod.eff
│       ├── string.eff
│       └── io.eff
├── tests/
│   ├── integration.eff
│   └── unit.eff
└── examples/
    └── hello.eff
```

## モジュール構文

### 基本モジュール定義

```effectlang
-- src/core/types.eff
module Core.Types

-- エクスポート宣言（明示的）
export {
  -- 型
  type User, Option, Result,
  -- 値
  value default_user, create_user,
  -- エフェクト
  effect UserStore, Logging
}

-- 型定義
data User = User {
  id: Int,
  name: String,
  email: String
}

data Option[a] = None | Some a
data Result[a, e] = Ok a | Error e

-- エフェクト定義  
effect UserStore {
  find_user : Int -> Option[User]
  save_user : User -> User
  delete_user : Int -> Bool
}

effect Logging {
  log : String -> ()
  debug : String -> ()
  error : String -> ()
}

-- プライベート関数（エクスポートされない）
private validate_email : String -> Bool
private validate_email email = 
  -- 実装...

-- パブリック関数
value default_user : User
value default_user = User { id = 0, name = "", email = "" }

value create_user : String -> String -> Result[User, String] <UserStore, Logging>
value create_user name email = do {
  log ("Creating user: " ++ name);
  if validate_email email then do {
    user <- save_user (User { id = 0, name = name, email = email });
    return (Ok user)
  } else do {
    error ("Invalid email: " ++ email);
    return (Error "Invalid email format")
  }
}
```

### モジュールインポート

```effectlang
-- src/main.eff
module Main

-- 修飾インポート
import Core.Types
import Utils.String as Str
import Utils.IO as IO

-- 選択的インポート
import Core.Types { type User, create_user }
import Utils.String { trim, split }

-- ワイルドカードインポート（推奨されない）
import Core.Types.*

-- エフェクトのみインポート
import Core.Types { effect UserStore, Logging }

main : () -> () <UserStore, Logging, IO.Console>
main () = do {
  -- 修飾名使用
  user <- Core.Types.create_user "Alice" "alice@example.com";
  
  -- エイリアス使用
  message <- Str.format "Created user: {}" [user.name];
  
  -- 選択的インポート使用
  trimmed <- trim message;
  
  -- エフェクトハンドラー
  result <- handle user {
    UserStore.find_user id -> -- ハンドラー実装
    UserStore.save_user u -> -- ハンドラー実装
    -- ...
  };
  
  IO.print_line trimmed
}
```

## エフェクトとモジュールの統合

### エフェクト可視性制御

```effectlang
module Web.Server

-- パブリックエフェクト（他モジュールから使用可能）
export {
  effect HTTP, WebSocket
}

-- プライベートエフェクト（モジュール内のみ）
private effect InternalState {
  get_connection_count : () -> Int
  increment_connections : () -> ()
}

effect HTTP {
  get : String -> Response
  post : String -> RequestBody -> Response  
  listen : Int -> ()
}

effect WebSocket {
  connect : String -> Connection
  send : Connection -> Message -> ()
  receive : Connection -> Message
}

-- エフェクト実装の隠蔽
value start_server : Int -> () <HTTP, InternalState>
value start_server port = do {
  listen port;
  increment_connections ();
  -- 実装詳細...
}

-- 外部から見える型シグネチャ
export value serve : Int -> () <HTTP>
value serve port = 
  handle start_server port {
    InternalState.get_connection_count () -> -- 内部実装
    InternalState.increment_connections () -> -- 内部実装
  }
```

### エフェクト合成

```effectlang
module App.Core

import Web.Server { effect HTTP }
import Database { effect DB }
import Logging { effect Logger }

-- エフェクト組み合わせ
effect AppEffects = HTTP + DB + Logger

-- または型エイリアス
type AppContext[a] = a <AppEffects>

value handle_request : Request -> AppContext[Response]
value handle_request req = do {
  log ("Handling request: " ++ req.path);
  
  data <- DB.query "SELECT * FROM users";
  response <- HTTP.respond_json data;
  
  log "Request completed";
  return response
}
```

## 依存関係管理

### effect.toml プロジェクト設定

```toml
[package]
name = "my-web-app"
version = "0.1.0"
edition = "2024"
authors = ["developer@example.com"]

[dependencies]
core = { path = "../effect-core", version = "1.0" }
web = { git = "https://github.com/effect-lang/web", tag = "v2.1" }
database = { registry = "effect-registry", version = "^3.0" }

[dev-dependencies]
testing = { path = "../effect-testing" }

[features]
default = ["web", "database"]
web = ["dep:web"]
database = ["dep:database"]
async = ["web/async", "database/async"]

[workspace]
members = [
  "core",
  "web", 
  "database",
  "examples/*"
]

[[bin]]
name = "server"
path = "src/main.eff"
required-features = ["web", "database"]

[lsp]
# LSP固有の設定
auto-import = true
effect-inference-hints = true
module-completion = true
```

### モジュール解決アルゴリズム

```rust
// モジュール解決の実装例
pub struct ModuleResolver {
    workspace_root: PathBuf,
    source_dirs: Vec<PathBuf>,
    dependencies: HashMap<String, DependencyInfo>,
}

impl ModuleResolver {
    /// モジュール名からファイルパスを解決
    pub fn resolve_module(&self, module_name: &ModulePath) -> Result<FileId> {
        // 1. ローカルモジュール検索
        if let Some(path) = self.resolve_local_module(module_name)? {
            return Ok(self.file_manager.add_file(path));
        }
        
        // 2. 依存関係モジュール検索  
        if let Some(path) = self.resolve_dependency_module(module_name)? {
            return Ok(self.file_manager.add_file(path));
        }
        
        // 3. 標準ライブラリ検索
        if let Some(path) = self.resolve_stdlib_module(module_name)? {
            return Ok(self.file_manager.add_file(path));
        }
        
        Err(ModuleError::NotFound(module_name.clone()))
    }
    
    /// インクリメンタル依存関係解析
    pub fn analyze_dependencies(&self, file_id: FileId) -> Vec<FileId> {
        // Salsa query: ファイルの依存関係を計算
    }
}
```

## 段階的読み込み

### 遅延モジュール解決

```effectlang
module App

-- 動的インポート（LSPで型チェックは行うが、実行時まで読み込まない）
lazy import Database.Migrations as Migrations
lazy import Features.UserProfile as Profile when feature("user-profiles")

-- 条件付きインポート
import Debug.Tools when debug_mode()
import Production.Optimizations when not debug_mode()

main : () -> () <IO>
main () = do {
  -- 必要時に動的読み込み
  if needs_migration () then do {
    Migrations.run_pending ();
  };
  
  -- フィーチャーフラグによる分岐
  when feature("user-profiles") do {
    Profile.initialize ();
  };
  
  start_application ()
}
```

## LSP統合機能

### モジュール解析とナビゲーション

```rust
// LSPでのモジュール機能実装
impl EffectLanguageServer {
    /// モジュール内シンボル検索
    async fn workspace_symbol(&self, params: WorkspaceSymbolParams) -> Vec<SymbolInformation> {
        let query = params.query;
        let mut symbols = Vec::new();
        
        // 全モジュールから検索
        for module in self.analysis_host.all_modules() {
            for symbol in module.exported_symbols() {
                if symbol.name.contains(&query) {
                    symbols.push(SymbolInformation {
                        name: symbol.name.clone(),
                        kind: symbol.kind.into(),
                        location: symbol.location.to_lsp_location(),
                        container_name: Some(module.name.to_string()),
                        deprecated: Some(symbol.is_deprecated),
                        tags: symbol.tags.clone(),
                    });
                }
            }
        }
        
        symbols
    }
    
    /// モジュール依存関係の可視化
    async fn show_module_dependencies(&self, uri: Url) -> Result<()> {
        let file_id = self.analysis_host.file_id_for_uri(&uri)?;
        let dependencies = self.analysis_host.module_dependencies(file_id);
        
        // カスタム通知でクライアントに送信
        self.client.send_notification::<ModuleDependencyGraph>(
            ModuleDependencyGraph {
                root_module: uri.to_string(),
                dependencies: dependencies.into_iter().map(|dep| {
                    DependencyInfo {
                        module: dep.module_name,
                        path: dep.file_path,
                        effect_usage: dep.used_effects,
                        import_type: dep.import_type,
                    }
                }).collect(),
            }
        ).await;
        
        Ok(())
    }
    
    /// 自動インポート提案
    async fn suggest_imports(&self, symbol: &str, position: Position) -> Vec<CodeAction> {
        let available_modules = self.analysis_host.modules_exporting_symbol(symbol);
        
        available_modules.into_iter().map(|module| {
            CodeAction {
                title: format!("Import {} from {}", symbol, module.name),
                kind: Some(CodeActionKind::QUICKFIX),
                edit: Some(WorkspaceEdit {
                    changes: Some({
                        let mut changes = HashMap::new();
                        changes.insert(
                            position.uri.clone(),
                            vec![TextEdit {
                                range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                                new_text: format!("import {} {{ {} }}\n", module.name, symbol),
                            }]
                        );
                        changes
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }
        }).collect()
    }
}
```

### インラインヒント

```rust
// モジュール情報のインラインヒント
async fn module_inlay_hints(&self, params: InlayHintParams) -> Vec<InlayHint> {
    let mut hints = Vec::new();
    
    // インポートされた関数の元モジュール表示
    for import in self.analysis_host.imports_in_file(file_id) {
        if import.is_unqualified {
            hints.push(InlayHint {
                position: import.usage_position,
                label: InlayHintLabel::String(format!("({})", import.source_module)),
                kind: Some(InlayHintKind::TYPE),
                tooltip: Some(InlayHintTooltip::String(
                    format!("Imported from {}", import.source_module)
                )),
                padding_left: Some(true),
                padding_right: Some(false),
                data: None,
            });
        }
    }
    
    // エフェクトの出所表示
    for effect_usage in self.analysis_host.effect_usages_in_file(file_id) {
        hints.push(InlayHint {
            position: effect_usage.position,
            label: InlayHintLabel::String(format!("<{}>", effect_usage.effect_name)),
            kind: Some(InlayHintKind::TYPE),
            tooltip: Some(InlayHintTooltip::String(
                format!("Effect from {}", effect_usage.defining_module)
            )),
            padding_left: Some(false),
            padding_right: Some(true),
            data: None,
        });
    }
    
    hints
}
```

## モジュール間エフェクト制御

### エフェクト境界の管理

```effectlang
module Secure.Database

-- 機密エフェクト（外部に漏らさない）
private effect DatabaseCredentials {
  get_password : () -> String
  get_connection_string : () -> String
}

-- 公開エフェクト（安全にラップ）
export effect SecureDB {
  query : String -> ResultSet
  transaction : forall a. (() -> a <SecureDB>) -> a
}

-- 内部実装（機密情報を扱う）
private value connect_db : () -> Connection <DatabaseCredentials>
private value connect_db () = do {
  password <- get_password ();
  conn_str <- get_connection_string ();
  -- データベース接続...
}

-- 公開API（機密エフェクトを隠蔽）
export value with_database : forall a. (() -> a <SecureDB>) -> a <FileSystem>
value with_database action = 
  handle action () {
    SecureDB.query sql -> 
      handle connect_db () {
        DatabaseCredentials.get_password () -> read_credential_file "db.password"
        DatabaseCredentials.get_connection_string () -> read_credential_file "db.connection"
      }
    SecureDB.transaction tx -> -- トランザクション実装
  }
```

### エフェクト伝播制御

```effectlang
module Web.API

import Database { effect SecureDB }
import Auth { effect Authentication }
import Logging { effect Logger }

-- エフェクト境界の明示的制御
effect APIEffects = SecureDB + Authentication + Logger

-- エフェクト制限付きハンドラー
value secure_endpoint : forall a. (Request -> a <Authentication>) -> Request -> Response <Logger>
value secure_endpoint handler request = do {
  log ("Processing request: " ++ request.path);
  
  -- Authenticationエフェクトのみ許可
  result <- handle handler request {
    Authentication.verify_token token -> -- 認証実装
    Authentication.get_user_id () -> -- ユーザーID取得
    -- SecureDB, Loggerエフェクトは使用不可（コンパイルエラー）
  };
  
  log "Request processed successfully";
  return (success_response result)
}
```

## 高度な機能

### モジュール型

```effectlang
-- モジュール型の定義
module_type DATABASE = {
  type Connection
  type Query
  
  effect DB {
    connect : String -> Connection
    execute : Connection -> Query -> ResultSet
    close : Connection -> ()
  }
  
  value create_query : String -> Query
}

-- モジュール型の実装
module PostgreSQL : DATABASE = {
  type Connection = PostgresConnection
  type Query = PostgresQuery
  
  effect DB = PostgresDB
  
  value create_query sql = PostgresQuery sql
  -- その他の実装...
}

module SQLite : DATABASE = {
  type Connection = SqliteConnection  
  type Query = SqliteQuery
  
  effect DB = SqliteDB
  
  value create_query sql = SqliteQuery sql
  -- その他の実装...
}

-- 抽象化されたデータベース使用
module App

import type DATABASE

value generic_database_usage : forall db. db : DATABASE => () -> () <db.DB>
value generic_database_usage [db] () = do {
  conn <- db.DB.connect "connection_string";
  query <- db.create_query "SELECT * FROM users";
  result <- db.DB.execute conn query;
  db.DB.close conn;
  -- 結果処理...
}

-- 具体的なデータベース選択
import PostgreSQL as DB

main = generic_database_usage [DB] ()
```

### プラグインシステム

```effectlang
-- プラグイン定義
module_type PLUGIN = {
  type Config
  
  effect Plugin {
    initialize : Config -> ()
    process : Request -> Response
    cleanup : () -> ()
  }
  
  value default_config : Config
}

-- 動的プラグイン読み込み
module PluginManager

effect PluginRegistry {
  register : forall p. p : PLUGIN => String -> p -> ()
  load : String -> Request -> Response  
  unload : String -> ()
}

value load_plugins : () -> () <PluginRegistry, FileSystem>
value load_plugins () = do {
  plugin_dir <- list_directory "plugins/";
  for plugin_file in plugin_dir do {
    plugin <- dynamic_load plugin_file;
    register (plugin_name plugin_file) plugin;
  }
}
```

この設計により、**エフェクトシステムと密接に統合された安全で表現豊かなモジュールシステム**が実現され、**LSPによる優れた開発体験**が提供されます。