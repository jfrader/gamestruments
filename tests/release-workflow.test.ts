import { readFile } from "node:fs/promises";
import { describe, expect, it } from "vitest";

describe("release workflow", () => {
  it("keeps synthetic packages non-release even on tag pushes without overriding real releases", async () => {
    const checkWorkflow = await readFile(new URL("../.github/workflows/check.yml", import.meta.url), "utf8");
    const releaseWorkflow = await readFile(new URL("../.github/workflows/release.yml", import.meta.url), "utf8");
    const fixtureCommands = checkWorkflow.split("\n").filter((line) => line.includes("tools/package_kit.sh"));
    expect(fixtureCommands).toHaveLength(1);
    expect(fixtureCommands[0]?.trim()).toMatch(/^GAMESTRUMENTS_PROVENANCE=local tools\/package_kit\.sh --version 0\.1\.0-ci /);
    expect(releaseWorkflow).not.toContain("GAMESTRUMENTS_PROVENANCE");
  });

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
