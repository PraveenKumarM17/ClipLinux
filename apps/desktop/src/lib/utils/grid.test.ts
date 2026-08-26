import { describe, expect, it } from "vitest";
import { moveGridIndex } from "./grid";

describe("moveGridIndex", () => {
  it("moves by cell and by row", () => {
    expect(moveGridIndex(5, 8, 20, "left")).toBe(4);
    expect(moveGridIndex(5, 8, 20, "right")).toBe(6);
    expect(moveGridIndex(10, 8, 20, "up")).toBe(2);
    expect(moveGridIndex(2, 8, 20, "down")).toBe(10);
  });

  it("clamps at edges", () => {
    expect(moveGridIndex(0, 8, 20, "left")).toBe(0);
    expect(moveGridIndex(0, 8, 20, "up")).toBe(0);
    expect(moveGridIndex(19, 8, 20, "right")).toBe(19);
    expect(moveGridIndex(19, 8, 20, "down")).toBe(19);
  });
});
