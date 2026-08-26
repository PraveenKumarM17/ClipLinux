import { describe, expect, it } from "vitest";
import { listSurface } from "./historyView";

describe("listSurface", () => {
  it("never treats a missing daemon as empty history", () => {
    expect(
      listSurface({
        connectionKind: "disconnected",
        historyError: null,
        query: "",
        itemCount: 0,
      }),
    ).toBe("disconnected");
  });

  it("shows empty vs no-results only when connected", () => {
    expect(
      listSurface({
        connectionKind: "connected",
        historyError: null,
        query: "",
        itemCount: 0,
      }),
    ).toBe("empty");
    expect(
      listSurface({
        connectionKind: "connected",
        historyError: null,
        query: "xyz",
        itemCount: 0,
      }),
    ).toBe("no-results");
  });

  it("prefers history errors over an empty list", () => {
    expect(
      listSurface({
        connectionKind: "connected",
        historyError: "socket reset",
        query: "",
        itemCount: 0,
      }),
    ).toBe("history-error");
  });
});
