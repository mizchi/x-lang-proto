#!/usr/bin/env tsx

/**
 * Git compaction効率のテスト
 */

import { $ } from 'zx';
import { writeFileSync } from 'fs';

$.verbose = false;

console.log('🗜️  Git Compaction効率分析\n');

// 1. 類似ファイルを大量作成してcompaction効率をテスト
console.log('📝 1. 類似ファイルセット作成');

const baseS = `(defun factorial (n)
  (if (= n 0)
      1
      (* n (factorial (- n 1)))))`;

// 微小な変更を加えた複数のファイルを作成
const variations = [
  baseS,
  baseS.replace('= n 0', '= n 1'),
  baseS.replace('= n 0', '<= n 1'),
  baseS.replace('factorial', 'fact'),
  baseS.replace('* n', '* n n'),
];

// テキスト版とバイナリ版を作成
for (let i = 0; i < variations.length; i++) {
  const textFile = `temp/var${i}.s`;
  const binFile = `temp/var${i}.s.bin`;
  
  // ディレクトリ作成
  await $`mkdir -p temp`;
  
  // テキストファイル作成
  writeFileSync(textFile, variations[i]);
  
  // バイナリファイル作成
  await $`npx tsx src/cli.ts compile ${textFile} ${binFile}`;
  
  console.log(`  作成: ${textFile} -> ${binFile}`);
}

// 2. Git repoを作成してcompaction効率をテスト
console.log('\n🗂️  2. Git compaction効率テスト');

// 一時的なgitリポジトリを作成
await $`rm -rf temp-git-test`;
await $`mkdir temp-git-test`;
await $`cd temp-git-test && git init`;

// テキストファイルを追加
console.log('\n📄 テキストファイル (.s) の場合:');
for (let i = 0; i < variations.length; i++) {
  const content = variations[i];
  writeFileSync(`temp-git-test/file${i}.s`, content);
  await $`cd temp-git-test && git add file${i}.s && git commit -m "Add file${i}.s"`;
}

// pack前のサイズ
const beforePack = await $`cd temp-git-test && du -s .git/objects`;
console.log(`  pack前: ${beforePack.stdout.trim()}`);

// git gc実行
await $`cd temp-git-test && git gc --aggressive`;

// pack後のサイズ
const afterPack = await $`cd temp-git-test && du -s .git/objects`;
console.log(`  pack後: ${afterPack.stdout.trim()}`);

// バイナリファイルでも同様のテスト
console.log('\n🔢 バイナリファイル (.s.bin) の場合:');
await $`rm -rf temp-git-test`;
await $`mkdir temp-git-test`;
await $`cd temp-git-test && git init`;

for (let i = 0; i < variations.length; i++) {
  const textFile = `temp/var${i}.s`;
  const binFile = `temp/var${i}.s.bin`;
  
  // バイナリファイルをgitリポジトリにコピー
  await $`cp ${binFile} temp-git-test/file${i}.s.bin`;
  await $`cd temp-git-test && git add file${i}.s.bin && git commit -m "Add file${i}.s.bin"`;
}

// pack前のサイズ
const beforePackBin = await $`cd temp-git-test && du -s .git/objects`;
console.log(`  pack前: ${beforePackBin.stdout.trim()}`);

// git gc実行
await $`cd temp-git-test && git gc --aggressive`;

// pack後のサイズ
const afterPackBin = await $`cd temp-git-test && du -s .git/objects`;
console.log(`  pack後: ${afterPackBin.stdout.trim()}`);

// 3. 個別ファイルのgit blob情報
console.log('\n🔍 3. 個別ファイルの圧縮効率');
for (let i = 0; i < 3; i++) {
  const textFile = `temp/var${i}.s`;
  const binFile = `temp/var${i}.s.bin`;
  
  // ファイルサイズ
  const textSize = await $`stat -c%s ${textFile}`;
  const binSize = await $`stat -c%s ${binFile}`;
  
  console.log(`  ファイル${i}:`);
  console.log(`    .s    : ${textSize.stdout.trim()} bytes`);
  console.log(`    .s.bin: ${binSize.stdout.trim()} bytes`);
}

// クリーンアップ
await $`rm -rf temp temp-git-test`;

console.log('\n✨ Git compaction分析完了');