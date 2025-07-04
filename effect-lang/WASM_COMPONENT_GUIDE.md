# WebAssembly Component Model Support

EffectLangでWebAssembly Component Modelコンパイルが利用できるようになりました。

## 新機能

### 1. WITジェネレーター
WebAssembly Interface Types (WIT) ファイルを生成します。

```bash
cargo run --bin compiler -- compile examples/component_demo.eff --target wit
```

生成されるファイル:
- `component_demo.wit` - WITインターフェース定義
- `Cargo.toml` - Rustプロジェクト設定

### 2. WebAssembly Component コンパイラ
完全なWebAssembly Componentを生成します。

```bash
cargo run --bin compiler -- compile examples/component_demo.eff --target wasm-component
```

生成されるファイル:
- `component_demo.wit` - WITインターフェース定義
- `src/lib.rs` - Rustソースコード (wit-bindgen対応)
- `Cargo.toml` - Rustプロジェクト設定
- `build.rs` - ビルドスクリプト

## 対応している機能

### インターフェース定義
```effect-lang
pub interface "wasi:filesystem@0.2.0"
  func open(path: i32) -> i32
  func read(fd: i32, size: i32) -> i32
  
  resource file
    method constructor create(path: i32) -> i32
    method get-size(fd: i32) -> i64
```

### Visibility修飾子
```effect-lang
pub let public_function = ...
pub(crate) let crate_function = ...
pub(package) let package_function = ...
pub(super) let super_function = ...
pub(in path::to::module) let scoped_function = ...
```

### Component Model Visibility
```effect-lang
pub(component export interface "my:api") let exported_api = ...
pub(component import interface "wasi:filesystem") let imported_wasi = ...
```

### Pipeline演算子
```effect-lang
let result = input
  |> transform1
  |> transform2
  |> handle_effect
```

### エフェクトシステム統合
```effect-lang
effect IO[a] = { read: String -> a, write: String -> a -> Unit }

let main = handle (
  do {
    let content <- IO.read "file.txt";
    IO.write "output.txt" content;
    return content
  }
) with {
  IO.read path k -> k (wasi_read path);
  IO.write path content k -> k (wasi_write path content);
  return value -> value
}
```

## コンパイル手順

### 1. WITファイルのみ生成
```bash
# WITファイルを生成
cargo run --bin compiler -- compile examples/component_demo.eff --target wit --output ./wit_output

# 生成されたWITを検証 (wasm-toolsが必要)
wasm-tools component wit ./wit_output/component_demo.wit
```

### 2. 完全なWebAssembly Component生成
```bash
# Rustプロジェクトを生成
cargo run --bin compiler -- compile examples/component_demo.eff --target wasm-component --output ./component_output

# 生成されたプロジェクトをビルド
cd ./component_output
cargo build --target wasm32-wasi

# Component化 (cargo-componentが必要)
cargo component build --target wasm32-wasi
```

### 3. 必要なツール

#### wasm-tools
```bash
cargo install wasm-tools
```

#### cargo-component
```bash
cargo install cargo-component
```

#### wit-bindgen
```bash
cargo install wit-bindgen-cli
```

## 使用例

### WASI filesystem interface
```effect-lang
// examples/wasm_interface.*.eff を参照
import (interface "wasi:filesystem@0.2.0") open read write close

pub let read_file = fun path ->
  path
  |> open
  |> fun fd -> (fd, read fd 1024)
  |> fun (fd, content) -> (close fd; content)
```

### カスタムコンポーネントインターフェース
```effect-lang
// カスタム計算インターフェース
pub interface "compute:math@1.0.0"
  func add(x: f64, y: f64) -> f64
  func multiply(x: f64, y: f64) -> f64
  
  resource matrix
    method constructor new(rows: i32, cols: i32)
    method multiply(other: matrix) -> matrix

// インターフェースの実装
pub let math_impl = {
  add = fun x y -> x + y;
  multiply = fun x y -> x * y;
  matrix_new = fun rows cols -> native_matrix_new rows cols;
  matrix_multiply = fun m1 m2 -> native_matrix_multiply m1 m2;
}
```

## コンパイラオプション

### 利用可能なターゲット
```bash
cargo run --bin compiler -- targets
```

出力:
```
Available compilation targets:

TypeScript (typescript, ts):
  - Generates type-safe TypeScript code
  - Supports all EffectLang features via async/await

WebAssembly GC (wasm-gc, wasm):
  - Generates WebAssembly GC bytecode
  - Efficient functional programming with GC

WebAssembly Component (wasm-component, component):
  - Generates WebAssembly Component Model compliant code
  - Full support for interfaces, resources, and imports/exports
  - Effect system integration with WASI
  - Generates Rust source code for wit-bindgen

WIT (wit):
  - Generates WebAssembly Interface Types definitions
  - Language-agnostic interface specifications
  - Compatible with wasm-tools and wit-bindgen
  - Export EffectLang interfaces to other languages
```

### コンパイルオプション
```bash
# デバッグ情報付き
cargo run --bin compiler -- compile input.eff --target wasm-component --debug

# 最適化レベル指定
cargo run --bin compiler -- compile input.eff --target wasm-component --optimization speed

# ソースマップ生成
cargo run --bin compiler -- compile input.eff --target wasm-component --source-maps
```

## テスト

### 単体テスト
```bash
cargo test wasm_component_tests
```

### 統合テスト
```bash
# WITファイル生成テスト
cargo test test_simple_wit_generation

# Component生成テスト
cargo test test_component_rust_generation

# インターフェース生成テスト
cargo test test_interface_wit_generation

# リソース生成テスト
cargo test test_resource_generation
```

## トラブルシューティング

### よくある問題

1. **wasm-toolsが見つからない**
   ```bash
   cargo install wasm-tools
   ```

2. **cargo-componentが見つからない**
   ```bash
   cargo install cargo-component
   ```

3. **WITファイルの構文エラー**
   - 生成されたWITファイルを `wasm-tools component wit` で検証
   - EffectLang構文とWIT構文の対応を確認

4. **Component Model機能が使えない**
   - WASM ランタイムがComponent Modelをサポートしているか確認
   - wasmtime 8.0以上など、Component Model対応ランタイムを使用

### デバッグ

詳細なログ出力:
```bash
cargo run --bin compiler -- compile input.eff --target wasm-component --verbose
```

中間ファイルの確認:
```bash
# 生成されたRustコードを確認
cat ./component_output/src/lib.rs

# 生成されたWITファイルを確認
cat ./component_output/component_demo.wit

# 生成されたCargo.tomlを確認
cat ./component_output/Cargo.toml
```

## 今後の予定

- [ ] エフェクトハンドラーの最適化
- [ ] より高度なWASIインターフェースサポート
- [ ] Component Model 最新仕様への対応
- [ ] パフォーマンスチューニング
- [ ] ドキュメント生成機能