/**
 * 差分結果の視覚化
 */

import { DiffOperation } from './diff.js';
import { SExp, sexpToString } from './sexp.js';

export interface RenderOptions {
  colorOutput: boolean;
  showPath: boolean;
  compact: boolean;
}

export class DiffRenderer {
  private options: RenderOptions;

  constructor(options: Partial<RenderOptions> = {}) {
    this.options = {
      colorOutput: true,
      showPath: true,
      compact: false,
      ...options
    };
  }

  render(operations: DiffOperation[]): string {
    const lines: string[] = [];
    
    for (const op of operations) {
      switch (op.type) {
        case 'equal':
          if (!this.options.compact) {
            lines.push(this.renderEqual(op));
          }
          break;
        case 'insert':
          lines.push(this.renderInsert(op));
          break;
        case 'delete':
          lines.push(this.renderDelete(op));
          break;
        case 'replace':
          lines.push(this.renderReplace(op));
          break;
      }
    }
    
    return lines.join('\n');
  }

  private renderEqual(op: { type: 'equal'; left: SExp; right: SExp }): string {
    const content = sexpToString(op.left);
    return this.options.colorOutput
      ? `  ${content}`
      : `  ${content}`;
  }

  private renderInsert(op: { type: 'insert'; value: SExp; path: number[] }): string {
    const content = sexpToString(op.value);
    const pathStr = this.options.showPath ? ` @${op.path.join('.')}` : '';
    
    return this.options.colorOutput
      ? `\x1b[32m+ ${content}\x1b[0m${pathStr}`
      : `+ ${content}${pathStr}`;
  }

  private renderDelete(op: { type: 'delete'; value: SExp; path: number[] }): string {
    const content = sexpToString(op.value);
    const pathStr = this.options.showPath ? ` @${op.path.join('.')}` : '';
    
    return this.options.colorOutput
      ? `\x1b[31m- ${content}\x1b[0m${pathStr}`
      : `- ${content}${pathStr}`;
  }

  private renderReplace(op: { type: 'replace'; oldValue: SExp; newValue: SExp; path: number[] }): string {
    const oldContent = sexpToString(op.oldValue);
    const newContent = sexpToString(op.newValue);
    const pathStr = this.options.showPath ? ` @${op.path.join('.')}` : '';
    
    const lines = [];
    if (this.options.colorOutput) {
      lines.push(`\x1b[31m- ${oldContent}\x1b[0m${pathStr}`);
      lines.push(`\x1b[32m+ ${newContent}\x1b[0m${pathStr}`);
    } else {
      lines.push(`- ${oldContent}${pathStr}`);
      lines.push(`+ ${newContent}${pathStr}`);
    }
    
    return lines.join('\n');
  }

  /**
   * 構造的な差分を階層的に表示
   */
  renderStructural(operations: DiffOperation[]): string {
    const lines: string[] = [];
    
    // パス別に操作をグループ化
    const grouped = this.groupByPath(operations);
    
    for (const [pathKey, ops] of grouped) {
      const path = pathKey.split('.').map(Number).filter(n => !isNaN(n));
      if (path.length > 0) {
        lines.push(this.renderPath(path));
      }
      
      for (const op of ops) {
        lines.push(this.renderOperation(op, path.length));
      }
    }
    
    return lines.join('\n');
  }

  private groupByPath(operations: DiffOperation[]): Map<string, DiffOperation[]> {
    const grouped = new Map<string, DiffOperation[]>();
    
    for (const op of operations) {
      const pathKey = this.getPathKey(op);
      if (!grouped.has(pathKey)) {
        grouped.set(pathKey, []);
      }
      grouped.get(pathKey)!.push(op);
    }
    
    return grouped;
  }

  private getPathKey(op: DiffOperation): string {
    switch (op.type) {
      case 'equal':
        return '';
      case 'insert':
      case 'delete':
      case 'replace':
        return op.path.slice(0, -1).join('.');
    }
  }

  private renderPath(path: number[]): string {
    const indent = '  '.repeat(path.length);
    const pathStr = path.join('.');
    
    return this.options.colorOutput
      ? `${indent}\x1b[36m[${pathStr}]\x1b[0m`
      : `${indent}[${pathStr}]`;
  }

  private renderOperation(op: DiffOperation, baseIndent: number): string {
    const indent = '  '.repeat(baseIndent + 1);
    
    switch (op.type) {
      case 'equal':
        return `${indent}${this.renderEqual(op)}`;
      case 'insert':
        return `${indent}${this.renderInsert(op)}`;
      case 'delete':
        return `${indent}${this.renderDelete(op)}`;
      case 'replace':
        return `${indent}${this.renderReplace(op)}`;
    }
  }
}