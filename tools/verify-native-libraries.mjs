import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";
import path from "node:path";

const filenames = {
  linux: "libgamestruments_godot.so",
  windows: "gamestruments_godot.dll",
  macos: "libgamestruments_godot.dylib",
};

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

export function verifyElfX8664(buffer) {
  assert(buffer.length >= 20, "Linux library is too small to be ELF");
  assert(buffer.subarray(0, 4).equals(Buffer.from([0x7f, 0x45, 0x4c, 0x46])), "Linux library is not ELF");
  assert(buffer[4] === 2, "Linux library is not 64-bit ELF");
  const littleEndian = buffer[5] === 1;
  assert(littleEndian || buffer[5] === 2, "Linux library has an invalid ELF byte order");
  const objectType = littleEndian ? buffer.readUInt16LE(16) : buffer.readUInt16BE(16);
  const machine = littleEndian ? buffer.readUInt16LE(18) : buffer.readUInt16BE(18);
  assert(objectType === 3, `Linux ELF type is ${objectType}, expected a shared object`);
  assert(machine === 62, `Linux ELF machine is ${machine}, expected x86_64`);
}

export function verifyPeX8664(buffer) {
  assert(buffer.length >= 64, "Windows library is too small to be PE");
  assert(buffer[0] === 0x4d && buffer[1] === 0x5a, "Windows library is missing the MZ header");
  const peOffset = buffer.readUInt32LE(0x3c);
  assert(peOffset + 24 <= buffer.length, "Windows library has an invalid PE offset");
  assert(buffer.subarray(peOffset, peOffset + 4).equals(Buffer.from("PE\0\0")), "Windows library is missing the PE signature");
  const machine = buffer.readUInt16LE(peOffset + 4);
  const characteristics = buffer.readUInt16LE(peOffset + 22);
  assert(machine === 0x8664, `Windows PE machine is 0x${machine.toString(16)}, expected x86_64`);
  assert((characteristics & 0x2000) !== 0, "Windows PE is not marked as a DLL");
}

export function verifyFatMachO(buffer) {
  assert(buffer.length >= 8, "macOS library is too small to be fat Mach-O");
  const magic = buffer.readUInt32BE(0);
  const formats = new Map([
    [0xcafebabe, { littleEndian: false, entrySize: 20 }],
    [0xbebafeca, { littleEndian: true, entrySize: 20 }],
    [0xcafebabf, { littleEndian: false, entrySize: 32 }],
    [0xbfbafeca, { littleEndian: true, entrySize: 32 }],
  ]);
  const format = formats.get(magic);
  assert(format, "macOS library is not a fat Mach-O binary");
  const readUInt32 = format.littleEndian
    ? (offset) => buffer.readUInt32LE(offset)
    : (offset) => buffer.readUInt32BE(offset);
  const architectureCount = readUInt32(4);
  assert(architectureCount > 0 && architectureCount <= 32, "macOS fat header has an invalid architecture count");
  assert(8 + architectureCount * format.entrySize <= buffer.length, "macOS fat architecture table is truncated");
  const architectures = new Set();
  for (let index = 0; index < architectureCount; index += 1) {
    const entryOffset = 8 + index * format.entrySize;
    const cpuType = readUInt32(entryOffset);
    const sliceOffset = format.entrySize === 20
      ? readUInt32(entryOffset + 8)
      : Number(format.littleEndian
        ? buffer.readBigUInt64LE(entryOffset + 8)
        : buffer.readBigUInt64BE(entryOffset + 8));
    const sliceSize = format.entrySize === 20
      ? readUInt32(entryOffset + 12)
      : Number(format.littleEndian
        ? buffer.readBigUInt64LE(entryOffset + 16)
        : buffer.readBigUInt64BE(entryOffset + 16));
    assert(Number.isSafeInteger(sliceOffset) && Number.isSafeInteger(sliceSize), "macOS fat slice exceeds safe file bounds");
    assert(sliceSize >= 16 && sliceOffset + sliceSize <= buffer.length, "macOS fat slice is truncated");
    const sliceMagic = buffer.readUInt32BE(sliceOffset);
    const sliceLittleEndian = sliceMagic === 0xcffaedfe;
    assert(sliceLittleEndian || sliceMagic === 0xfeedfacf, "macOS fat slice is not 64-bit Mach-O");
    const sliceCpuType = sliceLittleEndian
      ? buffer.readUInt32LE(sliceOffset + 4)
      : buffer.readUInt32BE(sliceOffset + 4);
    assert(sliceCpuType === cpuType, "macOS fat slice architecture does not match its table entry");
    const sliceFileType = sliceLittleEndian
      ? buffer.readUInt32LE(sliceOffset + 12)
      : buffer.readUInt32BE(sliceOffset + 12);
    assert(sliceFileType === 6, `macOS fat slice file type is ${sliceFileType}, expected MH_DYLIB`);
    architectures.add(cpuType);
  }
  assert(architectures.has(0x01000007), "macOS library is missing x86_64");
  assert(architectures.has(0x0100000c), "macOS library is missing arm64");
}

