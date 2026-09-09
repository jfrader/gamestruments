import { readFile } from "node:fs/promises";
import process from "node:process";

const [metadataPath, noticesPath] = process.argv.slice(2);
if (!metadataPath || !noticesPath) {
  throw new Error("Usage: node tests/verify-third-party-notices.mjs <metadata.json> <notices.md>");
}

const metadata = JSON.parse(await readFile(metadataPath, "utf8"));
const notices = await readFile(noticesPath, "utf8");
const missing = [
  ...new Set(
    metadata.packages
      .filter((pkg) => pkg.source !== null)
      .map((pkg) => pkg.name)
      .filter((name) => !notices.includes(`\`${name}\``)),
  ),
].sort();

if (missing.length > 0) {
  throw new Error(`THIRD_PARTY_NOTICES.md is missing: ${missing.join(", ")}`);
}

console.log("THIRD_PARTY_NOTICES_VERIFY_PASS");
