import { describe, expect, it } from "vitest";
import { navAction, nextIndex } from "./keyboard";
import { escapeOutcome } from "./escape";

describe("navAction", () => {
  it("maps arrows and ctrl-j/k", () => {
    expect(navAction({ key: "ArrowDown", ctrlKey: false, shiftKey: false, altKey: false, metaKey: false })).toBe(
      "down",
    );
    expect(navAction({ key: "j", ctrlKey: true, shiftKey: false, altKey: false, metaKey: false })).toBe("down");
    expect(navAction({ key: "k", ctrlKey: true, shiftKey: false, altKey: false, metaKey: false })).toBe("up");
    expect(
      navAction({ key: "ArrowLeft", ctrlKey: false, shiftKey: false, altKey: false, metaKey: false }),
    ).toBe("left");
    expect(
      navAction({ key: "d", ctrlKey: true, shiftKey: false, altKey: false, metaKey: false }),
    ).toBe("favorite");
  });

  it("copies with Enter even inside the search field", () => {
    const input = { tagName: "INPUT" } as unknown as EventTarget;
    expect(
      navAction({
        key: "Enter",
        ctrlKey: false,
        shiftKey: false,
        altKey: false,
        metaKey: false,
        target: input,
      }),
    ).toBe("copy");
  });

  it("maps escape, search, clear", () => {
    expect(navAction({ key: "Escape", ctrlKey: false, shiftKey: false, altKey: false, metaKey: false })).toBe(
      "escape",
    );
    expect(navAction({ key: "f", ctrlKey: true, shiftKey: false, altKey: false, metaKey: false })).toBe("search");
    expect(navAction({ key: "Delete", ctrlKey: true, shiftKey: true, altKey: false, metaKey: false })).toBe(
      "clear",
    );
  });
});

describe("nextIndex", () => {
  it("wraps", () => {
    expect(nextIndex(0, 3, -1)).toBe(2);
    expect(nextIndex(2, 3, 1)).toBe(0);
  });
});

describe("escapeOutcome", () => {
  it("clears search first", () => {
    expect(escapeOutcome("foo")).toBe("clear-search");
    expect(escapeOutcome("")).toBe("hide");
  });
});
