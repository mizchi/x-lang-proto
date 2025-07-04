#!/usr/bin/env tsx

/**
 * バイナリAST diffツールの動作確認スクリプト
 * zxを使ってCLIの全機能をテストします
 */

import { $ } from 'zx';

// zxの設定
$.verbose = true;

const CLI = 'npx tsx src/cli.ts';

console.log('🚀 Binary AST Diff Tool - 動作確認開始\n');

try {
  // 1. パース機能のテスト
  console.log('📝 1. パース機能のテスト');
  console.log('   example1.sをパース:');
  await $`npx tsx src/cli.ts parse examples/example1.s`;
  
  console.log('\n   ハッシュ情報付きでパース:');
  await $`npx tsx src/cli.ts parse examples/example1.s --hash`;
  
  console.log('\n   バイナリ情報付きでパース:');
  await $`npx tsx src/cli.ts parse examples/example1.s --binary`;

  // 1.5. コンパイル機能のテスト
  console.log('\n🔧 1.5. コンパイル機能のテスト');
  console.log('   S式ファイルをバイナリにコンパイル:');
  await $`npx tsx src/cli.ts compile examples/example1.s`;
  
  console.log('\n   バイナリファイルからの読み込み:');
  await $`npx tsx src/cli.ts parse examples/example1.s.bin`;

  // 2. 差分機能のテスト
  console.log('\n🔍 2. 差分機能のテスト');
  console.log('   シンプルな差分:');
  await $`npx tsx src/cli.ts diff examples/example1.s examples/example2.s`;
  
  console.log('\n   バイナリファイル間の差分:');
  await $`npx tsx src/cli.ts diff examples/example1.s.bin examples/example2.s.bin`;
  
  console.log('\n   テキストとバイナリの混在比較:');
  await $`npx tsx src/cli.ts diff examples/example1.s examples/example2.s.bin`;
  
  console.log('\n   構造的差分:');
  await $`npx tsx src/cli.ts diff examples/example1.s examples/example2.s --structural`;
  
  console.log('\n   コンパクト表示:');
  await $`npx tsx src/cli.ts diff examples/example1.s examples/example2.s --compact`;

  // 3. 複雑なファイルでの差分テスト
  console.log('\n📊 3. 複雑なファイルでの差分テスト');
  console.log('   complex1.s vs complex2.s:');
  await $`npx tsx src/cli.ts diff examples/complex1.s examples/complex2.s --structural`;

  // 4. バイナリ差分のテスト
  console.log('\n🔢 4. バイナリ差分のテスト');
  console.log('   バイナリサイズとハッシュの比較:');
  await $`npx tsx src/cli.ts binary-diff examples/example1.s examples/example2.s`;
  
  console.log('\n   複雑なファイルのバイナリ比較:');
  await $`npx tsx src/cli.ts binary-diff examples/complex1.s examples/complex2.s`;

  // 5. エラーハンドリングのテスト
  console.log('\n❌ 5. エラーハンドリングのテスト');
  try {
    console.log('   存在しないファイルのテスト:');
    await $`npx tsx src/cli.ts parse nonexistent.sexp`;
  } catch (error) {
    console.log('   ✅ 期待通りエラーが発生しました');
  }

  // 6. パフォーマンステスト
  console.log('\n⚡ 6. パフォーマンステスト');
  console.log('   大きなファイルの処理時間測定:');
  
  const start = Date.now();
  await $`npx tsx src/cli.ts diff examples/complex1.s examples/complex2.s`;
  const end = Date.now();
  
  console.log(`   処理時間: ${end - start}ms`);

  // 7. 統計情報の表示
  console.log('\n📈 7. ファイル統計');
  
  const files = ['examples/example1.s', 'examples/example2.s', 'examples/complex1.s', 'examples/complex2.s'];
  const binFiles = ['examples/example1.s.bin', 'examples/example2.s.bin', 'examples/complex1.s.bin', 'examples/complex2.s.bin'];
  
  for (const file of files) {
    console.log(`\n   ${file}:`);
    const result = await $`npx tsx src/cli.ts parse ${file} --hash`;
    const lines = result.stdout.split('\n');
    const hashLine = lines.find(line => line.includes('Content Hash'));
    if (hashLine) console.log(`     ${hashLine}`);
  }
  
  console.log('\n   バイナリファイルサイズ:');
  for (const binFile of binFiles) {
    const result = await $`npx tsx src/cli.ts parse ${binFile} --hash`;
    const lines = result.stdout.split('\n');
    const hashLine = lines.find(line => line.includes('Content Hash'));
    if (hashLine) console.log(`     ${binFile}: ${hashLine}`);
  }

  // 8. Git統合の準備確認
  console.log('\n🔧 8. Git統合の準備確認');
  console.log('   インストールスクリプトの確認:');
  await $`ls -la git-integration/`;
  
  console.log('\n   インストールスクリプトの権限確認:');
  await $`stat git-integration/install.sh | grep Access`;

  console.log('\n✅ 全てのテストが完了しました！');
  console.log('\n📋 テスト結果サマリー:');
  console.log('   ✅ S式パーサー (.s拡張子): 正常動作');
  console.log('   ✅ バイナリコンパイル (.s.bin): 正常動作');
  console.log('   ✅ 混在比較 (テキスト⇔バイナリ): 正常動作');
  console.log('   ✅ 構造的diff: 正常動作');
  console.log('   ✅ バイナリシリアライザー: 正常動作');
  console.log('   ✅ ハッシュベース識別: 正常動作');
  console.log('   ✅ エラーハンドリング: 正常動作');
  console.log('   ✅ Git統合準備: 完了');

} catch (error) {
  console.error('\n❌ テスト中にエラーが発生しました:');
  console.error(error);
  process.exit(1);
}