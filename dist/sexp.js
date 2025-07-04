/**
 * S式のデータ構造とパーサー
 */
export class SExpParser {
    pos = 0;
    input = '';
    parse(input) {
        this.input = input.trim();
        this.pos = 0;
        return this.parseExpression();
    }
    parseExpression() {
        this.skipWhitespace();
        if (this.pos >= this.input.length) {
            throw new Error('Unexpected end of input');
        }
        const char = this.input[this.pos];
        if (char === '(') {
            return this.parseList();
        }
        else if (char === '"') {
            return this.parseString();
        }
        else if (this.isDigit(char) || char === '-') {
            return this.parseNumber();
        }
        else if (char === '#') {
            return this.parseBoolean();
        }
        else {
            return this.parseSymbol();
        }
    }
    parseList() {
        this.pos++; // skip '('
        const elements = [];
        while (this.pos < this.input.length) {
            this.skipWhitespace();
            if (this.pos >= this.input.length) {
                throw new Error('Unclosed list');
            }
            if (this.input[this.pos] === ')') {
                this.pos++;
                return { type: 'list', elements };
            }
            elements.push(this.parseExpression());
        }
        throw new Error('Unclosed list');
    }
    parseString() {
        this.pos++; // skip opening quote
        let value = '';
        while (this.pos < this.input.length) {
            const char = this.input[this.pos];
            if (char === '"') {
                this.pos++;
                return { type: 'atom', value };
            }
            else if (char === '\\') {
                this.pos++;
                if (this.pos >= this.input.length) {
                    throw new Error('Unexpected end of string');
                }
                const escapeChar = this.input[this.pos];
                value += this.getEscapeChar(escapeChar);
            }
            else {
                value += char;
            }
            this.pos++;
        }
        throw new Error('Unclosed string');
    }
    parseNumber() {
        let value = '';
        if (this.input[this.pos] === '-') {
            // マイナス記号の次が数字でない場合はシンボル
            if (this.pos + 1 >= this.input.length || !this.isDigit(this.input[this.pos + 1])) {
                return this.parseSymbol();
            }
            value += '-';
            this.pos++;
        }
        while (this.pos < this.input.length && (this.isDigit(this.input[this.pos]) || this.input[this.pos] === '.')) {
            value += this.input[this.pos];
            this.pos++;
        }
        const num = value.includes('.') ? parseFloat(value) : parseInt(value, 10);
        return { type: 'atom', value: num };
    }
    parseBoolean() {
        if (this.input.substr(this.pos, 2) === '#t') {
            this.pos += 2;
            return { type: 'atom', value: true };
        }
        else if (this.input.substr(this.pos, 2) === '#f') {
            this.pos += 2;
            return { type: 'atom', value: false };
        }
        throw new Error('Invalid boolean');
    }
    parseSymbol() {
        let name = '';
        while (this.pos < this.input.length && !this.isWhitespace(this.input[this.pos]) && this.input[this.pos] !== ')') {
            name += this.input[this.pos];
            this.pos++;
        }
        return { type: 'symbol', name };
    }
    skipWhitespace() {
        while (this.pos < this.input.length && this.isWhitespace(this.input[this.pos])) {
            this.pos++;
        }
    }
    isWhitespace(char) {
        return /\s/.test(char);
    }
    isDigit(char) {
        return /\d/.test(char);
    }
    getEscapeChar(char) {
        switch (char) {
            case 'n': return '\n';
            case 't': return '\t';
            case 'r': return '\r';
            case '\\': return '\\';
            case '"': return '"';
            default: return char;
        }
    }
}
export function sexpToString(sexp) {
    switch (sexp.type) {
        case 'atom':
            if (typeof sexp.value === 'string') {
                return `"${sexp.value.replace(/\\/g, '\\\\').replace(/"/g, '\\"')}"`;
            }
            else if (typeof sexp.value === 'boolean') {
                return sexp.value ? '#t' : '#f';
            }
            else {
                return sexp.value.toString();
            }
        case 'symbol':
            return sexp.name;
        case 'list':
            return `(${sexp.elements.map(sexpToString).join(' ')})`;
    }
}
//# sourceMappingURL=sexp.js.map