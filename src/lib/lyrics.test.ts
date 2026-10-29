import { describe, expect, it } from "vitest";
import { cleanTrack, currentLine, parseLrc } from "./lyrics";

describe("cleanTrack", () => {
  it("strips bracketed noise and suffixes from YouTube/browser titles", () => {
    const res = cleanTrack("Tulus - Hati-Hati di Jalan (Official Music Video)", "TulusVEVO", "chrome");
    expect(res.artist).toBe("Tulus");
    expect(res.title).toBe("Hati-Hati di Jalan");
  });

  it("handles reversed Artist - Title and Title - Artist formats", () => {
    const res = cleanTrack("Bohemian Rhapsody - Queen", "Queen", "spotify");
    expect(res.title).toBe("Bohemian Rhapsody");
    expect(res.artist).toBe("Queen");
  });

  it("cleans trailing labels like Topic and Official", () => {
    const res = cleanTrack("Song Name", "Artist Name - Topic", "spotify");
    expect(res.artist).toBe("Artist Name");
    expect(res.title).toBe("Song Name");
  });
});

describe("parseLrc", () => {
  it("parses timestamps and lines sorted chronologically", () => {
    const lrc = `[00:12.50]First line\r\n[00:05.00]Intro\r\n[00:20.00]Chorus`;
    const parsed = parseLrc(lrc);
    expect(parsed).toHaveLength(3);
    expect(parsed[0]).toEqual({ time: 5, text: "Intro" });
    expect(parsed[1]).toEqual({ time: 12.5, text: "First line" });
    expect(parsed[2]).toEqual({ time: 20, text: "Chorus" });
  });

  it("replaces empty timestamped lines with music note symbol", () => {
    const lrc = `[00:01.00]   `;
    const parsed = parseLrc(lrc);
    expect(parsed[0].text).toBe("♪");
  });
});

describe("currentLine", () => {
  it("identifies the active line based on playback position", () => {
    const lines = [
      { time: 10, text: "Line 1" },
      { time: 20, text: "Line 2" },
      { time: 30, text: "Line 3" },
    ];
    expect(currentLine(lines, 5)).toBe(-1);
    expect(currentLine(lines, 10)).toBe(0);
    expect(currentLine(lines, 15)).toBe(0);
    expect(currentLine(lines, 20)).toBe(1);
    expect(currentLine(lines, 35)).toBe(2);
  });
});
