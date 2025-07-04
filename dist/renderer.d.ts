/**
 * 差分結果の視覚化
 */
import { DiffOperation } from './diff.js';
export interface RenderOptions {
    colorOutput: boolean;
    showPath: boolean;
    compact: boolean;
}
export declare class DiffRenderer {
    private options;
    constructor(options?: Partial<RenderOptions>);
    render(operations: DiffOperation[]): string;
    private renderEqual;
    private renderInsert;
    private renderDelete;
    private renderReplace;
    /**
     * 構造的な差分を階層的に表示
     */
    renderStructural(operations: DiffOperation[]): string;
    private groupByPath;
    private getPathKey;
    private renderPath;
    private renderOperation;
}
//# sourceMappingURL=renderer.d.ts.map