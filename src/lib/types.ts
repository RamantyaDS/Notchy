export type { Call, Download } from "./island";
export type { ClipEntry } from "./ClipboardPage.svelte";

export type Power = { charging: boolean; percent: number; battery: boolean };
export type InUse = { camera: string[]; mic: string[] };
