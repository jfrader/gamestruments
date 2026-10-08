import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";
import path from "node:path";

const filenames = {
  linux: "libgamestruments_godot.so",
  windows: "gamestruments_godot.dll",
  macos: "libgamestruments_godot.dylib",
  web: "gamestruments_godot.wasm",
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

function readLeb(buffer, offset) {
  let result = 0;
  let shift = 0;
  let position = offset;
  for (;;) {
    assert(position < buffer.length, "web library has a truncated LEB128 value");
    const byte = buffer[position];
    position += 1;
    result += (byte & 0x7f) * 2 ** shift;
    shift += 7;
    if ((byte & 0x80) === 0) {
      return { value: result, next: position };
    }
    assert(shift < 35, "web library has an oversized LEB128 value");
  }
}

function readName(buffer, offset) {
  const { value: length, next } = readLeb(buffer, offset);
  assert(next + length <= buffer.length, "web library has a truncated name");
  return { value: buffer.toString("utf8", next, next + length), next: next + length };
}

/** Reads the parts of a WebAssembly module a Godot dlink template cares about. */
export function inspectWasmModule(buffer) {
  assert(buffer.length >= 8, "web library is too small to be WebAssembly");
  assert(buffer.subarray(0, 4).equals(Buffer.from([0x00, 0x61, 0x73, 0x6d])), "web library is not WebAssembly");
  assert(buffer.readUInt32LE(4) === 1, "web library is not WebAssembly version 1");
  const customSections = [];
  const imports = [];
  const exports = [];
  let offset = 8;
  while (offset < buffer.length) {
    const id = buffer[offset];
    const size = readLeb(buffer, offset + 1);
    const start = size.next;
    const end = start + size.value;
    assert(end <= buffer.length, "web library has a truncated section");
    if (id === 0) {
      customSections.push(readName(buffer, start).value);
    } else if (id === 2) {
      const count = readLeb(buffer, start);
      let position = count.next;
      for (let index = 0; index < count.value; index += 1) {
        const module = readName(buffer, position);
        const field = readName(buffer, module.next);
        const kind = buffer[field.next];
        position = field.next + 1;
        const entry = { module: module.value, field: field.value, kind, shared: false };
        if (kind === 0) {
          position = readLeb(buffer, position).next;
        } else if (kind === 1) {
          const flags = buffer[position + 1];
          position = readLeb(buffer, position + 2).next;
          if (flags & 1) position = readLeb(buffer, position).next;
        } else if (kind === 2) {
          const flags = buffer[position];
          entry.shared = (flags & 2) !== 0;
          position = readLeb(buffer, position + 1).next;
          if (flags & 1) position = readLeb(buffer, position).next;
        } else if (kind === 3) {
          position += 2;
        } else if (kind === 4) {
          position = readLeb(buffer, position + 1).next;
        } else {
          throw new Error(`web library has an unknown import kind ${kind}`);
        }
        imports.push(entry);
      }
    } else if (id === 7) {
      let cursor = readLeb(buffer, start);
      let position = cursor.next;
      for (let index = 0; index < cursor.value; index += 1) {
        const name = readName(buffer, position);
        exports.push({ name: name.value, kind: buffer[name.next] });
        position = readLeb(buffer, name.next + 1).next;
      }
    }
    offset = end;
  }
  return { customSections, imports, exports };
}

/** A web side module for Godot's single-threaded dlink templates: dylink.0
 *  first, the gdext entry point exported, no exception tags (the official
 *  templates are built without exception support), and unshared memory. */
export function verifyWasmSideModule(buffer) {
  const label = "web library";
  const module = inspectWasmModule(buffer);
  assert(module.customSections[0] === "dylink.0", `${label} is not an Emscripten side module (no leading dylink.0 section)`);
  assert(
    module.exports.some((entry) => entry.name === "gdext_rust_init" && entry.kind === 0),
    `${label} does not export the gdext_rust_init entry point`,
  );
  const tags = module.imports.filter((entry) => entry.kind === 4);
  assert(tags.length === 0, `${label} imports exception tags (${tags.map((tag) => tag.field).join(", ")}); build with panic=abort`);
  const memory = module.imports.find((entry) => entry.kind === 2);
  assert(memory, `${label} does not import the main module's memory`);
  assert(!memory.shared, "web library uses shared memory; build the single-threaded (nothreads) side module");
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
    web: await readFile(path.join(assetsDir, filenames.web)),
  };
  verifyElfX8664(buffers.linux);
  verifyPeX8664(buffers.windows);
  verifyFatMachO(buffers.macos);
  verifyWasmSideModule(buffers.web);
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
  const examplesAssets = path.join(archiveRoot, "kit", "examples", "addons", "gamestruments", "bin");
  const rootBuffers = await readAndVerify(rootAssets);
  for (const [platform, filename] of Object.entries(filenames)) {
    const examplesBuffer = await readFile(path.join(examplesAssets, filename));
    assert(rootBuffers[platform].equals(examplesBuffer), `${filename} differs between root and examples addons`);
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
