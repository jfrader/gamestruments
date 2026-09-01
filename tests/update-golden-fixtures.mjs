import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const cliPath = fileURLToPath(
  new URL("../packages/studio/dist/cli.js", import.meta.url),
);
const fixtures = [
  {
    recipe: "pocket-circuit",
    directory: "pocket-circuit",
    arguments: [
      "--style",
      "neon",
      "--energy",
      "0.73",
      "--complexity",
      "0.67",
      "--brightness",
      "0.81",
      "--syncopation",
      "0.59",
    ],
  },
  {
    recipe: "lantern-trail",
    directory: "lantern-trail/v1",
    arguments: [
      "--wonder",
      "0.78",
      "--danger",
      "0.64",
      "--mystery",
      "0.83",
      "--motion",
      "0.57",
    ],
  },
];

for (const fixture of fixtures) {
  const scorePath = fileURLToPath(
    new URL(`./fixtures/${fixture.directory}/golden.score.json`, import.meta.url),
  );
  const manifestPath = fileURLToPath(
    new URL(
      `./fixtures/${fixture.directory}/golden.manifest.json`,
      import.meta.url,
    ),
  );
  const result = spawnSync(
    process.execPath,
    [
      cliPath,
      "--recipe",
      fixture.recipe,
      "--seed",
      "golden-v1",
      ...fixture.arguments,
      "--output",
      scorePath,
      "--manifest",
      manifestPath,
      "--pretty",
    ],
    { encoding: "utf8" },
  );

  if (result.status !== 0) {
    throw new Error(
      result.stderr ||
        `${fixture.recipe} fixture generation exited with ${result.status}`,
    );
  }
}

process.stdout.write("updated Pocket Circuit and Lantern Trail golden fixtures\n");
