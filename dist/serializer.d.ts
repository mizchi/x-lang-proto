/**
 * S式のバイナリシリアライザー
 * Unisonスタイルのcontent-addressed storageを参考にした実装
 */
import { SExp } from './sexp.js';
export declare class BinarySerializer {
    serialize(sexp: SExp): Uint8Array;
    private serializeValue;
    private serializeString;
    private serializeFloat;
    private serializeLength;
    private serializeVarint;
    deserialize(data: Uint8Array): SExp;
    private deserializeValue;
}
/**
 * Content-addressed storage: S式のハッシュを計算
 */
export declare function calculateHash(sexp: SExp): string;
/**
 * Unisonスタイルのハッシュベースの識別子を生成
 */
export declare function generateContentHash(sexp: SExp): string;
//# sourceMappingURL=serializer.d.ts.map