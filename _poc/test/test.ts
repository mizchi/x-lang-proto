import { test, expect } from "vitest";
import { SExpParser, sexpToString } from "../src/sexp.js";
import {
  BinarySerializer,
  calculateHash,
  generateContentHash,
} from "../src/serializer.js";
import { StructuralDiff } from "../src/diff.js";
import { DiffRenderer } from "../src/renderer.js";

test("S式のパース", () => {
  const parser = new SExpParser();

  const sexp = parser.parse("(+ 1 2)");
  expect(sexp).toEqual({
    type: "list",
    elements: [
      { type: "symbol", name: "+" },
      { type: "atom", value: 1 },
      { type: "atom", value: 2 },
    ],
  });
});

test("S式の文字列化", () => {
  const sexp = {
    type: "list" as const,
    elements: [
      { type: "symbol" as const, name: "+" },
      { type: "atom" as const, value: 1 },
      { type: "atom" as const, value: 2 },
    ],
  };

  const result = sexpToString(sexp);
  expect(result).toBe("(+ 1 2)");
});

test("バイナリシリアライゼーション", () => {
  const parser = new SExpParser();
  const serializer = new BinarySerializer();

  const sexp = parser.parse("(+ 1 2)");
  const binary = serializer.serialize(sexp);
  const deserialized = serializer.deserialize(binary);

  expect(deserialized).toEqual(sexp);
});

test("ハッシュ計算", () => {
  const parser = new SExpParser();
  const sexp1 = parser.parse("(+ 1 2)");
  const sexp2 = parser.parse("(+ 1 2)");
  const sexp3 = parser.parse("(+ 2 1)");

  const hash1 = calculateHash(sexp1);
  const hash2 = calculateHash(sexp2);
  const hash3 = calculateHash(sexp3);

  expect(hash1).toBe(hash2);
  expect(hash1).not.toBe(hash3);
});

test("構造的差分", () => {
  const parser = new SExpParser();
  const diff = new StructuralDiff();

  const sexp1 = parser.parse("(+ 1 2)");
  const sexp2 = parser.parse("(+ 1 3)");

  const operations = diff.diff(sexp1, sexp2);

  expect(operations.length).toBeGreaterThan(0);
  expect(operations.some((op) => op.type === "replace")).toBe(true);
});

test("差分レンダリング", () => {
  const parser = new SExpParser();
  const diff = new StructuralDiff();
  const renderer = new DiffRenderer({ colorOutput: false });

  const sexp1 = parser.parse("(+ 1 2)");
  const sexp2 = parser.parse("(+ 1 3)");

  const operations = diff.diff(sexp1, sexp2);
  const result = renderer.render(operations);

  expect(result).toContain("2");
  expect(result).toContain("3");
});

test("複雑なS式の差分", () => {
  const parser = new SExpParser();
  const diff = new StructuralDiff();

  const sexp1 = parser.parse(
    "(defun factorial (n) (if (= n 0) 1 (* n (factorial (- n 1)))))"
  );
  const sexp2 = parser.parse(
    "(defun factorial (n) (if (<= n 1) 1 (* n (factorial (- n 1)))))"
  );

  const operations = diff.diff(sexp1, sexp2);

  expect(operations.length).toBeGreaterThan(0);
  expect(operations.some((op) => op.type === "replace")).toBe(true);
});

test("Content-addressed hash生成", () => {
  const parser = new SExpParser();
  const sexp = parser.parse("(+ 1 2)");

  const contentHash = generateContentHash(sexp);

  expect(contentHash).toHaveLength(8);
  expect(contentHash).toMatch(/^[0-9a-f]+$/);
});
