import { describe, expect, it } from "vitest";
import {
  assertEntryPointMarker,
  findPrivateBuildPaths,
  verifyElfX8664,
  verifyFatMachO,
  verifyPeX8664,
} from "../tools/verify-native-libraries.mjs";

const ENTRY = "gdext_rust_init";
const MACHO_ENTRY = "_gdext_rust_init";

function elfX8664(): Buffer {
  const buffer = Buffer.alloc(128);
  Buffer.from([0x7f, 0x45, 0x4c, 0x46]).copy(buffer);
  buffer[4] = 2;
  buffer[5] = 1;
  buffer.writeUInt16LE(3, 16);
  buffer.writeUInt16LE(62, 18);
  buffer.write(ENTRY, 64, "latin1");
  return buffer;
}

function peX8664(): Buffer {
  const buffer = Buffer.alloc(160);
  buffer.write("MZ");
  buffer.writeUInt32LE(64, 0x3c);
  buffer.write("PE\0\0", 64);
  buffer.writeUInt16LE(0x8664, 68);
  buffer.writeUInt16LE(0x2000, 86);
  buffer.write(ENTRY, 128, "latin1");
  return buffer;
}

function fatMachO(): Buffer {
  const buffer = Buffer.alloc(160);
  buffer.writeUInt32BE(0xcafebabe, 0);
  buffer.writeUInt32BE(2, 4);
  buffer.writeUInt32BE(0x01000007, 8);
  buffer.writeUInt32BE(3, 12);
  buffer.writeUInt32BE(48, 16);
  buffer.writeUInt32BE(32, 20);
  buffer.writeUInt32BE(12, 24);
  buffer.writeUInt32BE(0x0100000c, 28);
  buffer.writeUInt32BE(0, 32);
  buffer.writeUInt32BE(80, 36);
  buffer.writeUInt32BE(32, 40);
  buffer.writeUInt32BE(12, 44);
  buffer.writeUInt32BE(0xcffaedfe, 48);
  buffer.writeUInt32LE(0x01000007, 52);
  buffer.writeUInt32LE(3, 56);
  buffer.writeUInt32LE(6, 60);
  buffer.writeUInt32LE(0, 64);
  buffer.writeUInt32LE(0, 68);
  buffer.writeUInt32LE(0, 72);
  buffer.writeUInt32LE(0, 76);
  buffer.writeUInt32BE(0xcffaedfe, 80);
  buffer.writeUInt32LE(0x0100000c, 84);
  buffer.writeUInt32LE(0, 88);
  buffer.writeUInt32LE(6, 92);
  buffer.writeUInt32LE(0, 96);
  buffer.writeUInt32LE(0, 100);
  buffer.writeUInt32LE(0, 104);
  buffer.writeUInt32LE(0, 108);
  buffer.write(MACHO_ENTRY, 112, "latin1");
  return buffer;
}

describe("native release library inspection", () => {
  it("accepts the expected native formats, architectures, and entry point", () => {
    expect(() => verifyElfX8664(elfX8664())).not.toThrow();
    expect(() => verifyPeX8664(peX8664())).not.toThrow();
    expect(() => verifyFatMachO(fatMachO())).not.toThrow();
    expect(() => assertEntryPointMarker(elfX8664())).not.toThrow();
    expect(() => assertEntryPointMarker(peX8664())).not.toThrow();
    expect(() => assertEntryPointMarker(fatMachO())).not.toThrow();
  });

  it("rejects a renamed ELF as a Windows or macOS library", () => {
    expect(() => verifyPeX8664(elfX8664())).toThrow(/Windows library/);
    expect(() => verifyFatMachO(elfX8664())).toThrow(/macOS library/);
  });

  it("rejects ELF libraries with the wrong class, byte order, type, or architecture", () => {
    const wrongClass = elfX8664();
    wrongClass[4] = 1;
    expect(() => verifyElfX8664(wrongClass)).toThrow(/64-bit ELF/);

    const wrongByteOrder = elfX8664();
    wrongByteOrder[5] = 0;
    expect(() => verifyElfX8664(wrongByteOrder)).toThrow(/byte order/);

    const wrongType = elfX8664();
    wrongType.writeUInt16LE(2, 16);
    expect(() => verifyElfX8664(wrongType)).toThrow(/shared object/);

    const wrongArch = elfX8664();
    wrongArch.writeUInt16LE(40, 18);
    expect(() => verifyElfX8664(wrongArch)).toThrow(/x86_64/);
  });

  it("rejects PE libraries with the wrong architecture or missing the DLL flag", () => {
    const wrongArch = peX8664();
    wrongArch.writeUInt16LE(0xaa64, 68);
    expect(() => verifyPeX8664(wrongArch)).toThrow(/x86_64/);

    const notDll = peX8664();
    notDll.writeUInt16LE(0x0000, 86);
    expect(() => verifyPeX8664(notDll)).toThrow(/DLL/);
  });

  it("rejects malformed, truncated, mismatched, and non-dylib Mach-O slices", () => {
    const malformed = fatMachO();
    malformed.writeUInt32BE(0x00000000, 0);
    expect(() => verifyFatMachO(malformed)).toThrow(/not a fat Mach-O/);

    const truncated = fatMachO();
    truncated.writeUInt32BE(200, 20); // slice 0 size exceeds the buffer
    expect(() => verifyFatMachO(truncated)).toThrow(/truncated/);

    const mismatched = fatMachO();
    mismatched.writeUInt32LE(0x0100000c, 52); // slice cputype disagrees with its table entry
    expect(() => verifyFatMachO(mismatched)).toThrow(/does not match/);

    const notDylib = fatMachO();
    notDylib.writeUInt32LE(2, 60); // MH_EXECUTE instead of MH_DYLIB
    expect(() => verifyFatMachO(notDylib)).toThrow(/MH_DYLIB/);
  });

  it("rejects a binary that does not export the entry point", () => {
    const missing = elfX8664();
    missing.fill(0, 64, 128); // erase the marker
    expect(() => assertEntryPointMarker(missing, "linux library")).toThrow(/gdext_rust_init/);
  });

  it("finds developer and CI paths but permits remapped toolchain paths", () => {
    const leaked = Buffer.from(
      [
        "/home/fran/Workspace/gamestruments/src/lib.rs",
        "/Users/runner/work/gamestruments/gamestruments/src/lib.rs",
        "/__w/gamestruments/gamestruments/src/lib.rs",
        "C:\\Users\\runneradmin\\.cargo\\registry\\src\\godot-core\\src\\lib.rs",
        "D:\\a\\gamestruments\\gamestruments\\src\\lib.rs",
      ].join("\0"),
    );
    expect(findPrivateBuildPaths(leaked)).toHaveLength(5);
    expect(
      findPrivateBuildPaths(Buffer.from("/workspace/crates/godot/src/lib.rs\0/cargo/registry/src/godot-core/src/lib.rs\0/rustc/hash/library/std/src/lib.rs")),
    ).toEqual([]);
  });
});
