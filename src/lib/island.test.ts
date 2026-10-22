import { describe, expect, it } from "vitest";
import { arrange, bytes, duration, islandWidth, type Activity } from "./island";

const call: Activity = { kind: "call", call: { app: "Teams", start: 0 } };
const timer: Activity = { kind: "timer" };
const download: Activity = { kind: "download", download: { name: "a.pdf", bytes: 1, speed: 1 } };
const music: Activity = { kind: "music", lyric: false };
const sung: Activity = { kind: "music", lyric: true };

describe("arrange", () => {
  it("is empty without activities", () => {
    expect(arrange([])).toEqual({ main: null, side: [] });
  });

  it("puts the call first, then timer, download, music", () => {
    expect(arrange([music, download, timer, call])).toEqual({ main: call, side: [timer, download] });
    expect(arrange([music, timer])).toEqual({ main: timer, side: [music] });
  });

  it("keeps music with a sung line in front", () => {
    expect(arrange([timer, sung, call])).toEqual({ main: sung, side: [call, timer] });
  });

  it("shows at most two beside the main one", () => {
    expect(arrange([call, timer, download, music]).side).toHaveLength(2);
  });

  it("cycles the main activity with swipes, both ways", () => {
    expect(arrange([call, timer, music], 1).main).toBe(timer);
    expect(arrange([call, timer, music], 2).main).toBe(music);
    expect(arrange([call, timer, music], 3).main).toBe(call);
    expect(arrange([call, timer, music], -1).main).toBe(music);
  });
});

describe("sizes and labels", () => {
  it("widens the pill for each side activity", () => {
    expect(islandWidth(music, [])).toBe(200);
    expect(islandWidth(sung, [timer])).toBe(380 + 84);
  });

  it("formats durations", () => {
    expect(duration(0)).toBe("0:00");
    expect(duration(754_000)).toBe("12:34");
    expect(duration(3_723_000)).toBe("1:02:03");
  });

  it("formats sizes the Indonesian way", () => {
    expect(bytes(512)).toBe("512 B");
    expect(bytes(12_400_000)).toBe("12 MB");
    expect(bytes(3_100_000)).toBe("3,1 MB");
    expect(bytes(1_500_000_000)).toBe("1,5 GB");
  });
});
