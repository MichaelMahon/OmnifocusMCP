import { describe, expect, it } from "vitest";

import { escapeForJxa } from "../src/jxa.js";

describe("escapeForJxa", () => {
  const adversarialCases = [
    'He said "hello"',
    String.raw`C:\Users\test`,
    "line1\nline2\tend",
    "null\0byte",
    "こんにちは",
    "emoji: 🚀✨",
    "",
    "a".repeat(10_000),
  ];

  it.each(adversarialCases)("round-trips %j", (value) => {
    const escaped = escapeForJxa(value);
    expect(JSON.parse(escaped)).toBe(value);
  });

  it("returns valid quoted json for empty strings", () => {
    expect(escapeForJxa("")).toBe('""');
  });
});
