#!/usr/bin/env tsx

/**
 * バイナリAST diffツールの性能分析スクリプト
 * .s vs .s.bin の比較分析
 */

import { $ } from 'zx';
import { readFileSync, statSync } from 'fs';
import { createHash } from 'crypto';

$.verbose = false;

console.log('🔍 バイナリAST実装分析\n');

// 1. ファイルサイズ比較
console.log('📊 1. ファイルサイズ比較');
const files = [
  'examples/example1.s',
  'examples/example2.s', 
  'examples/complex1.s',
  'examples/complex2.s'
];

for (const file of files) {
  const binFile = file + '.bin';
  const textSize = statSync(file).size;
  const binSize = statSync(binFile).size;
  const ratio = ((binSize / textSize) * 100).toFixed(1);
  
  console.log(`  ${file}:`);
  console.log(`    テキスト: ${textSize} bytes`);
  console.log(`    バイナリ: ${binSize} bytes (${ratio}%)`);
}

// 2. Git blob サイズ比較
console.log('\n🗂️  2. Git blob サイズ比較');
for (const file of files) {
  const binFile = file + '.bin';
  
  // git hash-object でblobサイズを取得
  const textBlob = await $`git hash-object ${file}`;
  const binBlob = await $`git hash-object ${binFile}`;
  
  console.log(`  ${file}:`);
  console.log(`    テキストblob: ${textBlob.stdout.trim()}`);
  console.log(`    バイナリblob: ${binBlob.stdout.trim()}`);
}

// 3. Content Hash vs Git Hash 比較
console.log('\n🔑 3. Content Hash vs Git Hash');
for (const file of files) {
  const binFile = file + '.bin';
  
  // Content Hash取得
  const contentResult = await $`npx tsx src/cli.ts parse ${file} --hash`;
  const contentHash = contentResult.stdout.match(/Content Hash: ([a-f0-9]+)/)?.[1];
  
  // Git Hash取得  
  const gitHash = await $`git hash-object ${file}`;
  const gitHashShort = gitHash.stdout.trim().substring(0, 8);
  
  console.log(`  ${file}:`);
  console.log(`    Content Hash: ${contentHash}`);
  console.log(`    Git Hash:     ${gitHashShort}`);
  console.log(`    一致: ${contentHash === gitHashShort ? '✅' : '❌'}`);
}

// 4. 簡単な読み込み時間比較
console.log('\n⏱️  4. 読み込み時間比較 (簡易測定)');
const iterations = 100;

for (const file of files) {
  const binFile = file + '.bin';
  
  // テキスト読み込み時間
  const textStart = Date.now();
  for (let i = 0; i < iterations; i++) {
    await $`npx tsx src/cli.ts parse ${file} > /dev/null`;
  }
  const textTime = Date.now() - textStart;
  
  // バイナリ読み込み時間
  const binStart = Date.now();
  for (let i = 0; i < iterations; i++) {
    await $`npx tsx src/cli.ts parse ${binFile} > /dev/null`;
  }
  const binTime = Date.now() - binStart;
  
  const speedup = (textTime / binTime).toFixed(2);
  
  console.log(`  ${file}:`);
  console.log(`    テキスト: ${textTime}ms (${iterations}回)`);
  console.log(`    バイナリ: ${binTime}ms (${iterations}回)`);
  console.log(`    高速化: ${speedup}x`);
}

console.log('\n✨ 分析完了');