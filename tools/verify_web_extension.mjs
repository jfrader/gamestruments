// Checks one freshly built web side module before it is staged:
//   node tools/verify_web_extension.mjs <gamestruments_godot.wasm>
import { readFileSync } from "node:fs";
import { findPrivateBuildPaths, verifyWasmSideModule } from "./verify-native-libraries.mjs";

const [file] = process.argv.slice(2);
if (!file) {
  console.error("usage: node tools/verify_web_extension.mjs <side-module.wasm>");
  process.exit(2);
}
const bytes = readFileSync(file);
verifyWasmSideModule(bytes);
const paths = findPrivateBuildPaths(bytes);
if (paths.length > 0) {
  console.error(`WEB_EXTENSION_VERIFY_FAIL ${file} contains private build paths: ${paths.slice(0, 3).join(", ")}`);
  process.exit(1);
}
console.log(`WEB_EXTENSION_VERIFY_PASS ${file} (${bytes.length} bytes)`);
