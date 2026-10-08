export function verifyElfX8664(buffer: Buffer): void;
export function verifyPeX8664(buffer: Buffer): void;
export function verifyFatMachO(buffer: Buffer): void;
export interface WasmModuleSummary {
  customSections: string[];
  imports: { module: string; field: string; kind: number; shared: boolean }[];
  exports: { name: string; kind: number }[];
}
export function inspectWasmModule(buffer: Buffer): WasmModuleSummary;
export function verifyWasmSideModule(buffer: Buffer): void;
export function assertEntryPointMarker(buffer: Buffer, label?: string): void;
export function findPrivateBuildPaths(buffer: Buffer): string[];
export function verifyNativeAssets(assetsDir: string): Promise<void>;
export function verifyPackagedNativeLibraries(root: string): Promise<void>;
