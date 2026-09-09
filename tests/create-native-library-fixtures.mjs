import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const [outputArgument] = process.argv.slice(2);
if (!outputArgument) {
  throw new Error("Usage: node tests/create-native-library-fixtures.mjs <output-dir>");
}
const output = path.resolve(outputArgument);
await mkdir(output, { recursive: true });

// Minimal-but-valid binaries that satisfy the format, architecture, and
// entry-point checks without being loadable. They are deliberately stubs: the
// entry-point marker proves the *name* is exported, nothing more.
const ELF_ENTRY = "gdext_rust_init";
const PE_ENTRY = "gdext_rust_init";
// Mach-O symbol tables prefix exported names with an underscore; the verifier
// matches the entry point by substring so both forms are accepted.
const MACHO_ENTRY = "_gdext_rust_init";

const elf = Buffer.alloc(128);
Buffer.from([0x7f, 0x45, 0x4c, 0x46]).copy(elf);
elf[4] = 2; // 64-bit
elf[5] = 1; // little-endian
elf.writeUInt16LE(3, 16); // ET_DYN (shared object)
elf.writeUInt16LE(62, 18); // x86_64
elf.write(ELF_ENTRY, 64, "latin1");

const pe = Buffer.alloc(160);
pe.write("MZ");
pe.writeUInt32LE(64, 0x3c); // PE header offset
pe.write("PE\0\0", 64);
pe.writeUInt16LE(0x8664, 68); // x86_64
pe.writeUInt16LE(0x2000, 86); // IMAGE_FILE_DLL
pe.write(PE_ENTRY, 128, "latin1");

// Big-endian fat Mach-O header with two 32-byte 64-bit little-endian slices,
// each carrying the MH_DYLIB file type and the underscore-prefixed entry point
// trailing after the slices.
const machO = Buffer.alloc(160);
machO.writeUInt32BE(0xcafebabe, 0); // FAT_MAGIC (BE)
machO.writeUInt32BE(2, 4); // two architectures
// fat_arch 0: x86_64
machO.writeUInt32BE(0x01000007, 8); // cputype
machO.writeUInt32BE(3, 12); // cpusubtype (x86_64 all)
machO.writeUInt32BE(48, 16); // slice offset
machO.writeUInt32BE(32, 20); // slice size
machO.writeUInt32BE(12, 24); // alignment
// fat_arch 1: arm64
machO.writeUInt32BE(0x0100000c, 28); // cputype
machO.writeUInt32BE(0, 32); // cpusubtype (arm64 all)
machO.writeUInt32BE(80, 36); // slice offset
machO.writeUInt32BE(32, 40); // slice size
machO.writeUInt32BE(12, 44); // alignment
// slice 0 (x86_64): 64-bit little-endian Mach-O header
machO.writeUInt32BE(0xcffaedfe, 48); // MH_MAGIC_64 stored little-endian, read BE
machO.writeUInt32LE(0x01000007, 52); // cputype
machO.writeUInt32LE(3, 56); // cpusubtype
machO.writeUInt32LE(6, 60); // filetype MH_DYLIB
machO.writeUInt32LE(0, 64); // ncmds
machO.writeUInt32LE(0, 68); // sizeofcmds
machO.writeUInt32LE(0, 72); // flags
machO.writeUInt32LE(0, 76); // reserved
// slice 1 (arm64): 64-bit little-endian Mach-O header
machO.writeUInt32BE(0xcffaedfe, 80);
machO.writeUInt32LE(0x0100000c, 84); // cputype
machO.writeUInt32LE(0, 88); // cpusubtype
machO.writeUInt32LE(6, 92); // filetype MH_DYLIB
machO.writeUInt32LE(0, 96); // ncmds
machO.writeUInt32LE(0, 100); // sizeofcmds
machO.writeUInt32LE(0, 104); // flags
machO.writeUInt32LE(0, 108); // reserved
machO.write(MACHO_ENTRY, 112, "latin1");

await Promise.all([
  writeFile(path.join(output, "libgamestruments_godot.so"), elf),
  writeFile(path.join(output, "gamestruments_godot.dll"), pe),
  writeFile(path.join(output, "libgamestruments_godot.dylib"), machO),
]);
