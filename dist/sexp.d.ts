/**
 * S式のデータ構造とパーサー
 */
export type SExp = {
    type: 'atom';
    value: string | number | boolean;
} | {
    type: 'list';
    elements: SExp[];
} | {
    type: 'symbol';
    name: string;
};
export declare class SExpParser {
    private pos;
    private input;
    parse(input: string): SExp;
    private parseExpression;
    private parseList;
    private parseString;
    private parseNumber;
    private parseBoolean;
    private parseSymbol;
    private skipWhitespace;
    private isWhitespace;
    private isDigit;
    private getEscapeChar;
}
export declare function sexpToString(sexp: SExp): string;
//# sourceMappingURL=sexp.d.ts.map