export function assertEntryPointMarker(buffer, label = "library") {
  assert(
    buffer.toString("latin1").includes("gdext_rust_init"),
    `${label} does not export the gdext_rust_init entry point`,
  );
}

export function findPrivateBuildPaths(buffer) {
  const printableStrings = buffer.toString("latin1").match(/[\x20-\x7e]{6,}/g) ?? [];
  const patterns = [
    /\/(?:home|Users)\/[^/\s\0]+\/(?:Workspace|workspace|work|projects?|src|\.cargo|\.rustup)(?:\/|\\)[^\s\0]*/gi,
    /\/__w\/[^\s\0]*/g,
    /[A-Za-z]:\\(?:Users\\[^\\\s\0]+\\(?:source|projects?|\.cargo|\.rustup)|a\\[^\\\s\0]+\\)[^\s\0]*/gi,
  ];
  return [
    ...new Set(
      printableStrings.flatMap((value) =>
        patterns.flatMap((pattern) => value.match(pattern) ?? []),
      ),
    ),
  ];
}

async function readAndVerify(assetsDir) {
  const buffers = {
    linux: await readFile(path.join(assetsDir, filenames.linux)),
    windows: await readFile(path.join(assetsDir, filenames.windows)),
    macos: await readFile(path.join(assetsDir, filenames.macos)),
  };
  verifyElfX8664(buffers.linux);
  verifyPeX8664(buffers.windows);
  verifyFatMachO(buffers.macos);
  for (const [platform, buffer] of Object.entries(buffers)) {
    assertEntryPointMarker(buffer, `${platform} library`);
    const paths = findPrivateBuildPaths(buffer);
    assert(paths.length === 0, `${platform} library contains private build paths: ${paths.slice(0, 3).join(", ")}`);
  }
  return buffers;
}

export async function verifyNativeAssets(assetsDir) {
  await readAndVerify(path.resolve(assetsDir));
}

export async function verifyPackagedNativeLibraries(root) {
  const archiveRoot = path.resolve(root);
  const rootAssets = path.join(archiveRoot, "addons", "gamestruments", "bin");
  const demoAssets = path.join(archiveRoot, "kit", "demo", "addons", "gamestruments", "bin");
  const rootBuffers = await readAndVerify(rootAssets);
  for (const [platform, filename] of Object.entries(filenames)) {
    const demoBuffer = await readFile(path.join(demoAssets, filename));
    assert(rootBuffers[platform].equals(demoBuffer), `${filename} differs between root and demo addons`);
  }
}

const isMain = process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url);
if (isMain) {
  const { values } = parseArgs({
    options: {
      root: { type: "string" },
      "assets-dir": { type: "string" },
    },
    strict: true,
  });
  if (values.root === undefined && values["assets-dir"] === undefined) {
    throw new Error("Usage: node tools/verify-native-libraries.mjs (--root DIR | --assets-dir DIR)");
  }
  if (values.root !== undefined && values["assets-dir"] !== undefined) {
    throw new Error("Choose either --root or --assets-dir");
  }
  if (values.root !== undefined) {
    await verifyPackagedNativeLibraries(values.root);
  } else {
    await verifyNativeAssets(values["assets-dir"]);
  }
  console.log("GAMESTRUMENTS_NATIVE_LIBRARIES_VERIFY_PASS");
}
