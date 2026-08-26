import { describe, expect, it } from "vitest";
import { relativeTime } from "./time";

describe("relativeTime", () => {
  it("formats buckets", () => {
    const now = 1_000_000;
    expect(relativeTime(now, now)).toBe("just now");
    expect(relativeTime(now - 45_000, now)).toBe("45s ago");
    expect(relativeTime(now - 5 * 60_000, now)).toBe("5 min ago");
  });
});
