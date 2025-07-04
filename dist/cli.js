#!/usr/bin/env node
/**
 * バイナリAST diff CLI
 */
import { Command } from 'commander';
import { readFileSync, writeFileSync } from 'fs';
import { extname } from 'path';
import { SExpParser } from './sexp.js';
import { BinarySerializer, calculateHash, generateContentHash } from './serializer.js';
import { StructuralDiff } from './diff.js';
import { DiffRenderer } from './renderer.js';
const program = new Command();
program
    .name('binary-ast-diff')
    .description('Binary AST diff tool with S-expression syntax')
    .version('0.1.0');
program
    .command('parse')
    .description('Parse S-expression file and show AST')
    .argument('<file>', 'S-expression file to parse')
    .option('--binary', 'Output binary representation')
    .option('--hash', 'Show content hash')
    .action(async (file, options) => {
    try {
        const ext = extname(file);
        let sexp;
        if (ext === '.bin' || file.endsWith('.s.bin')) {
            // バイナリファイルから読み込み
            const binaryData = readFileSync(file);
            const serializer = new BinarySerializer();
            sexp = serializer.deserialize(new Uint8Array(binaryData));
            console.log('Loaded from binary format');
        }
        else {
            // テキストファイルから読み込み
            const content = readFileSync(file, 'utf-8');
            const parser = new SExpParser();
            sexp = parser.parse(content);
        }
        console.log('Parsed AST:');
        console.log(JSON.stringify(sexp, null, 2));
        if (options.hash) {
            const hash = calculateHash(sexp);
            const contentHash = generateContentHash(sexp);
            console.log(`\\nContent Hash: ${contentHash}`);
            console.log(`Full Hash: ${hash}`);
        }
        if (options.binary) {
            const serializer = new BinarySerializer();
            const binary = serializer.serialize(sexp);
            console.log(`\\nBinary size: ${binary.length} bytes`);
            console.log(`Binary (hex): ${Array.from(binary).map(b => b.toString(16).padStart(2, '0')).join(' ')}`);
        }
    }
    catch (error) {
        console.error('Error:', error instanceof Error ? error.message : error);
        process.exit(1);
    }
});
program
    .command('compile')
    .description('Compile S-expression file to binary format')
    .argument('<input>', 'Input S-expression file (.s)')
    .argument('[output]', 'Output binary file (.s.bin) - defaults to input.s.bin')
    .action(async (input, output) => {
    try {
        const content = readFileSync(input, 'utf-8');
        const parser = new SExpParser();
        const sexp = parser.parse(content);
        const serializer = new BinarySerializer();
        const binary = serializer.serialize(sexp);
        const outputFile = output || input.replace(/\.s$/, '.s.bin').replace(/\.sexp$/, '.s.bin') + (input.endsWith('.s') || input.endsWith('.sexp') ? '' : '.s.bin');
        writeFileSync(outputFile, binary);
        const hash = generateContentHash(sexp);
        console.log(`Compiled: ${input} -> ${outputFile}`);
        console.log(`Size: ${binary.length} bytes`);
        console.log(`Content Hash: ${hash}`);
    }
    catch (error) {
        console.error('Error:', error instanceof Error ? error.message : error);
        process.exit(1);
    }
});
program
    .command('diff')
    .description('Compare two S-expression files')
    .argument('<file1>', 'First S-expression file')
    .argument('<file2>', 'Second S-expression file')
    .option('--no-color', 'Disable colored output')
    .option('--no-path', 'Hide path information')
    .option('--compact', 'Compact output (hide unchanged elements)')
    .option('--structural', 'Show structural diff')
    .action(async (file1, file2, options) => {
    try {
        // ファイル1の読み込み
        let sexp1;
        const ext1 = extname(file1);
        if (ext1 === '.bin' || file1.endsWith('.s.bin')) {
            const binaryData1 = readFileSync(file1);
            const serializer = new BinarySerializer();
            sexp1 = serializer.deserialize(new Uint8Array(binaryData1));
        }
        else {
            const content1 = readFileSync(file1, 'utf-8');
            const parser = new SExpParser();
            sexp1 = parser.parse(content1);
        }
        // ファイル2の読み込み
        let sexp2;
        const ext2 = extname(file2);
        if (ext2 === '.bin' || file2.endsWith('.s.bin')) {
            const binaryData2 = readFileSync(file2);
            const serializer = new BinarySerializer();
            sexp2 = serializer.deserialize(new Uint8Array(binaryData2));
        }
        else {
            const content2 = readFileSync(file2, 'utf-8');
            const parser = new SExpParser();
            sexp2 = parser.parse(content2);
        }
        const diff = new StructuralDiff();
        const operations = diff.diff(sexp1, sexp2);
        const renderer = new DiffRenderer({
            colorOutput: options.color,
            showPath: options.path,
            compact: options.compact
        });
        if (operations.length === 0) {
            console.log('No differences found.');
            return;
        }
        console.log(`Comparing ${file1} and ${file2}:`);
        console.log('');
        if (options.structural) {
            console.log(renderer.renderStructural(operations));
        }
        else {
            console.log(renderer.render(operations));
        }
        // 統計情報
        const stats = operations.reduce((acc, op) => {
            acc[op.type]++;
            return acc;
        }, { equal: 0, insert: 0, delete: 0, replace: 0 });
        console.log('');
        console.log(`Summary: ${stats.insert} insertions, ${stats.delete} deletions, ${stats.replace} replacements`);
    }
    catch (error) {
        console.error('Error:', error instanceof Error ? error.message : error);
        process.exit(1);
    }
});
program
    .command('binary-diff')
    .description('Compare binary representations of S-expression files')
    .argument('<file1>', 'First S-expression file')
    .argument('<file2>', 'Second S-expression file')
    .action(async (file1, file2) => {
    try {
        const content1 = readFileSync(file1, 'utf-8');
        const content2 = readFileSync(file2, 'utf-8');
        const parser = new SExpParser();
        const sexp1 = parser.parse(content1);
        const sexp2 = parser.parse(content2);
        const serializer = new BinarySerializer();
        const binary1 = serializer.serialize(sexp1);
        const binary2 = serializer.serialize(sexp2);
        const hash1 = calculateHash(sexp1);
        const hash2 = calculateHash(sexp2);
        console.log(`${file1}:`);
        console.log(`  Size: ${binary1.length} bytes`);
        console.log(`  Hash: ${hash1}`);
        console.log(`  Content Hash: ${generateContentHash(sexp1)}`);
        console.log(`\\n${file2}:`);
        console.log(`  Size: ${binary2.length} bytes`);
        console.log(`  Hash: ${hash2}`);
        console.log(`  Content Hash: ${generateContentHash(sexp2)}`);
        if (hash1 === hash2) {
            console.log('\\n✓ Files are identical (same content hash)');
        }
        else {
            console.log('\\n✗ Files are different');
            console.log(`Size difference: ${binary2.length - binary1.length} bytes`);
        }
    }
    catch (error) {
        console.error('Error:', error instanceof Error ? error.message : error);
        process.exit(1);
    }
});
program
    .command('git-diff')
    .description('Git difftool integration (internal use)')
    .argument('<path>', 'File path')
    .argument('<old-file>', 'Old file')
    .argument('<old-hex>', 'Old hex')
    .argument('<old-mode>', 'Old mode')
    .argument('<new-file>', 'New file')
    .argument('<new-hex>', 'New hex')
    .argument('<new-mode>', 'New mode')
    .action(async (path, oldFile, oldHex, oldMode, newFile, newHex, newMode) => {
    try {
        // Git difftool形式の引数を処理
        console.log(`Comparing ${path}:`);
        console.log('');
        const content1 = readFileSync(oldFile, 'utf-8');
        const content2 = readFileSync(newFile, 'utf-8');
        const parser = new SExpParser();
        const sexp1 = parser.parse(content1);
        const sexp2 = parser.parse(content2);
        const diff = new StructuralDiff();
        const operations = diff.diff(sexp1, sexp2);
        const renderer = new DiffRenderer({
            colorOutput: true,
            showPath: true,
            compact: false
        });
        if (operations.length === 0) {
            console.log('No differences found.');
            return;
        }
        console.log(renderer.renderStructural(operations));
    }
    catch (error) {
        console.error('Error:', error instanceof Error ? error.message : error);
        process.exit(1);
    }
});
program.parse();
//# sourceMappingURL=cli.js.map