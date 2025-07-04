/**
 * S式のバイナリシリアライザー
 * Unisonスタイルのcontent-addressed storageを参考にした実装
 */

import { SExp } from './sexp.js';
import { createHash } from 'crypto';

// バイナリフォーマットの定義
const ATOM_STRING = 0x01;
const ATOM_NUMBER_INT = 0x02;
const ATOM_NUMBER_FLOAT = 0x03;
const ATOM_BOOLEAN = 0x04;
const SYMBOL = 0x05;
const LIST = 0x06;

export class BinarySerializer {
  serialize(sexp: SExp): Uint8Array {
    const buffer: number[] = [];
    this.serializeValue(sexp, buffer);
    return new Uint8Array(buffer);
  }

  private serializeValue(sexp: SExp, buffer: number[]): void {
    switch (sexp.type) {
      case 'atom':
        if (typeof sexp.value === 'string') {
          buffer.push(ATOM_STRING);
          this.serializeString(sexp.value, buffer);
        } else if (typeof sexp.value === 'number') {
          if (Number.isInteger(sexp.value)) {
            buffer.push(ATOM_NUMBER_INT);
            this.serializeVarint(sexp.value, buffer);
          } else {
            buffer.push(ATOM_NUMBER_FLOAT);
            this.serializeFloat(sexp.value, buffer);
          }
        } else if (typeof sexp.value === 'boolean') {
          buffer.push(ATOM_BOOLEAN);
          buffer.push(sexp.value ? 1 : 0);
        }
        break;
      case 'symbol':
        buffer.push(SYMBOL);
        this.serializeString(sexp.name, buffer);
        break;
      case 'list':
        buffer.push(LIST);
        this.serializeLength(sexp.elements.length, buffer);
        for (const element of sexp.elements) {
          this.serializeValue(element, buffer);
        }
        break;
    }
  }

  private serializeString(str: string, buffer: number[]): void {
    const bytes = new TextEncoder().encode(str);
    this.serializeLength(bytes.length, buffer);
    buffer.push(...bytes);
  }

  private serializeFloat(num: number, buffer: number[]): void {
    const view = new DataView(new ArrayBuffer(8));
    view.setFloat64(0, num, true); // little endian
    for (let i = 0; i < 8; i++) {
      buffer.push(view.getUint8(i));
    }
  }

  private serializeLength(length: number, buffer: number[]): void {
    this.serializeVarint(length, buffer);
  }

  private serializeVarint(value: number, buffer: number[]): void {
    while (value >= 0x80) {
      buffer.push((value & 0xff) | 0x80);
      value >>>= 7;
    }
    buffer.push(value & 0xff);
  }

  deserialize(data: Uint8Array): SExp {
    const reader = new BinaryReader(data);
    return this.deserializeValue(reader);
  }

  private deserializeValue(reader: BinaryReader): SExp {
    const type = reader.readByte();
    
    switch (type) {
      case ATOM_STRING:
        return { type: 'atom', value: reader.readString() };
      case ATOM_NUMBER_INT:
        return { type: 'atom', value: reader.readVarint() };
      case ATOM_NUMBER_FLOAT:
        return { type: 'atom', value: reader.readFloat() };
      case ATOM_BOOLEAN:
        return { type: 'atom', value: reader.readByte() === 1 };
      case SYMBOL:
        return { type: 'symbol', name: reader.readString() };
      case LIST:
        const length = reader.readVarint();
        const elements: SExp[] = [];
        for (let i = 0; i < length; i++) {
          elements.push(this.deserializeValue(reader));
        }
        return { type: 'list', elements };
      default:
        throw new Error(`Unknown type: ${type}`);
    }
  }
}

class BinaryReader {
  private pos = 0;
  
  constructor(private data: Uint8Array) {}

  readByte(): number {
    if (this.pos >= this.data.length) {
      throw new Error('Unexpected end of data');
    }
    return this.data[this.pos++];
  }

  readString(): string {
    const length = this.readVarint();
    const bytes = this.data.slice(this.pos, this.pos + length);
    this.pos += length;
    return new TextDecoder().decode(bytes);
  }

  readFloat(): number {
    if (this.pos + 8 > this.data.length) {
      throw new Error(`Not enough data for float64: need 8 bytes, have ${this.data.length - this.pos}`);
    }
    
    const view = new DataView(this.data.buffer, this.data.byteOffset + this.pos, 8);
    const num = view.getFloat64(0, true);
    this.pos += 8;
    return num;
  }

  readVarint(): number {
    let value = 0;
    let shift = 0;
    
    while (true) {
      const byte = this.readByte();
      value |= (byte & 0x7f) << shift;
      
      if ((byte & 0x80) === 0) {
        break;
      }
      
      shift += 7;
    }
    
    return value;
  }
}

/**
 * Content-addressed storage: S式のハッシュを計算
 */
export function calculateHash(sexp: SExp): string {
  const serializer = new BinarySerializer();
  const binary = serializer.serialize(sexp);
  return createHash('sha256').update(binary).digest('hex');
}

/**
 * Unisonスタイルのハッシュベースの識別子を生成
 */
export function generateContentHash(sexp: SExp): string {
  const hash = calculateHash(sexp);
  // Unisonスタイルのbase32hex表現（最初の8文字）
  return hash.substring(0, 8);
}