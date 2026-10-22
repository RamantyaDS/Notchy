import { describe, expect, it } from "vitest";
import { meetingLink, upcoming } from "./calendar";

describe("meetingLink", () => {
  it("finds Teams, Meet, Zoom and Webex links", () => {
    expect(meetingLink("Join: https://teams.microsoft.com/l/meetup-join/19%3ameeting_abc/0?context=x now")).toBe(
      "https://teams.microsoft.com/l/meetup-join/19%3ameeting_abc/0?context=x",
    );
    expect(meetingLink("https://meet.google.com/abc-defg-hij")).toBe("https://meet.google.com/abc-defg-hij");
    expect(meetingLink("Zoom: https://us02web.zoom.us/j/123456789?pwd=XyZ")).toBe("https://us02web.zoom.us/j/123456789?pwd=XyZ");
    expect(meetingLink("https://acme.webex.com/meet/budi")).toBe("https://acme.webex.com/meet/budi");
  });

  it("ignores other links and takes the first text that has one", () => {
    expect(meetingLink("https://example.com/agenda")).toBe("");
    expect(meetingLink("Ruang 3", "https://meet.google.com/xyz-abcd-efg")).toBe("https://meet.google.com/xyz-abcd-efg");
  });
});

describe("upcoming", () => {
  it("carries the meeting link from the description", () => {
    const now = Date.UTC(2026, 9, 22, 1, 0, 0);
    const ics = [
      "BEGIN:VCALENDAR",
      "BEGIN:VEVENT",
      "UID:1",
      "SUMMARY:Daily stand-up",
      "DTSTART:20261022T020000Z",
      "DTEND:20261022T021500Z",
      "DESCRIPTION:Agenda\\nJoin: https://meet.google.com/abc-defg-hij\\nThanks",
      "END:VEVENT",
      "END:VCALENDAR",
    ].join("\r\n");
    const [e] = upcoming(ics, now);
    expect(e.title).toBe("Daily stand-up");
    expect(e.link).toBe("https://meet.google.com/abc-defg-hij");
  });
});
