export type GridDir = "left" | "right" | "up" | "down";

/** Move a linear index on a CSS grid. Clamps at the ends. */
export function moveGridIndex(
  current: number,
  cols: number,
  len: number,
  dir: GridDir,
): number {
  if (len <= 0 || cols <= 0) {
    return 0;
  }
  const idx = Math.min(Math.max(current, 0), len - 1);
  let next = idx;
  switch (dir) {
    case "left":
      next = idx - 1;
      break;
    case "right":
      next = idx + 1;
      break;
    case "up":
      next = idx - cols;
      break;
    case "down":
      next = idx + cols;
      break;
  }
  if (next < 0) {
    return 0;
  }
  if (next >= len) {
    return len - 1;
  }
  return next;
}
