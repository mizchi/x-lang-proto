/**
 * 構造的なS式の差分計算
 * difftasticのアプローチを参考に、AST構造の差分を計算
 */
import { SExp } from './sexp.js';
export type DiffOperation = {
    type: 'equal';
    left: SExp;
    right: SExp;
} | {
    type: 'insert';
    value: SExp;
    path: number[];
} | {
    type: 'delete';
    value: SExp;
    path: number[];
} | {
    type: 'replace';
    oldValue: SExp;
    newValue: SExp;
    path: number[];
};
export declare class StructuralDiff {
    /**
     * 2つのS式の構造的な差分を計算
     */
    diff(left: SExp, right: SExp): DiffOperation[];
    private diffRecursive;
    private diffLists;
    private longestCommonSubsequence;
    private isEqual;
}
//# sourceMappingURL=diff.d.ts.map