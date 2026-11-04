import { describe, expect, it } from "vitest";
import { getLocale, setLocale, t, translations } from "./i18n";

describe("i18n bilingual localization", () => {
  it("defaults to English locale", () => {
    setLocale(undefined);
    expect(getLocale()).toBe("en");
    expect(t("tabs.display")).toBe("Display");
    expect(t("tabs.system")).toBe("System");
    expect(t("pages.todo")).toBe("To-Do Notes");
  });

  it("switches accurately to Indonesian locale", () => {
    setLocale("id");
    expect(getLocale()).toBe("id");
    expect(t("tabs.display")).toBe("Layar");
    expect(t("tabs.system")).toBe("Sistem");
    expect(t("pages.todo")).toBe("Catatan To-Do");
    expect(t("settings.language")).toBe("Bahasa / Language");
  });

  it("handles parameter interpolation correctly", () => {
    setLocale("en");
    expect(t("peek.bluetoothConnected", { device: "Sony WH-1000XM5" })).toBe("Sony WH-1000XM5 Connected");
    expect(t("peek.bluetoothLow", { device: "Logitech MX Master" })).toBe("Logitech MX Master Low Battery");

    setLocale("id");
    expect(t("peek.bluetoothConnected", { device: "Sony WH-1000XM5" })).toBe("Sony WH-1000XM5 Terhubung");
    expect(t("peek.bluetoothLow", { device: "Logitech MX Master" })).toBe("Baterai Logitech MX Master Lemah");
  });

  it("falls back to English when a key is missing in another locale", () => {
    setLocale("id");
    expect(t("some.nonexistent.key")).toBe("some.nonexistent.key");
  });
});
