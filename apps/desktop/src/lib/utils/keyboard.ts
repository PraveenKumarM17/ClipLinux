export type NavAction =
  | "up"
  | "down"
  | "copy"
  | "escape"
  | "search"
  | "delete"
  | "clear";

function isTextField(target: EventTarget | null | undefined): boolean {
  if (!target || typeof target !== "object") {
    return false;
  }
  const tag = (target as { tagName?: string }).tagName;
  return tag === "INPUT" || tag === "TEXTAREA";
}

/** Map a key event to a palette action. Returns null when unhandled. */
export function navAction(event: {
  key: string;
  ctrlKey: boolean;
  shiftKey: boolean;
  altKey: boolean;
  metaKey: boolean;
  target?: EventTarget | null;
}): NavAction | null {
  const inField = isTextField(event.target);

  if (event.key === "Escape") {
    return "escape";
  }
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "f") {
    return "search";
  }
  if ((event.ctrlKey || event.metaKey) && event.shiftKey && event.key === "Delete") {
    return "clear";
  }
  if (event.key === "Enter" && !event.ctrlKey && !event.altKey && !event.metaKey) {
    return "copy";
  }
  if (event.key === "ArrowDown" || ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "j")) {
    return "down";
  }
  if (event.key === "ArrowUp" || ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k")) {
    return "up";
  }
  if (event.key === "Delete" && !inField && !event.shiftKey) {
    return "delete";
  }
  return null;
}

export function nextIndex(current: number, len: number, delta: number): number {
  if (len <= 0) {
    return 0;
  }
  return (current + delta + len) % len;
}
