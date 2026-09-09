export function verifyElfX8664(buffer: Buffer): void;
export function verifyPeX8664(buffer: Buffer): void;
export function verifyFatMachO(buffer: Buffer): void;
export function assertEntryPointMarker(buffer: Buffer, label?: string): void;
export function findPrivateBuildPaths(buffer: Buffer): string[];
export function verifyNativeAssets(assetsDir: string): Promise<void>;
export function verifyPackagedNativeLibraries(root: string): Promise<void>;
