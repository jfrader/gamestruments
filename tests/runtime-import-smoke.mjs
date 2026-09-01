import assert from "node:assert/strict";
import { readdir, readFile } from "node:fs/promises";
import {
  AdaptiveTransport,
  SCORE_SCHEMA_VERSION,
  validatePortableScore,
} from "@gamestruments/runtime";

const score = {
  schemaVersion: SCORE_SCHEMA_VERSION,
  id: "runtime-import-smoke",
  title: "Runtime Import Smoke",
  bpm: 120,
  beatsPerBar: 4,
  ticksPerBeat: 960,
  crossfadeBars: 1,
  defaultSection: "only",
  sections: [
    {
      id: "only",
      label: "Only",
      feeling: "steady",
      color: "#ffffff",
      lengthTicks: 3840,
      events: [],
    },
  ],
  rules: [],
};

validatePortableScore(score);
assert.equal(new AdaptiveTransport(score).snapshot().currentSection, "only");

const runtimeDist = new URL("../packages/runtime/dist/", import.meta.url);
const javascriptFiles = (await readdir(runtimeDist)).filter((file) =>
  file.endsWith(".js"),
);
assert.ok(javascriptFiles.length > 0);
const emittedSource = (
  await Promise.all(
    javascriptFiles.map((file) => readFile(new URL(file, runtimeDist), "utf8")),
  )
).join("\n");
assert.equal(emittedSource.toLowerCase().includes("strudel"), false);

process.stdout.write("runtime Node ESM import smoke passed\n");
