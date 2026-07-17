import { describe, expect, it } from "vitest";
import { formatDuration, formatSessionRange, initials, timeAgo, topicColor } from "./presentation";

describe("presentation helpers", () => {
  it("formats short and long session durations", () => {
    expect(formatDuration(0)).toBe("0m");
    expect(formatDuration(59 * 60_000)).toBe("59m");
    expect(formatDuration(2 * 60 * 60_000 + 16 * 60_000)).toBe("2h 16m");
  });

  it("keeps topic colors stable", () => {
    expect(topicColor("docker")).toBe(topicColor("docker"));
    expect(topicColor("docker")).toMatch(/^#/);
  });

  it("derives compact local source initials", () => {
    expect(initials("docs.docker.com")).toBe("DD");
    expect(initials("Visual Studio Code")).toBe("VS");
    expect(initials("")).toBe("M");
  });

  it("formats timestamps without invalid output", () => {
    const now = Date.now();
    expect(timeAgo(now - 90_000)).toBe("1m ago");
    expect(formatSessionRange(now, now + 30 * 60_000)).not.toContain("Invalid");
  });
});
