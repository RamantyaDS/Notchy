export type NetworkSpeed = {
  down: number;
  up: number;
};

export const DEFAULT_SPEED_THRESHOLD_DOWN = 500_000; // 500 KB/s
export const DEFAULT_SPEED_THRESHOLD_UP = 250_000;   // 250 KB/s

/**
 * Formats bytes/sec into a clean, human-readable speed string.
 * Example: 450 KB/s, 1.2 MB/s, 15 MB/s
 */
export function formatSpeed(bytesPerSec: number): string {
  if (!Number.isFinite(bytesPerSec) || bytesPerSec <= 0) {
    return "0 KB/s";
  }

  const k = 1024;
  const m = k * 1024;

  if (bytesPerSec < k) {
    return "< 1 KB/s";
  }

  if (bytesPerSec < m) {
    return `${Math.round(bytesPerSec / k)} KB/s`;
  }

  const mb = Math.round((bytesPerSec / m) * 10) / 10;
  return mb >= 10 ? `${Math.round(mb)} MB/s` : `${mb.toFixed(1)} MB/s`;
}

/**
 * Determines whether traffic is high enough to trigger the Dynamic Island.
 */
export function isHighTraffic(
  speed: NetworkSpeed | null | undefined,
  thresholdDown = DEFAULT_SPEED_THRESHOLD_DOWN,
  thresholdUp = DEFAULT_SPEED_THRESHOLD_UP
): boolean {
  if (!speed) return false;
  const downActive = thresholdDown <= 0 ? speed.down > 0 : speed.down >= thresholdDown;
  const upActive = thresholdUp <= 0 ? speed.up > 0 : speed.up >= thresholdUp;
  return downActive || upActive;
}
