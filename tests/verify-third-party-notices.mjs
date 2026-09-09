import { createHash } from "node:crypto";
import { access, readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const [metadataPath, noticesPath, inventoryPath, licensesPath] = process.argv.slice(2);
if (!metadataPath || !noticesPath || !inventoryPath || !licensesPath) {
  throw new Error(
    "Usage: node tests/verify-third-party-notices.mjs <metadata.json> <notices.md> <inventory.json> <licenses-dir>",
  );
}

const metadata = JSON.parse(await readFile(metadataPath, "utf8"));
const notices = await readFile(noticesPath, "utf8");
const inventory = JSON.parse(await readFile(inventoryPath, "utf8"));
if (inventory.schemaVersion !== 1 || !Array.isArray(inventory.packages)) {
  throw new Error("Cargo dependency license inventory has an unsupported schema");
}

const cargoPackages = metadata.packages
  .filter((pkg) => pkg.source !== null)
  .map((pkg) => ({ name: pkg.name, version: pkg.version, license: pkg.license }))
  .sort((left, right) => left.name.localeCompare(right.name));
const inventoriedPackages = [...inventory.packages].sort((left, right) =>
  left.name.localeCompare(right.name),
);
if (JSON.stringify(cargoPackages) !== JSON.stringify(inventoriedPackages)) {
  throw new Error("Cargo dependency license inventory does not exactly match cargo metadata");
}

const missing = cargoPackages
  .filter(({ name, version }) => {
    const nameIndex = notices.indexOf(`\`${name}\``);
    const versionIndex = notices.indexOf(version, nameIndex);
    return nameIndex < 0 || versionIndex < nameIndex || versionIndex - nameIndex > 500;
  })
  .map(({ name, version }) => `${name}@${version}`);

if (missing.length > 0) {
  throw new Error(`THIRD_PARTY_NOTICES.md is missing: ${missing.join(", ")}`);
}

const usedLicenseIds = new Set(
  cargoPackages.flatMap(({ license }) =>
    Object.keys(inventory.licenseFiles).filter((licenseId) => license?.includes(licenseId)),
  ),
);
for (const licenseId of usedLicenseIds) {
  await access(path.join(licensesPath, inventory.licenseFiles[licenseId]));
}
for (const attribution of inventory.requiredAttributions ?? []) {
  const dependency = cargoPackages.find(
    ({ name, version }) => name === attribution.package && version === attribution.version,
  );
  if (!dependency) {
    throw new Error(
      `Attribution references absent dependency ${attribution.package}@${attribution.version}`,
    );
  }
  await access(path.join(licensesPath, attribution.file));
  if (!notices.includes(`licenses/${attribution.file}`)) {
    throw new Error(`THIRD_PARTY_NOTICES.md does not link ${attribution.file}`);
  }
}
for (const attribution of inventory.toolchainAttributions ?? []) {
  if (
    typeof attribution.component !== "string" ||
    typeof attribution.version !== "string" ||
    typeof attribution.file !== "string" ||
    !/^[0-9a-f]{64}$/.test(attribution.sha256 ?? "")
  ) {
    throw new Error("Toolchain attribution has an invalid schema");
  }
  const attributionPath = path.join(licensesPath, attribution.file);
  const bytes = await readFile(attributionPath);
  const digest = createHash("sha256").update(bytes).digest("hex");
  if (digest !== attribution.sha256) {
    throw new Error(`Toolchain attribution digest mismatch for ${attribution.file}`);
  }
  if (!notices.includes(`licenses/${attribution.file}`)) {
    throw new Error(`THIRD_PARTY_NOTICES.md does not link ${attribution.file}`);
  }
}

console.log("THIRD_PARTY_NOTICES_VERIFY_PASS");
