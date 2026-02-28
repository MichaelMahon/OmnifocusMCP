import { beforeEach, describe, expect, test, vi } from "vitest";

const execFileAsyncMock = vi.fn();

vi.mock("node:child_process", () => ({
  execFile: vi.fn(),
}));

vi.mock("node:util", () => ({
  promisify: vi.fn(() => execFileAsyncMock),
}));

describe("jxa error paths", () => {
  beforeEach(() => {
    execFileAsyncMock.mockReset();
  });

  test("runJxa surfaces non-zero stderr cleanly", async () => {
    const error = new Error("failed") as Error & { stderr?: unknown };
    error.stderr = "execution error: OmniFocus got an error";
    execFileAsyncMock.mockRejectedValueOnce(error);
    const { runJxa } = await import("../src/jxa.js");
    await expect(runJxa("1+1")).rejects.toThrow("JXA execution failed:");
  });

  test("runJxa reports timeout error", async () => {
    const error = new Error("timed out") as Error & { code?: unknown };
    error.code = "ETIMEDOUT";
    execFileAsyncMock.mockRejectedValueOnce(error);
    const { runJxa } = await import("../src/jxa.js");
    await expect(runJxa("1+1", 1_000)).rejects.toThrow("JXA command timed out after 1s.");
  });

  test("runOmniJs fails on malformed JSON envelope", async () => {
    execFileAsyncMock.mockResolvedValueOnce({ stdout: "not-json" });
    const { runOmniJs } = await import("../src/jxa.js");
    await expect(runOmniJs("return 1;")).rejects.toThrow("JXA command returned malformed JSON.");
  });
});
