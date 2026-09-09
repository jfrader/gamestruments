import { readFile } from "node:fs/promises";
import { describe, expect, it } from "vitest";

describe("release workflow", () => {
  it("passes repository context to release commands that run without checkout", async () => {
    const workflow = await readFile(
      new URL("../.github/workflows/release.yml", import.meta.url),
      "utf8",
    );
    const releaseCommands = workflow.match(/gh release (?:view|create)[^\n]*/g) ?? [];

    expect(releaseCommands).toHaveLength(2);
    for (const command of releaseCommands) {
      expect(command).toContain('--repo "$GITHUB_REPOSITORY"');
    }
  });
});
