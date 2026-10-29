import { describe, expect, it } from "vitest";
import {
  DEFAULT_SPEED_THRESHOLD_DOWN,
  DEFAULT_SPEED_THRESHOLD_UP,
  formatSpeed,
  isHighTraffic,
  type NetworkSpeed,
} from "./network";

describe("formatSpeed", () => {
  it("formats zero and negative speeds gracefully", () => {
    expect(formatSpeed(0)).toBe("0 KB/s");
    expect(formatSpeed(-100)).toBe("0 KB/s");
    expect(formatSpeed(NaN)).toBe("0 KB/s");
  });

  it("formats sub-kilobyte speeds", () => {
    expect(formatSpeed(500)).toBe("< 1 KB/s");
    expect(formatSpeed(1023)).toBe("< 1 KB/s");
  });

  it("formats kilobyte speeds cleanly", () => {
    expect(formatSpeed(1024)).toBe("1 KB/s");
    expect(formatSpeed(450 * 1024)).toBe("450 KB/s");
    expect(formatSpeed(1023 * 1024)).toBe("1023 KB/s");
  });

  it("formats megabyte speeds with one decimal when under 10 MB/s", () => {
    expect(formatSpeed(1024 * 1024)).toBe("1.0 MB/s");
    expect(formatSpeed(1.45 * 1024 * 1024)).toBe("1.5 MB/s");
    expect(formatSpeed(9.8 * 1024 * 1024)).toBe("9.8 MB/s");
  });

  it("formats megabyte speeds rounded when 10 MB/s or higher", () => {
    expect(formatSpeed(10 * 1024 * 1024)).toBe("10 MB/s");
    expect(formatSpeed(24.7 * 1024 * 1024)).toBe("25 MB/s");
    expect(formatSpeed(120 * 1024 * 1024)).toBe("120 MB/s");
  });
});

describe("isHighTraffic", () => {
  it("returns false for null or undefined speed", () => {
    expect(isHighTraffic(null)).toBe(false);
    expect(isHighTraffic(undefined)).toBe(false);
  });

  it("returns false when both upload and download are below threshold", () => {
    const speed: NetworkSpeed = {
      down: DEFAULT_SPEED_THRESHOLD_DOWN - 1,
      up: DEFAULT_SPEED_THRESHOLD_UP - 1,
    };
    expect(isHighTraffic(speed)).toBe(false);
  });

  it("returns true when download meets or exceeds threshold", () => {
    const speed: NetworkSpeed = {
      down: DEFAULT_SPEED_THRESHOLD_DOWN,
      up: 10_000,
    };
    expect(isHighTraffic(speed)).toBe(true);
  });

  it("returns true when upload meets or exceeds threshold", () => {
    const speed: NetworkSpeed = {
      down: 20_000,
      up: DEFAULT_SPEED_THRESHOLD_UP,
    };
    expect(isHighTraffic(speed)).toBe(true);
  });

  it("supports custom thresholds", () => {
    const speed: NetworkSpeed = { down: 100_000, up: 50_000 };
    expect(isHighTraffic(speed, 200_000, 100_000)).toBe(false);
    expect(isHighTraffic(speed, 50_000, 50_000)).toBe(true);
  });
});
